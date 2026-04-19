use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Extension, Router};
use serde::Deserialize;

use crate::error::ApiError;
use crate::middleware::tenant::Tenant;
use crate::state::AppState;

pub async fn authorize(
    State(_state): State<AppState>,
    Extension(_tenant): Extension<Tenant>,
    Path(_provider): Path<String>,
) -> Result<(), ApiError> {
    Err(ApiError::NotImplemented)
}

#[derive(Deserialize)]
pub struct CallbackQuery {
    #[serde(rename = "code")]
    pub _code: Option<String>,
    #[serde(rename = "state")]
    pub _state: Option<String>,
}

pub async fn callback(
    State(_state): State<AppState>,
    Extension(_tenant): Extension<Tenant>,
    Path(_provider): Path<String>,
    axum::extract::Query(_q): axum::extract::Query<CallbackQuery>,
) -> Result<(), ApiError> {
    Err(ApiError::NotImplemented)
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/oauth/{provider}/authorize", get(authorize))
        .route("/oauth/{provider}/callback", get(callback))
}
