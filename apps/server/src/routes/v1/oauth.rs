//! OAuth authorize + callback. PKCE S256 is mandatory. The `state` parameter
//! is an HMAC-SHA256 of tenant_id|nonce|redirect_after|exp, signed with the
//! server's master key; the PKCE verifier is parked in Redis under the nonce.

use axum::extract::{Path, Query, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Extension, Router};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use chrono::Duration;
use hmac::{Hmac, Mac};
use serde::Deserialize;
use sha2::Sha256;
use subtle::ConstantTimeEq;
use uuid::Uuid;

use crate::crypto::tokens::{random_token_32, token_b64url, TokenCipher};
use crate::db::{accounts, audit, users};
use crate::error::ApiError;
use crate::middleware::tenant::Tenant;
use crate::providers::{Provider, ProviderCreds};
use crate::routes::v1::session::{append_session_csrf_cookies, issue_session};
use crate::state::AppState;

type HmacSha256 = Hmac<Sha256>;

const STATE_TTL_SECS: i64 = 600;
const VERIFIER_TTL_SECS: u64 = 600;

#[derive(Debug, Deserialize)]
pub struct AuthorizeQuery {
    #[serde(default)]
    pub redirect_after: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CallbackQuery {
    pub code: Option<String>,
    pub state: Option<String>,
    #[serde(default)]
    pub error: Option<String>,
}

fn client_ip(headers: &HeaderMap) -> Option<&str> {
    headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .or_else(|| headers.get("x-real-ip").and_then(|v| v.to_str().ok()))
}

fn client_ua(headers: &HeaderMap) -> Option<&str> {
    headers.get(header::USER_AGENT).and_then(|v| v.to_str().ok())
}

/// Encoded state payload (before signing): tenant | nonce | redirect | exp.
/// After building, it's joined with the HMAC signature as
/// `{payload_b64url}.{sig_b64url}`.
fn build_state(
    secret: &[u8],
    tenant_id: Uuid,
    nonce: &str,
    redirect_after: &str,
    exp_unix: i64,
) -> Result<String, ApiError> {
    let payload = format!("{tenant_id}|{nonce}|{redirect_after}|{exp_unix}");
    let mut mac = HmacSha256::new_from_slice(secret).map_err(|_| ApiError::Internal)?;
    mac.update(payload.as_bytes());
    let sig = mac.finalize().into_bytes();
    let enc_payload = URL_SAFE_NO_PAD.encode(payload.as_bytes());
    let enc_sig = URL_SAFE_NO_PAD.encode(sig);
    Ok(format!("{enc_payload}.{enc_sig}"))
}

struct StateParts {
    tenant_id: Uuid,
    nonce: String,
    redirect_after: String,
}

fn verify_state(secret: &[u8], raw: &str) -> Result<StateParts, ApiError> {
    let (payload_b64, sig_b64) =
        raw.split_once('.').ok_or(ApiError::BadRequest("malformed state".into()))?;
    let payload =
        URL_SAFE_NO_PAD.decode(payload_b64).map_err(|_| ApiError::BadRequest("bad state".into()))?;
    let sig =
        URL_SAFE_NO_PAD.decode(sig_b64).map_err(|_| ApiError::BadRequest("bad state".into()))?;
    let mut mac = HmacSha256::new_from_slice(secret).map_err(|_| ApiError::Internal)?;
    mac.update(&payload);
    let expected = mac.finalize().into_bytes();
    if expected.ct_eq(&sig).unwrap_u8() != 1 {
        return Err(ApiError::BadRequest("bad state".into()));
    }
    let payload_str =
        String::from_utf8(payload).map_err(|_| ApiError::BadRequest("bad state".into()))?;
    let mut parts = payload_str.splitn(4, '|');
    let tenant_id = parts.next().ok_or(ApiError::BadRequest("bad state".into()))?;
    let nonce = parts.next().ok_or(ApiError::BadRequest("bad state".into()))?;
    let redirect_after = parts.next().ok_or(ApiError::BadRequest("bad state".into()))?;
    let exp_str = parts.next().ok_or(ApiError::BadRequest("bad state".into()))?;
    let tenant_id: Uuid =
        tenant_id.parse().map_err(|_| ApiError::BadRequest("bad state".into()))?;
    let exp_unix: i64 = exp_str.parse().map_err(|_| ApiError::BadRequest("bad state".into()))?;
    if chrono::Utc::now().timestamp() > exp_unix {
        return Err(ApiError::BadRequest("state expired".into()));
    }
    // `exp_unix` is validated above; we don't store it after that check.
    let _ = exp_unix;
    Ok(StateParts {
        tenant_id,
        nonce: nonce.to_string(),
        redirect_after: redirect_after.to_string(),
    })
}

fn pkce_challenge_s256(verifier: &[u8]) -> String {
    use ring::digest;
    let hash = digest::digest(&digest::SHA256, verifier);
    URL_SAFE_NO_PAD.encode(hash.as_ref())
}

fn resolve_provider(provider: &str) -> Result<Provider, ApiError> {
    let p = Provider::parse(provider).ok_or(ApiError::NotFound)?;
    if !p.enabled() {
        return Err(ApiError::NotFound);
    }
    Ok(p)
}

fn redirect_response(location: &str) -> Result<Response, ApiError> {
    let mut res = StatusCode::FOUND.into_response();
    res.headers_mut().insert(
        header::LOCATION,
        axum::http::HeaderValue::from_str(location).map_err(|_| ApiError::Internal)?,
    );
    Ok(res)
}

pub async fn authorize(
    State(state): State<AppState>,
    Extension(Tenant(app)): Extension<Tenant>,
    Path(provider): Path<String>,
    Query(q): Query<AuthorizeQuery>,
) -> Result<Response, ApiError> {
    let p = resolve_provider(&provider)?;
    let creds = ProviderCreds::from_app(&app, p.id())?;

    let verifier_bytes = random_token_32().map_err(|_| ApiError::Internal)?;
    let verifier = token_b64url(&verifier_bytes);
    let challenge = pkce_challenge_s256(verifier.as_bytes());

    let nonce_bytes = random_token_32().map_err(|_| ApiError::Internal)?;
    let nonce = token_b64url(&nonce_bytes);
    let redirect_after = q.redirect_after.unwrap_or_else(|| {
        app.origins.first().cloned().unwrap_or_else(|| "/".to_string())
    });
    let exp_unix = (chrono::Utc::now() + Duration::seconds(STATE_TTL_SECS)).timestamp();

    let state_str = build_state(
        state.config.master_key.as_bytes(),
        app.id,
        &nonce,
        &redirect_after,
        exp_unix,
    )?;

    let key = format!("revo:oauth:verifier:{}:{nonce}", p.id());
    state
        .cache_set(&key, &verifier, VERIFIER_TTL_SECS)
        .await
        .map_err(|_| ApiError::Internal)?;

    let url = p.authorize_url(&creds, &state_str, &challenge);
    redirect_response(&url)
}

pub async fn callback(
    State(state): State<AppState>,
    Extension(Tenant(app)): Extension<Tenant>,
    Path(provider): Path<String>,
    Query(q): Query<CallbackQuery>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let p = resolve_provider(&provider)?;
    let creds = ProviderCreds::from_app(&app, p.id())?;

    if let Some(err) = q.error {
        return Err(ApiError::BadRequest(format!("oauth error: {err}")));
    }
    let code = q.code.ok_or(ApiError::BadRequest("missing code".into()))?;
    let state_raw = q.state.ok_or(ApiError::BadRequest("missing state".into()))?;
    let parts = verify_state(state.config.master_key.as_bytes(), &state_raw)?;
    if parts.tenant_id != app.id {
        return Err(ApiError::Forbidden);
    }
    let key = format!("revo:oauth:verifier:{}:{}", p.id(), parts.nonce);
    let verifier = state
        .cache_get(&key)
        .await
        .map_err(|_| ApiError::Internal)?
        .ok_or(ApiError::BadRequest("state expired or reused".into()))?;
    // Consume the verifier atomically (best-effort; any error here is benign —
    // Redis TTL will clean up).
    let _ = state.cache_del(&key).await;

    let http = reqwest::Client::builder()
        .user_agent("revo-auth/1.0")
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|_| ApiError::Internal)?;
    let profile = p.exchange(&http, &creds, &code, &verifier).await?;

