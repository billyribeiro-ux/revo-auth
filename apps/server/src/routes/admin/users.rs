use axum::extract::{Path, Query, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use uuid::Uuid;

use crate::db::{audit, sessions, users};
use crate::error::ApiError;
use crate::routes::v1::session::{append_session_csrf_cookies, issue_session};
use crate::state::AppState;

use super::apps::require_master;

const DEFAULT_LIMIT: i64 = 50;
const MAX_LIMIT: i64 = 200;

#[derive(Deserialize)]
pub struct ListQuery {
    pub app_id: Option<Uuid>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

fn ip_from_headers(headers: &HeaderMap) -> Option<&str> {
    headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .or_else(|| headers.get("x-real-ip").and_then(|v| v.to_str().ok()))
}

pub async fn list_users(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(q): Query<ListQuery>,
) -> Result<Json<serde_json::Value>, ApiError> {
    require_master(&headers, &state.config.master_key)?;
    let app_id = q.app_id.ok_or(ApiError::BadRequest("app_id required".into()))?;
    let limit = q.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
    let offset = q.offset.unwrap_or(0).max(0);
    let rows = users::list_by_app(&state.pool, app_id, limit, offset)
        .await
        .map_err(|_| ApiError::Internal)?;
    let out: Vec<_> = rows
        .iter()
        .map(|u| {
            serde_json::json!({
                "id": u.id,
                "app_id": u.app_id,
                "email": u.email,
                "email_verified": u.email_verified_at.is_some(),
                "name": u.name,
                "image": u.image_url,
                "banned_at": u.banned_at,
                "created_at": u.created_at,
            })
        })
        .collect();
    Ok(Json(serde_json::json!({ "users": out, "limit": limit, "offset": offset })))
}

async fn revoke_all_sessions(state: &AppState, user_id: Uuid) -> Result<(), ApiError> {
    let revoked = sessions::revoke_all_for_user(&state.pool, user_id)
        .await
        .map_err(|_| ApiError::Internal)?;
    for s in revoked {
        let ttl = (s.expires_at - chrono::Utc::now()).num_seconds().max(0) as u64;
        let _ = state.mark_session_revoked(s.id, ttl).await;
    }
    Ok(())
}

pub async fn ban_user(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(uid): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    require_master(&headers, &state.config.master_key)?;
    let u = users::get_by_id(&state.pool, uid)
        .await
        .map_err(|_| ApiError::Internal)?
        .ok_or(ApiError::NotFound)?;
    users::set_banned(&state.pool, u.id, true).await.map_err(|_| ApiError::Internal)?;
    revoke_all_sessions(&state, u.id).await?;
    audit::log_event(
        &state.pool,
        Some(u.app_id),
        Some(u.id),
        "admin-ban",
        serde_json::json!({}),
        ip_from_headers(&headers),
        headers.get(header::USER_AGENT).and_then(|v| v.to_str().ok()),
    )
    .await
    .ok();
    Ok(StatusCode::NO_CONTENT)
}

pub async fn unban_user(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(uid): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    require_master(&headers, &state.config.master_key)?;
    let u = users::get_by_id(&state.pool, uid)
        .await
        .map_err(|_| ApiError::Internal)?
        .ok_or(ApiError::NotFound)?;
    users::set_banned(&state.pool, u.id, false).await.map_err(|_| ApiError::Internal)?;
    audit::log_event(
        &state.pool,
        Some(u.app_id),
        Some(u.id),
        "admin-unban",
        serde_json::json!({}),
        ip_from_headers(&headers),
        headers.get(header::USER_AGENT).and_then(|v| v.to_str().ok()),
    )
    .await
    .ok();
    Ok(StatusCode::NO_CONTENT)
}

pub async fn impersonate(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(uid): Path<Uuid>,
) -> Result<Response, ApiError> {
    require_master(&headers, &state.config.master_key)?;
    let target = users::get_by_id(&state.pool, uid)
        .await
        .map_err(|_| ApiError::Internal)?
        .ok_or(ApiError::NotFound)?;
    if target.banned_at.is_some() {
        return Err(ApiError::Forbidden);
    }

    // Issue a session for the target user. Flag the meta as impersonation.
    let (session, token, csrf) = issue_session(&state, target.id, &headers).await?;

    audit::log_event(
        &state.pool,
        Some(target.app_id),
        Some(target.id),
        "impersonation-start",
        serde_json::json!({
            "impersonator": "master-key",
            "session_id": session.id,
        }),
        ip_from_headers(&headers),
        headers.get(header::USER_AGENT).and_then(|v| v.to_str().ok()),
    )
    .await
    .ok();

    let body = serde_json::json!({
        "session": session,
        "token": token,
        "meta": { "impersonator": "master-key" },
    });
    let mut res = (StatusCode::OK, Json(body)).into_response();
    append_session_csrf_cookies(&mut res, &token, &csrf, state.config.cookie_secure)?;
    Ok(res)
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/users", get(list_users))
        .route("/users/{id}/ban", post(ban_user))
        .route("/users/{id}/unban", post(unban_user))
        .route("/impersonate/{user_id}", post(impersonate))
}
