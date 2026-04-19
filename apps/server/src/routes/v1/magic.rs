//! Magic-link request + verify.
//!
//! `request` is enumeration-safe: identical 202 response whether or not the
//! email exists, with 50-150ms jitter on misses. Pair-bucket rate limit
//! enforced per `(ip, email)`.

use axum::extract::{Query, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Extension, Json, Router};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use chrono::Duration;
use garde::Validate;
use serde::Deserialize;
use uuid::Uuid;

use crate::crypto::password::constant_time_delay_miss;
use crate::crypto::tokens::{hash_session_token, random_token_32, token_b64url};
use crate::db::{audit, magic_links, users};
use crate::error::ApiError;
use crate::middleware::rate_limit::pair_bucket_check;
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

#[derive(Deserialize, Validate)]
pub struct RequestBody {
    #[garde(email)]
    pub email: String,
    #[garde(skip)]
    pub redirect_after: Option<String>,
}

pub async fn request(
    State(state): State<AppState>,
    Extension(Tenant(app)): Extension<Tenant>,
    headers: HeaderMap,
    Json(body): Json<RequestBody>,
) -> Result<StatusCode, ApiError> {
    body.validate().map_err(|e| ApiError::Validation(e.to_string()))?;
    let ip = client_ip(&headers);
    pair_bucket_check(&state, "magic", ip, &body.email, 5, 3600).await?;

    let user = users::find_by_email(&state.pool, app.id, &body.email)
        .await
        .map_err(|_| ApiError::Internal)?;

    if let Some(u) = user.as_ref() {
        let raw = random_token_32().map_err(|_| ApiError::Internal)?;
        let tok = token_b64url(&raw);
        let th = hash_session_token(&raw);
        let mid = Uuid::now_v7();
        let expires_at = chrono::Utc::now() + Duration::minutes(10);
        magic_links::insert(&state.pool, mid, Some(u.id), &body.email, app.id, &th, expires_at)
            .await
            .map_err(|_| ApiError::Internal)?;

        let base = app.origins.first().cloned().unwrap_or_else(|| "https://example.invalid".to_string());
        let redirect = body.redirect_after.as_deref().unwrap_or("");
        let url = format!(
            "{base}/v1/magic/verify?token={}&redirect_after={}",
            urlencoding::encode(&tok),
            urlencoding::encode(redirect),
        );
        crate::email::send_magic_link(state.mail.as_ref(), &state.config, &body.email, &url)
            .await
            .map_err(|_| ApiError::Internal)?;

        audit::log_event(
            &state.pool,
            Some(app.id),
            Some(u.id),
            "magic-link-request",
            serde_json::json!({}),
            ip,
            client_ua(&headers),
        )
        .await
        .ok();
    } else {
        constant_time_delay_miss().await;
    }

    Ok(StatusCode::ACCEPTED)
}

#[derive(Deserialize)]
pub struct VerifyQuery {
    pub token: String,
    #[serde(default)]
    pub redirect_after: Option<String>,
}

pub async fn verify(
    State(state): State<AppState>,
    Extension(Tenant(app)): Extension<Tenant>,
    headers: HeaderMap,
    Query(q): Query<VerifyQuery>,
) -> Result<Response, ApiError> {
    let bytes = URL_SAFE_NO_PAD
        .decode(q.token.as_bytes())
        .map_err(|_| ApiError::BadRequest("invalid token".into()))?;
    let th = hash_session_token(&bytes);
    let row = magic_links::take_by_hash(&state.pool, app.id, &th)
        .await
        .map_err(|_| ApiError::Internal)?
        .ok_or(ApiError::BadRequest("invalid or expired token".into()))?;
    // Verify a user actually existed (row.user_id may be None if the row was
    // created for a not-yet-existent user; reject those).
    let user_id = row.user_id.ok_or(ApiError::BadRequest("invalid token".into()))?;
    let user = users::get_by_id(&state.pool, user_id)
        .await
        .map_err(|_| ApiError::Internal)?
        .ok_or(ApiError::BadRequest("invalid token".into()))?;
    if user.app_id != app.id {
        return Err(ApiError::Forbidden);
    }

    let (_session, token, csrf) = issue_session(&state, user.id, &headers).await?;
    audit::log_event(
        &state.pool,
        Some(app.id),
        Some(user.id),
        "signin",
        serde_json::json!({ "method": "magic-link" }),
        client_ip(&headers),
        client_ua(&headers),
    )
    .await
    .map_err(|_| ApiError::Internal)?;

    let redirect_to = q
        .redirect_after
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| app.origins.first().cloned().unwrap_or_else(|| "/".to_string()));

    let mut res = StatusCode::FOUND.into_response();
    res.headers_mut().insert(
        header::LOCATION,
        axum::http::HeaderValue::from_str(&redirect_to).map_err(|_| ApiError::Internal)?,
    );
    append_session_csrf_cookies(&mut res, &token, &csrf, state.config.cookie_secure)?;
    Ok(res)
}

pub fn router() -> Router<AppState> {
    Router::new().route("/magic/request", post(request)).route("/magic/verify", get(verify))
}
