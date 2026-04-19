use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use fred::prelude::*;
use serde::Serialize;

use crate::state::AppState;

#[derive(Serialize)]
pub struct HealthOk {
    pub status: &'static str,
}

pub async fn health() -> Json<HealthOk> {
    Json(HealthOk { status: "ok" })
}

#[derive(Serialize)]
pub struct ReadyOut {
    pub status: &'static str,
    pub database: bool,
    pub redis: bool,
}

pub async fn ready(State(state): State<AppState>) -> (StatusCode, Json<ReadyOut>) {
    let db_ok = sqlx::query_scalar::<_, i32>("SELECT 1").fetch_one(&state.pool).await.is_ok();
    let redis_ok = state.redis.get::<Option<String>, _>("revo:__health_check__").await.is_ok();
    let ok = db_ok && redis_ok;
    let status = if ok { StatusCode::OK } else { StatusCode::SERVICE_UNAVAILABLE };
    (
        status,
        Json(ReadyOut {
            status: if ok { "ready" } else { "not_ready" },
            database: db_ok,
            redis: redis_ok,
        }),
    )
}
