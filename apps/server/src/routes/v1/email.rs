//! Email verification: request (session-scoped) + confirm.

use axum::extract::State;
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::{Extension, Json, Router};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use chrono::Duration;
use garde::Validate;
use serde::Deserialize;
use uuid::Uuid;

use crate::crypto::tokens::{hash_session_token, random_token_32, token_b64url};
use crate::db::{audit, sessions, users, verification};
use crate::error::ApiError;
use crate::middleware::auth::load_session_user;
use crate::middleware::tenant::Tenant;
use crate::routes::v1::session::{append_session_csrf_cookies, issue_session};
use crate::state::AppState;

fn client_ip(headers: &HeaderMap) -> Option<&str> {
    headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .or_else(|| headers.get("x-real-ip").and_then(|v| v.to_str().ok()))
}
fn client_ua(headers: &HeaderMap) -> Option<&str> {
    headers.get(header::USER_AGENT).and_then(|v| v.to_str().ok())
}

pub async fn verify_request(
    State(state): State<AppState>,
    Extension(Tenant(app)): Extension<Tenant>,
    headers: HeaderMap,
) -> Result<StatusCode, ApiError> {
    let Some((_sess, user)) = load_session_user(&state, &headers).await? else {
        return Err(ApiError::Unauthorized);
    };
    if user.app_id != app.id {
        return Err(ApiError::Forbidden);
    }
    let email = user.email.clone().ok_or(ApiError::BadRequest("user has no email".into()))?;
    if user.email_verified_at.is_some() {
        return Ok(StatusCode::NO_CONTENT);
    }

    let raw = random_token_32().map_err(|_| ApiError::Internal)?;
    let tok = token_b64url(&raw);
    let th = hash_session_token(&raw);
    let vid = Uuid::now_v7();
    verification::insert_token(
        &state.pool,
        vid,
        user.id,
        "email_verify",
        &th,
        chrono::Utc::now() + Duration::hours(24),
    )
    .await
    .map_err(|_| ApiError::Internal)?;

    let base = app.origins.first().cloned().unwrap_or_else(|| "https://example.invalid".to_string());
    let url = format!("{base}/verify-email?token={}", urlencoding::encode(&tok));
    crate::email::send_verification(state.mail.as_ref(), &state.config, &email, &url)
        .await
        .map_err(|_| ApiError::Internal)?;

    audit::log_event(
        &state.pool,
        Some(app.id),
        Some(user.id),
        "email-verify-request",
        serde_json::json!({}),
        client_ip(&headers),
        client_ua(&headers),
    )
    .await
    .ok();

    Ok(StatusCode::ACCEPTED)
}

#[derive(Deserialize, Validate)]
pub struct ConfirmBody {
    #[garde(length(min = 1))]
    pub token: String,
}

pub async fn verify_confirm(
    State(state): State<AppState>,
    Extension(Tenant(app)): Extension<Tenant>,
    headers: HeaderMap,
    Json(body): Json<ConfirmBody>,
) -> Result<Response, ApiError> {
    body.validate().map_err(|e| ApiError::Validation(e.to_string()))?;
    let bytes = URL_SAFE_NO_PAD
        .decode(body.token.as_bytes())
        .map_err(|_| ApiError::BadRequest("invalid token".into()))?;
    let th = hash_session_token(&bytes);
    let row = verification::take_by_hash(&state.pool, "email_verify", &th)
        .await
        .map_err(|_| ApiError::Internal)?
        .ok_or(ApiError::BadRequest("invalid or expired token".into()))?;
    let user = users::get_by_id(&state.pool, row.user_id)
        .await
        .map_err(|_| ApiError::Internal)?
        .ok_or(ApiError::BadRequest("invalid token".into()))?;
    if user.app_id != app.id {
        return Err(ApiError::Forbidden);
    }
    users::verify_email(&state.pool, user.id).await.map_err(|_| ApiError::Internal)?;

    audit::log_event(
        &state.pool,
        Some(app.id),
        Some(user.id),
        "email-verified",
        serde_json::json!({}),
        client_ip(&headers),
        client_ua(&headers),
    )
    .await
    .ok();

    // Session rotation on email verification.
    if let Some((cur, _)) = load_session_user(&state, &headers).await? {
        sessions::revoke(&state.pool, cur.id).await.map_err(|_| ApiError::Internal)?;
        let ttl = (cur.expires_at - chrono::Utc::now()).num_seconds().max(0) as u64;
        let _ = state.mark_session_revoked(cur.id, ttl).await;
        let (session, token, csrf) = issue_session(&state, user.id, &headers).await?;
        let mut res =
            (StatusCode::OK, Json(serde_json::json!({ "session": session }))).into_response();
        append_session_csrf_cookies(&mut res, &token, &csrf, state.config.cookie_secure)?;
        return Ok(res);
    }

    Ok(StatusCode::NO_CONTENT.into_response())
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/email/verify/request", post(verify_request))
        .route("/email/verify/confirm", post(verify_confirm))
}
