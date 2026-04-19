//! TOTP setup / confirm / verify / disable.
//!
//! Secrets are stored AES-256-GCM encrypted via `TokenCipher`. Recovery codes
//! are SHA-256 hashed and consumed on use. A TOTP verify step issues an
//! "upgraded" session (MFA-rotated).

use axum::extract::State;
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::{Extension, Json, Router};
use garde::Validate;
use ring::digest;
use ring::rand::{SecureRandom, SystemRandom};
use serde::{Deserialize, Serialize};
use subtle::ConstantTimeEq;
use totp_rs::{Algorithm, TOTP};

use crate::crypto::tokens::TokenCipher;
use crate::db::{audit, sessions, totp, users};
use crate::error::ApiError;
use crate::middleware::auth::load_session_user;
use crate::middleware::tenant::Tenant;
use crate::routes::v1::session::{append_session_csrf_cookies, issue_session};
use crate::state::AppState;

const TOTP_DIGITS: usize = 6;
const TOTP_STEP_SECS: u64 = 30;
const TOTP_SKEW: u8 = 1;
const SECRET_LEN: usize = 20; // RFC 6238 recommends ≥160 bits.
const ISSUER: &str = "Revo Auth";
const RECOVERY_CODE_COUNT: usize = 10;
const RECOVERY_CODE_BYTES: usize = 10;

fn client_ip(headers: &HeaderMap) -> Option<&str> {
    headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .or_else(|| headers.get("x-real-ip").and_then(|v| v.to_str().ok()))
}
fn client_ua(headers: &HeaderMap) -> Option<&str> {
    headers.get(header::USER_AGENT).and_then(|v| v.to_str().ok())
}

fn sha256(bytes: &[u8]) -> Vec<u8> {
    digest::digest(&digest::SHA256, bytes).as_ref().to_vec()
}

fn gen_secret() -> Result<Vec<u8>, ApiError> {
    let rng = SystemRandom::new();
    let mut buf = [0u8; SECRET_LEN];
    rng.fill(&mut buf).map_err(|_| ApiError::Internal)?;
    Ok(buf.to_vec())
}

fn gen_recovery_codes() -> Result<Vec<String>, ApiError> {
    let rng = SystemRandom::new();
    let mut out = Vec::with_capacity(RECOVERY_CODE_COUNT);
    for _ in 0..RECOVERY_CODE_COUNT {
        let mut b = [0u8; RECOVERY_CODE_BYTES];
        rng.fill(&mut b).map_err(|_| ApiError::Internal)?;
        out.push(hex::encode(b));
    }
    Ok(out)
}

fn build_totp(account: &str, secret: Vec<u8>) -> Result<TOTP, ApiError> {
    TOTP::new(
        Algorithm::SHA1,
        TOTP_DIGITS,
        TOTP_SKEW,
        TOTP_STEP_SECS,
        secret,
        Some(ISSUER.to_string()),
        account.to_string(),
    )
    .map_err(|_| ApiError::Internal)
}

#[derive(Serialize)]
pub struct SetupResponse {
    pub otpauth_url: String,
    pub recovery_codes: Vec<String>,
}

pub async fn setup(
    State(state): State<AppState>,
    Extension(Tenant(app)): Extension<Tenant>,
    headers: HeaderMap,
) -> Result<Json<SetupResponse>, ApiError> {
    let Some((_sess, user)) = load_session_user(&state, &headers).await? else {
        return Err(ApiError::Unauthorized);
    };
    if user.app_id != app.id {
        return Err(ApiError::Forbidden);
    }

    let secret = gen_secret()?;
    let cipher =
        TokenCipher::from_master(&state.config.encryption_key).map_err(|_| ApiError::Internal)?;
    let enc = cipher.encrypt(&secret).map_err(|_| ApiError::Internal)?;

    let account = user.email.clone().unwrap_or_else(|| user.id.to_string());
    let totp = build_totp(&account, secret)?;
    let otpauth_url = totp.get_url();

    totp::upsert_unconfirmed(&state.pool, user.id, &enc).await.map_err(|_| ApiError::Internal)?;

    let codes = gen_recovery_codes()?;
    let hashes: Vec<Vec<u8>> = codes.iter().map(|c| sha256(c.as_bytes())).collect();
    totp::set_recovery_codes(&state.pool, user.id, &hashes)
        .await
        .map_err(|_| ApiError::Internal)?;

    Ok(Json(SetupResponse { otpauth_url, recovery_codes: codes }))
}

#[derive(Deserialize, Validate)]
pub struct CodeBody {
    #[garde(length(min = 6, max = 12))]
    pub code: String,
}

async fn load_totp_or_err(state: &AppState, user_id: uuid::Uuid) -> Result<TOTP, ApiError> {
    let enc = totp::get_secret_enc(&state.pool, user_id)
        .await
        .map_err(|_| ApiError::Internal)?
        .ok_or(ApiError::BadRequest("totp is not configured".into()))?;
    let cipher =
        TokenCipher::from_master(&state.config.encryption_key).map_err(|_| ApiError::Internal)?;
    let secret = cipher.decrypt(&enc).map_err(|_| ApiError::Internal)?;
    build_totp(&user_id.to_string(), secret)
}

fn check_code_constant_time(totp: &TOTP, provided: &str) -> Result<bool, ApiError> {
    let current = totp.generate_current().map_err(|_| ApiError::Internal)?;
    Ok(current.as_bytes().ct_eq(provided.as_bytes()).unwrap_u8() == 1
        || totp.check_current(provided).map_err(|_| ApiError::Internal)?)
}

