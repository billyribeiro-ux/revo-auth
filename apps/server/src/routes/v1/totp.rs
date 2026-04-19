use axum::extract::State;
use axum::routing::post;
use axum::{Extension, Router};

use crate::error::ApiError;
use crate::middleware::tenant::Tenant;
use crate::state::AppState;

pub async fn setup(
    State(_state): State<AppState>,
    Extension(_tenant): Extension<Tenant>,
) -> Result<(), ApiError> {
    Err(ApiError::NotImplemented)
}

pub async fn confirm(
    State(_state): State<AppState>,
    Extension(_tenant): Extension<Tenant>,
) -> Result<(), ApiError> {
    Err(ApiError::NotImplemented)
}

pub async fn verify(
    State(_state): State<AppState>,
    Extension(_tenant): Extension<Tenant>,
) -> Result<(), ApiError> {
    Err(ApiError::NotImplemented)
}

pub async fn disable(
    State(_state): State<AppState>,
    Extension(_tenant): Extension<Tenant>,
) -> Result<(), ApiError> {
    Err(ApiError::NotImplemented)
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/totp/setup", post(setup))
        .route("/totp/confirm", post(confirm))
        .route("/totp/verify", post(verify))
        .route("/totp/disable", post(disable))
}
