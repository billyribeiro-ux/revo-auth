use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::routing::post;
use axum::{Extension, Json, Router};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use chrono::Duration;
use garde::Validate;
use serde::Deserialize;
use uuid::Uuid;

use crate::crypto::password::{self, verify_start};
use crate::crypto::tokens::{hash_session_token, random_token_32, token_b64url};
use crate::db::{audit, users, verification};
use crate::error::ApiError;
use crate::middleware::tenant::Tenant;
use crate::state::AppState;

#[derive(Deserialize, Validate)]
pub struct ResetRequestBody {
    #[garde(email)]
    pub email: String,
}

#[derive(Deserialize)]
pub struct ResetConfirmBody {
    pub token: String,
    pub password: String,
}

pub async fn reset_request(
    State(state): State<AppState>,
    Extension(Tenant(app)): Extension<Tenant>,
    headers: HeaderMap,
    Json(body): Json<ResetRequestBody>,
) -> Result<StatusCode, ApiError> {
    body.validate().map_err(|e| ApiError::Validation(e.to_string()))?;
    let start = verify_start();
    let user = users::find_by_email(&state.pool, app.id, &body.email)
        .await
        .map_err(|_| ApiError::Internal)?;
    if let Some(u) = user {
        let raw = random_token_32().map_err(|_| ApiError::Internal)?;
        let tok = token_b64url(&raw);
        let th = hash_session_token(&raw);
        let vid = Uuid::now_v7();
        verification::insert_token(
            &state.pool,
            vid,
            u.id,
            "password_reset",
            &th,
            chrono::Utc::now() + Duration::hours(1),
        )
        .await
        .map_err(|_| ApiError::Internal)?;
        let url = format!("https://example.invalid/reset?token={}", urlencoding::encode(&tok));
        crate::email::send_password_reset(state.mail.as_ref(), &state.config, &body.email, &url)
            .await
            .map_err(|_| ApiError::Internal)?;
    }
    password::constant_time_delay_miss().await;
    password::pad_verify_elapsed(start).await;
    audit::log_event(
        &state.pool,
        Some(app.id),
        None,
        "password-reset-request",
        serde_json::json!({}),
        headers
            .get("x-forwarded-for")
            .and_then(|v| v.to_str().ok())
            .or_else(|| headers.get("x-real-ip").and_then(|v| v.to_str().ok())),
        headers.get(axum::http::header::USER_AGENT).and_then(|v| v.to_str().ok()),
    )
    .await
    .ok();
    Ok(StatusCode::ACCEPTED)
}

pub async fn reset_confirm(
    State(state): State<AppState>,
    Extension(Tenant(app)): Extension<Tenant>,
    headers: HeaderMap,
    Json(body): Json<ResetConfirmBody>,
) -> Result<StatusCode, ApiError> {
    password::validate_password_strength(&body.password)?;
    let bytes = URL_SAFE_NO_PAD
        .decode(body.token.as_bytes())
        .map_err(|_| ApiError::BadRequest("invalid token".into()))?;
    let th = hash_session_token(&bytes);
    let row = verification::take_by_hash(&state.pool, "password_reset", &th)
        .await
        .map_err(|_| ApiError::Internal)?
        .ok_or(ApiError::BadRequest("invalid token".into()))?;
    let user = users::get_by_id(&state.pool, row.user_id)
        .await
        .map_err(|_| ApiError::Internal)?
        .ok_or(ApiError::BadRequest("invalid token".into()))?;
    if user.app_id != app.id {
        return Err(ApiError::Forbidden);
    }
    let hash = password::hash_password(&body.password)?;
    users::set_password_hash(&state.pool, user.id, &hash).await.map_err(|_| ApiError::Internal)?;
    audit::log_event(
        &state.pool,
        Some(app.id),
        Some(user.id),
        "password-change",
        serde_json::json!({}),
        headers
            .get("x-forwarded-for")
            .and_then(|v| v.to_str().ok())
            .or_else(|| headers.get("x-real-ip").and_then(|v| v.to_str().ok())),
        headers.get(axum::http::header::USER_AGENT).and_then(|v| v.to_str().ok()),
    )
    .await
    .map_err(|_| ApiError::Internal)?;
    Ok(StatusCode::NO_CONTENT)
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/password/reset/request", post(reset_request))
        .route("/password/reset/confirm", post(reset_confirm))
}
