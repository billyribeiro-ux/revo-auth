use axum::extract::State;
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::{Extension, Json, Router};
use garde::Validate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::crypto::password;
use crate::db::{audit, users};
use crate::error::ApiError;
use crate::middleware::tenant::Tenant;
use crate::routes::v1::session::{append_session_csrf_cookies, issue_session, SessionOut};
use crate::state::AppState;

#[derive(Deserialize, Validate)]
pub struct SignupBody {
    #[garde(email)]
    pub email: String,
    #[garde(length(min = 12))]
    pub password: String,
    #[garde(skip)]
    pub name: Option<String>,
}

#[derive(Serialize)]
pub struct SignupResponse {
    pub session: SessionOut,
}

pub async fn signup(
    State(state): State<AppState>,
    Extension(Tenant(app)): Extension<Tenant>,
    headers: HeaderMap,
    Json(body): Json<SignupBody>,
) -> Result<Response, ApiError> {
    body.validate().map_err(|e| ApiError::Validation(e.to_string()))?;
    password::validate_password_strength(&body.password)?;
    if users::find_by_email(&state.pool, app.id, &body.email)
        .await
        .map_err(|_| ApiError::Internal)?
        .is_some()
    {
        return Err(ApiError::Conflict);
    }
    let uid = Uuid::now_v7();
    let hash = password::hash_password(&body.password)?;
    let user = users::insert_user(
        &state.pool,
        uid,
        app.id,
        &body.email,
        Some(&hash),
        body.name.as_deref(),
    )
    .await
    .map_err(|_| ApiError::Internal)?;
    let (session, token, csrf) = issue_session(&state, user.id, &headers).await?;
    audit::log_event(
        &state.pool,
        Some(app.id),
        Some(user.id),
        "signup",
        serde_json::json!({}),
        headers
            .get("x-forwarded-for")
            .and_then(|v| v.to_str().ok())
            .or_else(|| headers.get("x-real-ip").and_then(|v| v.to_str().ok())),
        headers.get(header::USER_AGENT).and_then(|v| v.to_str().ok()),
    )
    .await
    .map_err(|_| ApiError::Internal)?;
    let mut res = (StatusCode::CREATED, Json(SignupResponse { session })).into_response();
    append_session_csrf_cookies(&mut res, &token, &csrf, state.config.cookie_secure)?;
    Ok(res)
}

pub fn router() -> Router<AppState> {
    Router::new().route("/signup", post(signup))
}