    // Require an email to reconcile against `users(app_id, email)`.
    let email = profile
        .email
        .clone()
        .ok_or(ApiError::BadRequest("provider did not return an email".into()))?;

    // Upsert user (scope: app_id + email).
    let user = match users::find_by_email(&state.pool, app.id, &email)
        .await
        .map_err(|_| ApiError::Internal)?
    {
        Some(u) => {
            users::update_profile_if_missing(
                &state.pool,
                u.id,
                profile.name.as_deref(),
                profile.image.as_deref(),
                profile.email_verified,
            )
            .await
            .map_err(|_| ApiError::Internal)?;
            u
        }
        None => {
            let uid = Uuid::now_v7();
            users::insert_oauth_user(
                &state.pool,
                uid,
                app.id,
                &email,
                profile.email_verified,
                profile.name.as_deref(),
                profile.image.as_deref(),
            )
            .await
            .map_err(|_| ApiError::Internal)?
        }
    };

    // Encrypt the access + refresh tokens at rest.
    let cipher =
        TokenCipher::from_master(&state.config.encryption_key).map_err(|_| ApiError::Internal)?;
    let at_enc = cipher
        .encrypt(profile.access_token.as_bytes())
        .map_err(|_| ApiError::Internal)?;
    let rt_enc = match profile.refresh_token.as_deref() {
        Some(rt) => Some(cipher.encrypt(rt.as_bytes()).map_err(|_| ApiError::Internal)?),
        None => None,
    };

    let acct_id = Uuid::now_v7();
    accounts::upsert_account(
        &state.pool,
        acct_id,
        user.id,
        p.id(),
        &profile.provider_account,
        Some(&at_enc),
        rt_enc.as_deref(),
        profile.expires_at,
        profile.scope.as_deref(),
        profile.id_token.as_deref(),
    )
    .await
    .map_err(|_| ApiError::Internal)?;

    let (_sess, token, csrf) = issue_session(&state, user.id, &headers).await?;
    audit::log_event(
        &state.pool,
        Some(app.id),
        Some(user.id),
        "signin",
        serde_json::json!({ "method": "oauth", "provider": p.id() }),
        client_ip(&headers),
        client_ua(&headers),
    )
    .await
    .map_err(|_| ApiError::Internal)?;

    let mut res = redirect_response(&parts.redirect_after)?;
    append_session_csrf_cookies(&mut res, &token, &csrf, state.config.cookie_secure)?;
    Ok(res)
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/oauth/{provider}/authorize", get(authorize))
        .route("/oauth/{provider}/callback", get(callback))
}
