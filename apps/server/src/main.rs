use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Context;
use revo_auth_server::config::Config;
use revo_auth_server::email::transport::{
    EmailTransport, LogTransport, ResendTransport, SmtpTransport,
};
use revo_auth_server::state::AppState;
use revo_auth_server::{app_router, telemetry};
use sqlx::postgres::PgPoolOptions;

fn build_mail_transport(cfg: &Config) -> anyhow::Result<Arc<dyn EmailTransport>> {
    match cfg.email_provider.as_str() {
        "smtp" => {
            let host = cfg.smtp_host.as_deref().context("smtp_host")?;
            let port = cfg.smtp_port.context("smtp_port")?;
            let user = cfg.smtp_username.as_deref().context("smtp_username")?;
            let pass = cfg.smtp_password.as_deref().context("smtp_password")?;
            let t = SmtpTransport::new(host, port, user, pass, &cfg.email_from, cfg.smtp_starttls)
                .map_err(|e| anyhow::anyhow!("smtp transport: {e}"))?;
            Ok(Arc::new(t))
        }
        "resend" => {
            let key = cfg.resend_api_key.as_deref().context("resend_api_key")?;
            Ok(Arc::new(ResendTransport::new(key, &cfg.email_from)))
        }
        "log" | "" => Ok(Arc::new(LogTransport)),
        other => Err(anyhow::anyhow!("unknown email_provider: {other}")),
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::load().context("config")?;
    telemetry::init();

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await
        .context("database")?;
    sqlx::migrate!("./migrations").run(&pool).await.context("migrate")?;

    let redis = AppState::connect_redis(&config.redis_url).await.context("redis")?;

    let mail = build_mail_transport(&config).context("email transport")?;

    let state = AppState { pool, redis, config: Arc::new(config.clone()), mail };

    let app = app_router(state);
    let addr: SocketAddr =
        format!("{}:{}", config.host, config.port).parse().context("bind addr")?;
    let listener = tokio::net::TcpListener::bind(addr).await.context("listen")?;
    tracing::info!(%addr, "revo-auth-server listening");
    axum::serve(listener, app).await.context("serve")?;
    Ok(())
}
