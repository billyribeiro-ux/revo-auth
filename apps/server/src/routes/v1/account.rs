use axum::extract::{Path, State};
use axum::routing::{delete, post};
use axum::{Extension, Router};

use crate::error::ApiError;
use crate::middleware::tenant::Tenant;
use crate::state::AppState;

pub async fn link(
    State(_state): State<AppState>,
    Extension(_tenant): Extension<Tenant>,
    Path(_provider): Path<String>,
) -> Result<(), ApiError> {
    Err(ApiError::NotImplemented)
}

pub async fn unlink(
    State(_state): State<AppState>,
    Extension(_tenant): Extension<Tenant>,
    Path(_provider): Path<String>,
) -> Result<(), ApiError> {
    Err(ApiError::NotImplemented)
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/account/link/{provider}", post(link))
        .route("/account/{provider}", delete(unlink))
}
