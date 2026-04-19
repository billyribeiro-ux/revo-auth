use axum::extract::State;
use axum::routing::post;
use axum::{Extension, Router};

use crate::error::ApiError;
use crate::middleware::tenant::Tenant;
use crate::state::AppState;

pub async fn verify_request(
    State(_state): State<AppState>,
    Extension(_tenant): Extension<Tenant>,
) -> Result<(), ApiError> {
    Err(ApiError::NotImplemented)
}

pub async fn verify_confirm(
    State(_state): State<AppState>,
    Extension(_tenant): Extension<Tenant>,
) -> Result<(), ApiError> {
    Err(ApiError::NotImplemented)
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/email/verify/request", post(verify_request))
        .route("/email/verify/confirm", post(verify_confirm))
}
