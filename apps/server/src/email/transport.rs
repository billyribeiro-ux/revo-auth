use async_trait::async_trait;
use serde_json::Value;

#[derive(Debug, thiserror::Error)]
pub enum EmailError {
    #[error("smtp: {0}")]
    Smtp(String),
    #[error("http: {0}")]
    Http(String),
    #[error("build: {0}")]
    Build(String),
    #[error("transport unavailable")]
    Unavailable,
}

#[derive(Debug, Clone)]
pub struct EmailMessage {
    pub to: String,
    pub subject: String,
    pub text_body: String,
    pub html_body: String,
    pub meta: Value,
}

/// Transport-agnostic email send contract. Implementations MUST never log
/// secrets (credentials, api keys, full message bodies in prod).
#[async_trait]
pub trait EmailTransport: Send + Sync {
    async fn send(&self, msg: EmailMessage) -> Result<(), EmailError>;
}

// ---------------------------------------------------------------------------
// LogTransport — dev-only, prints subject/to via tracing. Never logs bodies.
// ---------------------------------------------------------------------------

pub struct LogTransport;

#[async_trait]
impl EmailTransport for LogTransport {
    async fn send(&self, msg: EmailMessage) -> Result<(), EmailError> {
        tracing::info!(to = %msg.to, subject = %msg.subject, "email (log transport)");
        Ok(())
    }
}

/// Back-compat unit-struct for existing callers (tests, older bin wiring).
/// Delegates to `LogTransport`. Prefer `LogTransport` in new code.
pub struct LoggingTransport;

#[async_trait]
impl EmailTransport for LoggingTransport {
    async fn send(&self, msg: EmailMessage) -> Result<(), EmailError> {
        LogTransport.send(msg).await
    }
}

// ---------------------------------------------------------------------------
// SmtpTransport — lettre, TLS, opportunistic STARTTLS if host doesn't support
// SMTPS, credentials from config.
// ---------------------------------------------------------------------------

pub struct SmtpTransport {
    mailer: lettre::AsyncSmtpTransport<lettre::Tokio1Executor>,
    from: String,
}

impl SmtpTransport {
    pub fn new(
        host: &str,
        port: u16,
        username: &str,
        password: &str,
        from: &str,
        use_starttls: bool,
    ) -> Result<Self, EmailError> {
        use lettre::transport::smtp::authentication::Credentials;
        let creds = Credentials::new(username.to_string(), password.to_string());
        let builder = if use_starttls {
            lettre::AsyncSmtpTransport::<lettre::Tokio1Executor>::starttls_relay(host)
                .map_err(|e| EmailError::Smtp(e.to_string()))?
        } else {
            lettre::AsyncSmtpTransport::<lettre::Tokio1Executor>::relay(host)
                .map_err(|e| EmailError::Smtp(e.to_string()))?
        };
        let mailer = builder.credentials(creds).port(port).build();
        Ok(Self { mailer, from: from.to_string() })
    }
}

#[async_trait]
impl EmailTransport for SmtpTransport {
    async fn send(&self, msg: EmailMessage) -> Result<(), EmailError> {
        use lettre::message::{header, MultiPart, SinglePart};
        use lettre::{AsyncTransport, Message};

        let from: lettre::message::Mailbox = self
            .from
            .parse()
            .map_err(|e: lettre::address::AddressError| EmailError::Build(e.to_string()))?;
        let to: lettre::message::Mailbox = msg
            .to
            .parse()
            .map_err(|e: lettre::address::AddressError| EmailError::Build(e.to_string()))?;

        let email = Message::builder()
            .from(from)
            .to(to)
            .subject(msg.subject.clone())
            .multipart(
                MultiPart::alternative()
                    .singlepart(
                        SinglePart::builder()
                            .header(header::ContentType::TEXT_PLAIN)
                            .body(msg.text_body.clone()),
                    )
                    .singlepart(
                        SinglePart::builder()
                            .header(header::ContentType::TEXT_HTML)
                            .body(msg.html_body.clone()),
                    ),
            )
            .map_err(|e| EmailError::Build(e.to_string()))?;

        self.mailer.send(email).await.map_err(|e| EmailError::Smtp(e.to_string()))?;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// ResendTransport — HTTPS POST https://api.resend.com/emails with bearer auth.
// ---------------------------------------------------------------------------

pub struct ResendTransport {
    client: reqwest::Client,
    api_key: String,
    from: String,
}

impl ResendTransport {
    pub fn new(api_key: &str, from: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key: api_key.to_string(),
            from: from.to_string(),
        }
    }
}

#[async_trait]
impl EmailTransport for ResendTransport {
    async fn send(&self, msg: EmailMessage) -> Result<(), EmailError> {
        let body = serde_json::json!({
            "from": self.from,
            "to": [msg.to],
            "subject": msg.subject,
            "html": msg.html_body,
            "text": msg.text_body,
        });
        let res = self
            .client
            .post("https://api.resend.com/emails")
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| EmailError::Http(e.to_string()))?;
        if !res.status().is_success() {
            return Err(EmailError::Http(format!("resend status {}", res.status())));
        }
        Ok(())
    }
}
