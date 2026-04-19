use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Extension, Router};
use uuid::Uuid;

use crate::error::ApiError;
use crate::middleware::tenant::Tenant;
use crate::state::AppState;

pub async fn list_orgs(
    State(_state): State<AppState>,
    Extension(_tenant): Extension<Tenant>,
) -> Result<(), ApiError> {
    Err(ApiError::NotImplemented)
}

pub async fn create_org(
    State(_state): State<AppState>,
    Extension(_tenant): Extension<Tenant>,
) -> Result<(), ApiError> {
    Err(ApiError::NotImplemented)
}

pub async fn invite(
    State(_state): State<AppState>,
    Extension(_tenant): Extension<Tenant>,
    Path(_id): Path<Uuid>,
) -> Result<(), ApiError> {
    Err(ApiError::NotImplemented)
}

pub async fn accept(
    State(_state): State<AppState>,
    Extension(_tenant): Extension<Tenant>,
    Path(_id): Path<Uuid>,
) -> Result<(), ApiError> {
    Err(ApiError::NotImplemented)
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/orgs", get(list_orgs).post(create_org))
        .route("/orgs/{id}/invite", post(invite))
        .route("/orgs/{id}/accept", post(accept))
}
