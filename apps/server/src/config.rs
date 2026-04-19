use figment::{providers::Env, Figment};
use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub redis_url: String,
    #[serde(default = "default_master_key")]
    pub master_key: String,
    pub encryption_key: String,
    pub jwt_issuer: String,
    pub jwt_es256_private_pem: String,
    pub jwt_es256_public_pem: String,
    #[serde(default = "default_cookie_secure")]
    pub cookie_secure: bool,

    // --- Email transport ---
    /// One of "smtp" | "resend" | "log". Defaults to "log" for dev.
    #[serde(default = "default_email_provider")]
    pub email_provider: String,
    #[serde(default = "default_email_from")]
    pub email_from: String,

    // SMTP (used when email_provider = "smtp")
    #[serde(default)]
    pub smtp_host: Option<String>,
    #[serde(default)]
    pub smtp_port: Option<u16>,
    #[serde(default)]
    pub smtp_username: Option<String>,
    #[serde(default)]
    pub smtp_password: Option<String>,
    #[serde(default = "default_smtp_starttls")]
    pub smtp_starttls: bool,

    // Resend (used when email_provider = "resend")
    #[serde(default)]
    pub resend_api_key: Option<String>,

    // --- App base URL (used to build email/verify/reset/magic links) ---
    #[serde(default = "default_app_base_url")]
    pub app_base_url: String,
}

fn default_master_key() -> String {
    String::new()
}

fn default_cookie_secure() -> bool {
    true
}

fn default_email_provider() -> String {
    "log".to_string()
}

fn default_email_from() -> String {
    "Revo Auth <noreply@example.invalid>".to_string()
}

fn default_smtp_starttls() -> bool {
    true
}

fn default_app_base_url() -> String {
    "https://example.invalid".to_string()
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let c: Config = Figment::new().merge(Env::prefixed("REVO_AUTH_").split("__")).extract()?;
        Ok(c)
    }
}

// Manual Debug that NEVER exposes secrets (database URL may contain password,
// master_key/encryption_key/jwt pem/smtp_password/resend_api_key all sensitive).
impl std::fmt::Debug for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Config")
            .field("host", &self.host)
            .field("port", &self.port)
            .field("database_url", &"<redacted>")
            .field("redis_url", &"<redacted>")
            .field("master_key", &"<redacted>")
            .field("encryption_key", &"<redacted>")
            .field("jwt_issuer", &self.jwt_issuer)
            .field("jwt_es256_private_pem", &"<redacted>")
            .field("jwt_es256_public_pem", &"<redacted>")
            .field("cookie_secure", &self.cookie_secure)
            .field("email_provider", &self.email_provider)
            .field("email_from", &self.email_from)
            .field("smtp_host", &self.smtp_host)
            .field("smtp_port", &self.smtp_port)
            .field("smtp_username", &self.smtp_username.as_ref().map(|_| "<redacted>"))
            .field("smtp_password", &"<redacted>")
            .field("smtp_starttls", &self.smtp_starttls)
            .field("resend_api_key", &"<redacted>")
            .field("app_base_url", &self.app_base_url)
            .finish()
    }
}
