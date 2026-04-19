use figment::{providers::Env, Figment};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
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
}

fn default_master_key() -> String {
    String::new()
}

fn default_cookie_secure() -> bool {
    true
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let c: Config = Figment::new().merge(Env::prefixed("REVO_AUTH_").split("__")).extract()?;
        Ok(c)
    }
}
