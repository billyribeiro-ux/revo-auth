use axum::extract::State;
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::{Extension, Json, Router};
use garde::Validate;
use serde::Deserialize;

use crate::crypto::password::{self, verify_start};
use crate::db::{audit, users};
use crate::error::ApiError;
use crate::middleware::tenant::Tenant;
use crate::routes::v1::session::{append_session_csrf_cookies, issue_session};
use crate::state::AppState;

#[derive(Deserialize, Validate)]
pub struct SigninBody {
    #[garde(email)]
    pub email: String,
    #[garde(length(min = 1))]
    pub password: String,
}

pub async fn signin(
    State(state): State<AppState>,
    Extension(Tenant(app)): Extension<Tenant>,
    headers: HeaderMap,
    Json(body): Json<SigninBody>,
) -> Result<Response, ApiError> {
    body.validate().map_err(|e| ApiError::Validation(e.to_string()))?;
    let start = verify_start();
    let user = users::find_by_email(&state.pool, app.id, &body.email)
        .await
        .map_err(|_| ApiError::Internal)?;
    let ok = match &user {
        Some(u) if u.banned_at.is_none() => match &u.password_hash {
            Some(ph) => password::verify_password(&body.password, ph).unwrap_or(false),
            None => false,
        },
        _ => false,
    };
    if !ok {
        password::constant_time_delay_miss().await;
        password::pad_verify_elapsed(start).await;
        audit::log_event(
            &state.pool,
            Some(app.id),
            user.as_ref().map(|u| u.id),
            "failed-signin",
            serde_json::json!({ "email": body.email }),
            headers
                .get("x-forwarded-for")
                .and_then(|v| v.to_str().ok())
                .or_else(|| headers.get("x-real-ip").and_then(|v| v.to_str().ok())),
            headers.get(header::USER_AGENT).and_then(|v| v.to_str().ok()),
        )
        .await
        .ok();
        return Err(ApiError::InvalidCredentials);
    }
    let user = user.ok_or(ApiError::Internal)?;
    if let Some(ref ph) = user.password_hash {
        if password::needs_rehash(ph) {
            let new_hash = password::hash_password(&body.password)?;
            users::set_password_hash(&state.pool, user.id, &new_hash)
                .await
                .map_err(|_| ApiError::Internal)?;
        }
    }
    password::pad_verify_elapsed(start).await;
    let (session, token, csrf) = issue_session(&state, user.id, &headers).await?;
    audit::log_event(
        &state.pool,
        Some(app.id),
        Some(user.id),
        "signin",
        serde_json::json!({}),
        headers
            .get("x-forwarded-for")
            .and_then(|v| v.to_str().ok())
            .or_else(|| headers.get("x-real-ip").and_then(|v| v.to_str().ok())),
        headers.get(header::USER_AGENT).and_then(|v| v.to_str().ok()),
    )
    .await
    .map_err(|_| ApiError::Internal)?;
    let mut res = (StatusCode::OK, Json(serde_json::json!({ "session": session }))).into_response();
    append_session_csrf_cookies(&mut res, &token, &csrf, state.config.cookie_secure)?;
    Ok(res)
}

pub fn router() -> Router<AppState> {
    Router::new().route("/signin", post(signin))
}
