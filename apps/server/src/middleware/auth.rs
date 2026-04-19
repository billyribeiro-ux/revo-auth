use axum::http::{header, HeaderMap};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};

use crate::crypto::tokens::hash_session_token;
use crate::db::{self, sessions, users};
use crate::error::ApiError;
use crate::state::AppState;

pub const SESSION_COOKIE: &str = "__Host-revoauth.session";
pub const CSRF_COOKIE: &str = "__Host-revoauth.csrf";

pub fn extract_session_token(headers: &HeaderMap) -> Option<String> {
    if let Some(auth) = headers.get(header::AUTHORIZATION).and_then(|v| v.to_str().ok()) {
        let rest = auth.strip_prefix("Bearer ")?;
        return Some(rest.to_string());
    }
    let cookie = headers.get(header::COOKIE)?.to_str().ok()?;
    for part in cookie.split(';') {
        let part = part.trim();
        if let Some(v) = part.strip_prefix(&format!("{SESSION_COOKIE}=")) {
            return Some(v.to_string());
        }
    }
    None
}

pub fn extract_csrf_token(headers: &HeaderMap) -> Option<String> {
    let cookie = headers.get(header::COOKIE)?.to_str().ok()?;
    for part in cookie.split(';') {
        let part = part.trim();
        if let Some(v) = part.strip_prefix(&format!("{CSRF_COOKIE}=")) {
            return Some(v.to_string());
        }
    }
    None
}

pub async fn load_session_user(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<Option<(db::SessionRow, db::UserRow)>, ApiError> {
    let Some(tok_str) = extract_session_token(headers) else {
        return Ok(None);
    };
    let bytes = URL_SAFE_NO_PAD.decode(tok_str.as_bytes()).map_err(|_| ApiError::Unauthorized)?;
    let th = hash_session_token(&bytes);
    let sess =
        sessions::find_by_token_hash(&state.pool, &th).await.map_err(|_| ApiError::Internal)?;
    let Some(sess) = sess else {
        return Ok(None);
    };
    if state.is_session_revoked(sess.id).await.map_err(|_| ApiError::Internal)? {
        return Ok(None);
    }
    if sess.expires_at < chrono::Utc::now() {
        return Ok(None);
    }
    let user = users::get_by_id(&state.pool, sess.user_id)
        .await
        .map_err(|_| ApiError::Internal)?
        .ok_or(ApiError::Unauthorized)?;
    if user.banned_at.is_some() {
        return Err(ApiError::Forbidden);
    }
    sessions::touch(&state.pool, sess.id).await.map_err(|_| ApiError::Internal)?;
    Ok(Some((sess, user)))
}
