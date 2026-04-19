use axum::extract::{Query, State};
use axum::routing::{get, post};
use axum::{Extension, Router};
use std::collections::HashMap;

use crate::error::ApiError;
use crate::middleware::tenant::Tenant;
use crate::state::AppState;

pub async fn request(
    State(_state): State<AppState>,
    Extension(_tenant): Extension<Tenant>,
) -> Result<(), ApiError> {
    Err(ApiError::NotImplemented)
}

pub async fn verify(
    State(_state): State<AppState>,
    Extension(_tenant): Extension<Tenant>,
    Query(_q): Query<HashMap<String, String>>,
) -> Result<(), ApiError> {
    Err(ApiError::NotImplemented)
}

pub fn router() -> Router<AppState> {
    Router::new().route("/magic/request", post(request)).route("/magic/verify", get(verify))
}
