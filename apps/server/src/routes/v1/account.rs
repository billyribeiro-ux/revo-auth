use axum::extract::{Path, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::routing::{delete, post};
use axum::{Extension, Json, Router};
use chrono::Duration;
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::crypto::tokens::{random_token_32, token_b64url};
use crate::db::{accounts, audit};
use crate::error::ApiError;
use crate::middleware::auth::load_session_user;
use crate::middleware::tenant::Tenant;
use crate::providers::{Provider, ProviderCreds};
use crate::state::AppState;

/// Short-lived (15min) JWT issued when starting an account-link flow.
/// Verified by the OAuth callback handler to convert a "new signup" into a
/// "link to existing session user".
#[derive(Debug, Serialize, Deserialize)]
struct LinkClaims {
    sub: Uuid,
    app: Uuid,
    provider: String,
    nonce: String,
    purpose: String,
    iss: String,
    aud: String,
    exp: i64,
    iat: i64,
}

fn ip_from_headers(headers: &HeaderMap) -> Option<&str> {
    headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .or_else(|| headers.get("x-real-ip").and_then(|v| v.to_str().ok()))
}

#[derive(Serialize)]
pub struct LinkResponse {
    link_token: String,
    authorize_url: String,
    state: String,
    nonce: String,
    expires_in: i64,
}

pub async fn link(
    State(state): State<AppState>,
    Extension(Tenant(app)): Extension<Tenant>,
    headers: HeaderMap,
    Path(provider): Path<String>,
) -> Result<Json<LinkResponse>, ApiError> {
    let (_sess, user) = load_session_user(&state, &headers).await?.ok_or(ApiError::Unauthorized)?;
    if user.app_id != app.id {
        return Err(ApiError::Forbidden);
    }

    let prov = Provider::parse(&provider).ok_or(ApiError::NotFound)?;
    if !prov.enabled() {
        return Err(ApiError::NotFound);
    }
    let creds = ProviderCreds::from_app(&app, prov.id())?;

    // Short-lived link token — signed ES256 JWT, `purpose: "oauth_link"`.
    let nonce_bytes = random_token_32().map_err(|_| ApiError::Internal)?;
    let nonce = token_b64url(&nonce_bytes);
    let now = chrono::Utc::now();
    let exp_secs = 15i64 * 60;
    let claims = LinkClaims {
        sub: user.id,
        app: app.id,
        provider: prov.id().to_string(),
        nonce: nonce.clone(),
        purpose: "oauth_link".to_string(),
        iss: state.config.jwt_issuer.clone(),
        aud: app.id.to_string(),
        exp: (now + Duration::seconds(exp_secs)).timestamp(),
        iat: now.timestamp(),
    };
    let mut header = Header::new(Algorithm::ES256);
    header.typ = Some("JWT".into());
    let key = EncodingKey::from_ec_pem(state.config.jwt_es256_private_pem.as_bytes())
        .map_err(|_| ApiError::Internal)?;
    let link_token = encode(&header, &claims, &key).map_err(|_| ApiError::Internal)?;

    // Redis pending-link intent — keyed by (user, nonce). Callback handler
    // checks this before creating a brand-new user.
    let key = format!("revo:link:{}:{}", user.id, nonce);
    let val = serde_json::json!({
        "user_id": user.id,
        "app_id": app.id,
        "provider": prov.id(),
    })
    .to_string();
    state.cache_set(&key, &val, exp_secs as u64).await.map_err(|_| ApiError::Internal)?;

    // Build an authorize URL that binds our state to (nonce) so the callback
    // can look it up. We use the same PKCE/state pattern as normal signin —
    // the callback is expected to verify state separately.
    let state_param = format!("link:{}:{}", user.id, nonce);
    let challenge = token_b64url(&random_token_32().map_err(|_| ApiError::Internal)?);
    let authorize_url = prov.authorize_url(&creds, &state_param, &challenge);

    audit::log_event(
        &state.pool,
        Some(app.id),
        Some(user.id),
        "account-link",
        serde_json::json!({ "provider": prov.id(), "stage": "begin" }),
        ip_from_headers(&headers),
        headers.get(header::USER_AGENT).and_then(|v| v.to_str().ok()),
    )
    .await
    .ok();

    Ok(Json(LinkResponse {
        link_token,
        authorize_url,
        state: state_param,
        nonce,
        expires_in: exp_secs,
    }))
}

pub async fn unlink(
    State(state): State<AppState>,
    Extension(Tenant(app)): Extension<Tenant>,
    headers: HeaderMap,
    Path(provider): Path<String>,
) -> Result<StatusCode, ApiError> {
    let (_sess, user) = load_session_user(&state, &headers).await?.ok_or(ApiError::Unauthorized)?;
    if user.app_id != app.id {
        return Err(ApiError::Forbidden);
    }
    let prov = Provider::parse(&provider).ok_or(ApiError::NotFound)?;

    // Verify the account exists for this user+provider.
    let acct = accounts::find_for_user_provider(&state.pool, user.id, prov.id())
        .await
        .map_err(|_| ApiError::Internal)?
        .ok_or(ApiError::NotFound)?;

    // Require at least one other auth method.
    let linked =
        accounts::count_for_user(&state.pool, user.id).await.map_err(|_| ApiError::Internal)?;
    let has_password = user.password_hash.is_some();
    // `linked` includes this provider — so "other linked" = linked - 1.
    if !has_password && linked <= 1 {
        // 409 per spec; client distinguishes via status + stable string code.
        return Err(ApiError::Conflict);
    }

    let removed = accounts::delete_for_user_provider(&state.pool, user.id, prov.id())
        .await
        .map_err(|_| ApiError::Internal)?;
    if removed == 0 {
        return Err(ApiError::NotFound);
    }

    audit::log_event(
        &state.pool,
        Some(app.id),
        Some(user.id),
        "account-unlink",
        serde_json::json!({ "provider": prov.id(), "account_id": acct.id }),
        ip_from_headers(&headers),
        headers.get(header::USER_AGENT).and_then(|v| v.to_str().ok()),
    )
    .await
    .ok();

    Ok(StatusCode::NO_CONTENT)
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/account/link/{provider}", post(link))
        .route("/account/{provider}", delete(unlink))
}
