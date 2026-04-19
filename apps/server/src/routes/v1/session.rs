use axum::extract::State;
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, post};
use axum::{Extension, Json, Router};
use chrono::Duration;
use serde::Serialize;
use uuid::Uuid;

use crate::crypto::tokens::{hash_session_token, random_token_32, token_b64url};
use crate::db::{sessions, users};
use crate::db::{SessionRow, UserRow};
use crate::error::ApiError;
use crate::middleware::auth::{self, load_session_user};
use crate::middleware::tenant::Tenant;
use crate::state::AppState;

#[derive(Serialize)]
pub struct UserOut {
    pub id: Uuid,
    pub email: Option<String>,
    pub email_verified: bool,
    pub name: Option<String>,
    pub image: Option<String>,
    pub custom_fields: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize)]
pub struct SessionOut {
    pub id: Uuid,
    pub user_id: Uuid,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub user: UserOut,
}

impl From<&UserRow> for UserOut {
    fn from(u: &UserRow) -> Self {
        Self {
            id: u.id,
            email: u.email.clone(),
            email_verified: u.email_verified_at.is_some(),
            name: u.name.clone(),
            image: u.image_url.clone(),
            custom_fields: u.custom_fields.clone(),
            created_at: u.created_at,
        }
    }
}

pub fn session_out(sess: &SessionRow, user: &UserRow) -> SessionOut {
    SessionOut {
        id: sess.id,
        user_id: sess.user_id,
        expires_at: sess.expires_at,
        user: UserOut::from(user),
    }
}

pub fn session_cookie_value(token: &str, max_age_secs: i64, _secure: bool) -> String {
    // __Host- prefix mandates Path=/, no Domain, and Secure — always.
    format!(
        "{}={token}; Path=/; HttpOnly; SameSite=Lax; Max-Age={max_age_secs}; Secure",
        auth::SESSION_COOKIE
    )
}

pub fn csrf_cookie_value(token: &str, max_age_secs: i64, _secure: bool) -> String {
    // __Host- prefix mandates Path=/, no Domain, and Secure — always.
    format!("{}={token}; Path=/; SameSite=Lax; Max-Age={max_age_secs}; Secure", auth::CSRF_COOKIE)
}

pub fn clear_csrf_cookie(_secure: bool) -> String {
    format!("{}=; Path=/; SameSite=Lax; Max-Age=0; Secure", auth::CSRF_COOKIE)
}

pub async fn issue_session(
    state: &AppState,
    user_id: Uuid,
    headers: &HeaderMap,
) -> Result<(SessionOut, String, String), ApiError> {
    let raw = random_token_32().map_err(|_| ApiError::Internal)?;
    let token_str = token_b64url(&raw);
    let th = hash_session_token(&raw);
    let sid = Uuid::now_v7();
    let ua = headers.get(header::USER_AGENT).and_then(|v| v.to_str().ok());
    let ip = headers
        .get("x-forwarded-for")
        .or_else(|| headers.get("x-real-ip"))
        .and_then(|v| v.to_str().ok());
    let exp = chrono::Utc::now() + Duration::days(14);
    let sess = sessions::insert_session(&state.pool, sid, user_id, &th, ua, ip, exp)
        .await
        .map_err(|_| ApiError::Internal)?;
    let user = users::get_by_id(&state.pool, user_id)
        .await
        .map_err(|_| ApiError::Internal)?
        .ok_or(ApiError::Internal)?;
    let csrf_raw = random_token_32().map_err(|_| ApiError::Internal)?;
    let csrf = token_b64url(&csrf_raw);
    Ok((session_out(&sess, &user), token_str, csrf))
}

pub fn append_session_csrf_cookies(
    res: &mut Response,
    token: &str,
    csrf: &str,
    secure: bool,
) -> Result<(), ApiError> {
    let max_age = 14 * 24 * 3600i64;
    let c1 = session_cookie_value(token, max_age, secure);
    let c2 = csrf_cookie_value(csrf, max_age, secure);
    res.headers_mut().append(
        header::SET_COOKIE,
        axum::http::HeaderValue::from_str(&c1).map_err(|_| ApiError::Internal)?,
    );
    res.headers_mut().append(
        header::SET_COOKIE,
        axum::http::HeaderValue::from_str(&c2).map_err(|_| ApiError::Internal)?,
    );
    Ok(())
}

pub async fn get_session(
    State(state): State<AppState>,
    Extension(Tenant(app)): Extension<Tenant>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, ApiError> {
    let Some((sess, user)) = load_session_user(&state, &headers).await? else {
        return Err(ApiError::Unauthorized);
    };
    if user.app_id != app.id {
        return Err(ApiError::Forbidden);
    }
    Ok(Json(serde_json::json!({ "session": session_out(&sess, &user) })))
}

pub async fn refresh_session(
    State(state): State<AppState>,
    Extension(Tenant(app)): Extension<Tenant>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let Some((old, user)) = load_session_user(&state, &headers).await? else {
        return Err(ApiError::Unauthorized);
    };
    if user.app_id != app.id {
        return Err(ApiError::Forbidden);
    }
    sessions::revoke(&state.pool, old.id).await.map_err(|_| ApiError::Internal)?;
    let ttl = (old.expires_at - chrono::Utc::now()).num_seconds().max(0) as u64;
    let _ = state.mark_session_revoked(old.id, ttl).await;
    let (session, token, csrf) = issue_session(&state, user.id, &headers).await?;
    let mut res = (StatusCode::OK, Json(serde_json::json!({ "session": session }))).into_response();
    append_session_csrf_cookies(&mut res, &token, &csrf, state.config.cookie_secure)?;
    Ok(res)
}

pub async fn list_sessions(
    State(state): State<AppState>,
    Extension(Tenant(app)): Extension<Tenant>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, ApiError> {
    let Some((_sess, user)) = load_session_user(&state, &headers).await? else {
        return Err(ApiError::Unauthorized);
    };
    if user.app_id != app.id {
        return Err(ApiError::Forbidden);
    }
    let list =
        sessions::list_for_user(&state.pool, user.id).await.map_err(|_| ApiError::Internal)?;
    let out: Vec<_> = list
        .iter()
        .map(|s| {
            serde_json::json!({
                "id": s.id,
                "expires_at": s.expires_at,
                "created_at": s.created_at,
                "last_used_at": s.last_used_at,
            })
        })
        .collect();
    Ok(Json(serde_json::json!({ "sessions": out })))
}

pub async fn delete_session(
    State(state): State<AppState>,
    Extension(Tenant(app)): Extension<Tenant>,
    headers: HeaderMap,
    axum::extract::Path(sid): axum::extract::Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let Some((cur, user)) = load_session_user(&state, &headers).await? else {
        return Err(ApiError::Unauthorized);
    };
    if user.app_id != app.id {
        return Err(ApiError::Forbidden);
    }
    let rows = sqlx::query("update sessions set revoked_at = now() where id = $1 and user_id = $2 and revoked_at is null")
		.bind(sid)
		.bind(user.id)
		.execute(&state.pool)
		.await
		.map_err(|_| ApiError::Internal)?;
    if rows.rows_affected() == 0 {
        return Err(ApiError::NotFound);
    }
    let ttl = (cur.expires_at - chrono::Utc::now()).num_seconds().max(0) as u64;
    let _ = state.mark_session_revoked(sid, ttl).await;
    Ok(StatusCode::NO_CONTENT)
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/session", get(get_session))
        .route("/session/refresh", post(refresh_session))
        .route("/sessions", get(list_sessions))
        .route("/sessions/{id}", delete(delete_session))
}
