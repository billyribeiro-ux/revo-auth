use async_trait::async_trait;
use serde_json::Value;

#[derive(Debug, thiserror::Error)]
pub enum EmailError {
    #[error("smtp: {0}")]
    Smtp(String),
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

#[async_trait]
pub trait EmailTransport: Send + Sync {
    async fn send(&self, msg: EmailMessage) -> Result<(), EmailError>;
}

pub struct LoggingTransport;

#[async_trait]
impl EmailTransport for LoggingTransport {
    async fn send(&self, msg: EmailMessage) -> Result<(), EmailError> {
        tracing::info!(to = %msg.to, subject = %msg.subject, "email (log transport)");
        Ok(())
    }
}