pub async fn confirm(
    State(state): State<AppState>,
    Extension(Tenant(app)): Extension<Tenant>,
    headers: HeaderMap,
    Json(body): Json<CodeBody>,
) -> Result<StatusCode, ApiError> {
    body.validate().map_err(|e| ApiError::Validation(e.to_string()))?;
    let Some((_sess, user)) = load_session_user(&state, &headers).await? else {
        return Err(ApiError::Unauthorized);
    };
    if user.app_id != app.id {
        return Err(ApiError::Forbidden);
    }
    let totp = load_totp_or_err(&state, user.id).await?;
    if !check_code_constant_time(&totp, &body.code)? {
        return Err(ApiError::InvalidCredentials);
    }
    totp::confirm(&state.pool, user.id).await.map_err(|_| ApiError::Internal)?;
    audit::log_event(
        &state.pool,
        Some(app.id),
        Some(user.id),
        "mfa-enable",
        serde_json::json!({ "kind": "totp" }),
        client_ip(&headers),
        client_ua(&headers),
    )
    .await
    .map_err(|_| ApiError::Internal)?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize, Validate)]
pub struct VerifyBody {
    #[garde(email)]
    pub email: String,
    #[garde(length(min = 6, max = 20))]
    pub code: String,
}

/// Verify TOTP during MFA step-up. Consumes recovery codes if a recovery code
/// is supplied (length > 6, hex).
pub async fn verify(
    State(state): State<AppState>,
    Extension(Tenant(app)): Extension<Tenant>,
    headers: HeaderMap,
    Json(body): Json<VerifyBody>,
) -> Result<Response, ApiError> {
    body.validate().map_err(|e| ApiError::Validation(e.to_string()))?;
    let user = users::find_by_email(&state.pool, app.id, &body.email)
        .await
        .map_err(|_| ApiError::Internal)?
        .ok_or(ApiError::InvalidCredentials)?;
    let confirmed_at =
        totp::get_confirmed_at(&state.pool, user.id).await.map_err(|_| ApiError::Internal)?;
    if confirmed_at.is_none() {
        return Err(ApiError::BadRequest("totp is not configured".into()));
    }

    let totp_ok = match load_totp_or_err(&state, user.id).await {
        Ok(t) => check_code_constant_time(&t, &body.code)?,
        Err(_) => false,
    };

    let ok = if totp_ok {
        true
    } else {
        // Try recovery codes.
        let codes = totp::get_recovery_codes(&state.pool, user.id)
            .await
            .map_err(|_| ApiError::Internal)?
            .unwrap_or_default();
        let h = sha256(body.code.as_bytes());
        let mut hit = None;
        for (i, stored) in codes.iter().enumerate() {
            if stored.ct_eq(&h).unwrap_u8() == 1 {
                hit = Some(i);
                break;
            }
        }
        match hit {
            Some(i) => {
                let remaining: Vec<Vec<u8>> = codes
                    .iter()
                    .enumerate()
                    .filter_map(|(j, c)| if j == i { None } else { Some(c.clone()) })
                    .collect();
                totp::set_recovery_codes(&state.pool, user.id, &remaining)
                    .await
                    .map_err(|_| ApiError::Internal)?;
                true
            }
            None => false,
        }
    };

    if !ok {
        audit::log_event(
            &state.pool,
            Some(app.id),
            Some(user.id),
            "failed-signin",
            serde_json::json!({ "method": "totp" }),
            client_ip(&headers),
            client_ua(&headers),
        )
        .await
        .ok();
        return Err(ApiError::InvalidCredentials);
    }

    // Session rotation on MFA step-up: revoke any current session token if
    // present, issue a fresh one.
    if let Some((cur, _)) = load_session_user(&state, &headers).await? {
        sessions::revoke(&state.pool, cur.id).await.map_err(|_| ApiError::Internal)?;
        let ttl = (cur.expires_at - chrono::Utc::now()).num_seconds().max(0) as u64;
        let _ = state.mark_session_revoked(cur.id, ttl).await;
    }
    let (session, token, csrf) = issue_session(&state, user.id, &headers).await?;

    audit::log_event(
        &state.pool,
        Some(app.id),
        Some(user.id),
        "signin",
        serde_json::json!({ "method": "totp" }),
        client_ip(&headers),
        client_ua(&headers),
    )
    .await
    .map_err(|_| ApiError::Internal)?;

    let mut res = (StatusCode::OK, Json(serde_json::json!({ "session": session }))).into_response();
    append_session_csrf_cookies(&mut res, &token, &csrf, state.config.cookie_secure)?;
    Ok(res)
}

pub async fn disable(
    State(state): State<AppState>,
    Extension(Tenant(app)): Extension<Tenant>,
    headers: HeaderMap,
    Json(body): Json<CodeBody>,
) -> Result<StatusCode, ApiError> {
    body.validate().map_err(|e| ApiError::Validation(e.to_string()))?;
    let Some((_sess, user)) = load_session_user(&state, &headers).await? else {
        return Err(ApiError::Unauthorized);
    };
    if user.app_id != app.id {
        return Err(ApiError::Forbidden);
    }
    let totp = load_totp_or_err(&state, user.id).await?;
    if !check_code_constant_time(&totp, &body.code)? {
        return Err(ApiError::InvalidCredentials);
    }
    totp::delete_for_user(&state.pool, user.id).await.map_err(|_| ApiError::Internal)?;
    audit::log_event(
        &state.pool,
        Some(app.id),
        Some(user.id),
        "mfa-disable",
        serde_json::json!({ "kind": "totp" }),
        client_ip(&headers),
        client_ua(&headers),
    )
    .await
    .map_err(|_| ApiError::Internal)?;
    Ok(StatusCode::NO_CONTENT)
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/totp/setup", post(setup))
        .route("/totp/confirm", post(confirm))
        .route("/totp/verify", post(verify))
        .route("/totp/disable", post(disable))
}
