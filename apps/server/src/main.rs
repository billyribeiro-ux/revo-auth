use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Context;
use revo_auth_server::config::Config;
use revo_auth_server::email::transport::LoggingTransport;
use revo_auth_server::state::AppState;
use revo_auth_server::{app_router, telemetry};
use sqlx::postgres::PgPoolOptions;

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

    let state = AppState {
        pool,
        redis,
        config: Arc::new(config.clone()),
        mail: Arc::new(LoggingTransport),
    };

    let app = app_router(state);
    let addr: SocketAddr =
        format!("{}:{}", config.host, config.port).parse().context("bind addr")?;
    let listener = tokio::net::TcpListener::bind(addr).await.context("listen")?;
    tracing::info!(%addr, "revo-auth-server listening");
    axum::serve(listener, app).await.context("serve")?;
    Ok(())
}
