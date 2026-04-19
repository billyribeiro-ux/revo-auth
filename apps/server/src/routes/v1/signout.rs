use axum::extract::State;
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::routing::post;
use axum::{Extension, Router};

use crate::db::{audit, sessions};
use crate::error::ApiError;
use crate::middleware::auth::{self, load_session_user};
use crate::middleware::tenant::Tenant;
use crate::routes::v1::session::clear_csrf_cookie;
use crate::state::AppState;

pub async fn signout(
    State(state): State<AppState>,
    Extension(Tenant(app)): Extension<Tenant>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ApiError> {
    let Some((sess, user)) = load_session_user(&state, &headers).await? else {
        return Err(ApiError::Unauthorized);
    };
    if user.app_id != app.id {
        return Err(ApiError::Forbidden);
    }
    sessions::revoke(&state.pool, sess.id).await.map_err(|_| ApiError::Internal)?;
    let ttl = (sess.expires_at - chrono::Utc::now()).num_seconds().max(0) as u64;
    let _ = state.mark_session_revoked(sess.id, ttl).await;
    audit::log_event(
        &state.pool,
        Some(app.id),
        Some(user.id),
        "signout",
        serde_json::json!({ "session_id": sess.id }),
        headers
            .get("x-forwarded-for")
            .and_then(|v| v.to_str().ok())
            .or_else(|| headers.get("x-real-ip").and_then(|v| v.to_str().ok())),
        headers.get(header::USER_AGENT).and_then(|v| v.to_str().ok()),
    )
    .await
    .map_err(|_| ApiError::Internal)?;
    let clear_sess =
        format!("{}=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0; Secure", auth::SESSION_COOKIE);
    let clear_csrf = clear_csrf_cookie(state.config.cookie_secure);
    let mut res = StatusCode::NO_CONTENT.into_response();
    res.headers_mut().append(
        header::SET_COOKIE,
        axum::http::HeaderValue::from_str(&clear_sess).map_err(|_| ApiError::Internal)?,
    );
    res.headers_mut().append(
        header::SET_COOKIE,
        axum::http::HeaderValue::from_str(&clear_csrf).map_err(|_| ApiError::Internal)?,
    );
    Ok(res)
}

pub fn router() -> Router<AppState> {
    Router::new().route("/signout", post(signout))
}
