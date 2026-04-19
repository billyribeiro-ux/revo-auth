use axum::extract::{Path, State};
use axum::routing::{delete, get, post};
use axum::{Extension, Router};
use uuid::Uuid;

use crate::error::ApiError;
use crate::middleware::tenant::Tenant;
use crate::state::AppState;

pub async fn register_begin(
    State(_state): State<AppState>,
    Extension(_tenant): Extension<Tenant>,
) -> Result<(), ApiError> {
    Err(ApiError::NotImplemented)
}

pub async fn register_finish(
    State(_state): State<AppState>,
    Extension(_tenant): Extension<Tenant>,
) -> Result<(), ApiError> {
    Err(ApiError::NotImplemented)
}

pub async fn authenticate_begin(
    State(_state): State<AppState>,
    Extension(_tenant): Extension<Tenant>,
) -> Result<(), ApiError> {
    Err(ApiError::NotImplemented)
}

pub async fn authenticate_finish(
    State(_state): State<AppState>,
    Extension(_tenant): Extension<Tenant>,
) -> Result<(), ApiError> {
    Err(ApiError::NotImplemented)
}

pub async fn list_passkeys(
    State(_state): State<AppState>,
    Extension(_tenant): Extension<Tenant>,
) -> Result<(), ApiError> {
    Err(ApiError::NotImplemented)
}

pub async fn delete_passkey(
    State(_state): State<AppState>,
    Extension(_tenant): Extension<Tenant>,
    Path(_id): Path<Uuid>,
) -> Result<(), ApiError> {
    Err(ApiError::NotImplemented)
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/passkey/register/begin", post(register_begin))
        .route("/passkey/register/finish", post(register_finish))
        .route("/passkey/authenticate/begin", post(authenticate_begin))
        .route("/passkey/authenticate/finish", post(authenticate_finish))
        .route("/passkey", get(list_passkeys))
        .route("/passkey/{id}", delete(delete_passkey))
}
