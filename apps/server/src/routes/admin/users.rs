use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};

use crate::error::ApiError;
use crate::state::AppState;

use super::apps::require_master;
use axum::http::HeaderMap;

pub async fn list_users(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, ApiError> {
    require_master(&headers, &state.config.master_key)?;
    Err(ApiError::NotImplemented)
}

pub fn router() -> Router<AppState> {
    Router::new().route("/users", get(list_users))
}
