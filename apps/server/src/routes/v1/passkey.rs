//! WebAuthn / passkey routes (webauthn-rs 0.5, passkey mode).
//!
//! See `crate::webauthn` for the in-process state store and the feature-flag
//! trade-off that requires it. `passkeys.public_key` carries the serialised
//! `Passkey` JSON so we can rehydrate during authentication without schema
//! changes.

use axum::extract::{Path, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, post};
use axum::{Extension, Json, Router};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use garde::Validate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use webauthn_rs::prelude::{PublicKeyCredential, RegisterPublicKeyCredential};

use crate::crypto::tokens::{random_token_32, token_b64url};
use crate::db::{audit, passkeys, users};
use crate::error::ApiError;
use crate::middleware::auth::load_session_user;
use crate::middleware::tenant::Tenant;
use crate::routes::v1::session::{append_session_csrf_cookies, issue_session};
use crate::state::AppState;
use crate::webauthn::{auth_put, auth_take, build_webauthn, reg_put, reg_take, rp_from_app};

const FLOW_TTL_SECS: i64 = 600;
const REDIS_PREFIX_REG: &str = "webauthn:reg";
const REDIS_PREFIX_AUTH: &str = "webauthn:auth";

fn new_flow_id() -> Result<String, ApiError> {
    let raw = random_token_32().map_err(|_| ApiError::Internal)?;
    Ok(token_b64url(&raw))
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

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterFinishBody {
    #[garde(skip)]
    pub flow_id: String,
    #[garde(skip)]
    pub credential: RegisterPublicKeyCredential,
    #[garde(length(min = 1, max = 64))]
    pub friendly_name: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct AuthenticateBeginBody {
    #[garde(email)]
    pub email: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct AuthenticateFinishBody {
    #[garde(skip)]
    pub flow_id: String,
    #[garde(skip)]
    pub credential: PublicKeyCredential,
}

#[derive(Serialize)]
struct BeginResponse<T: Serialize> {
    flow_id: String,
    challenge: T,
}

pub async fn register_begin(
    State(state): State<AppState>,
    Extension(Tenant(app)): Extension<Tenant>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let Some((_sess, user)) = load_session_user(&state, &headers).await? else {
        return Err(ApiError::Unauthorized);
    };
    if user.app_id != app.id {
        return Err(ApiError::Forbidden);
    }
    let (rp_id, origin) = rp_from_app(&app.origins).map_err(|_| ApiError::Internal)?;
    let webauthn = build_webauthn(&rp_id, &origin).map_err(|_| ApiError::Internal)?;

    // Exclude all existing credentials.
    let existing =
        passkeys::list_for_user(&state.pool, user.id).await.map_err(|_| ApiError::Internal)?;
    let exclude: Vec<webauthn_rs::prelude::CredentialID> = existing
        .iter()
        .map(|p| webauthn_rs::prelude::CredentialID::from(p.credential_id.clone()))
        .collect();
    let exclude = if exclude.is_empty() { None } else { Some(exclude) };

    let display =
        user.name.clone().or_else(|| user.email.clone()).unwrap_or_else(|| user.id.to_string());
    let name = user.email.clone().unwrap_or_else(|| user.id.to_string());

    let (ccr, reg_state) =
        webauthn.start_passkey_registration(user.id, &name, &display, exclude).map_err(|e| {
            tracing::warn!(error = ?e, "start_passkey_registration failed");
            ApiError::Internal
        })?;

    let flow_id = new_flow_id()?;
    reg_put(&flow_id, reg_state, FLOW_TTL_SECS);
    // Mirror the existence marker in Redis with TTL so ops can see active flows.
    let _ = state
        .cache_set(&format!("{REDIS_PREFIX_REG}:{}:{flow_id}", user.id), "1", FLOW_TTL_SECS as u64)
        .await;

    Ok((StatusCode::OK, Json(BeginResponse { flow_id, challenge: ccr })).into_response())
}

pub async fn register_finish(
    State(state): State<AppState>,
    Extension(Tenant(app)): Extension<Tenant>,
    headers: HeaderMap,
    Json(body): Json<RegisterFinishBody>,
) -> Result<StatusCode, ApiError> {
    body.validate().map_err(|e| ApiError::Validation(e.to_string()))?;
    let Some((_sess, user)) = load_session_user(&state, &headers).await? else {
        return Err(ApiError::Unauthorized);
    };
    if user.app_id != app.id {
        return Err(ApiError::Forbidden);
    }
    let (rp_id, origin) = rp_from_app(&app.origins).map_err(|_| ApiError::Internal)?;
    let webauthn = build_webauthn(&rp_id, &origin).map_err(|_| ApiError::Internal)?;

    let reg_state = reg_take(&body.flow_id)
        .ok_or(ApiError::BadRequest("passkey registration flow expired or unknown".into()))?;
    let _ = state.cache_del(&format!("{REDIS_PREFIX_REG}:{}:{}", user.id, body.flow_id)).await;

    let passkey =
        webauthn.finish_passkey_registration(&body.credential, &reg_state).map_err(|e| {
            tracing::warn!(error = ?e, "finish_passkey_registration failed");
            ApiError::BadRequest("passkey registration failed".into())
        })?;

    let cred_id_bytes: Vec<u8> = passkey.cred_id().as_ref().to_vec();
    // Store full serialised Passkey JSON in `public_key` so we can rehydrate.
    let passkey_blob = serde_json::to_vec(&passkey).map_err(|_| ApiError::Internal)?;
    let transports: Vec<String> = Vec::new();

    let id = Uuid::now_v7();
    passkeys::insert_passkey(
        &state.pool,
        id,
        user.id,
        &cred_id_bytes,
        &passkey_blob,
        &transports,
        false,
        false,
        body.friendly_name.as_deref(),
    )
    .await
    .map_err(|e| {
        // The unique constraint on `credential_id` prevents duplicate enrolment.
        tracing::warn!(error = ?e, "insert_passkey failed");
        ApiError::Conflict
    })?;

    audit::log_event(
        &state.pool,
        Some(app.id),
        Some(user.id),
        "passkey-add",
        serde_json::json!({ "passkey_id": id }),
        client_ip(&headers),
        client_ua(&headers),
    )
    .await
    .map_err(|_| ApiError::Internal)?;

    Ok(StatusCode::CREATED)
}

pub async fn authenticate_begin(
    State(state): State<AppState>,
    Extension(Tenant(app)): Extension<Tenant>,
    Json(body): Json<AuthenticateBeginBody>,
) -> Result<Response, ApiError> {
    body.validate().map_err(|e| ApiError::Validation(e.to_string()))?;
    let (rp_id, origin) = rp_from_app(&app.origins).map_err(|_| ApiError::Internal)?;
    let webauthn = build_webauthn(&rp_id, &origin).map_err(|_| ApiError::Internal)?;

    // Per webauthn-rs 0.5 (without the `resident-key-support` / discoverable
    // feature flag we'd need upstream), discoverable-credential flow is not
    // available on the stable Passkey API. Email is therefore required here.
    let email = body
        .email
        .as_deref()
        .ok_or(ApiError::BadRequest("email is required for passkey authentication".into()))?;
    let user = users::find_by_email(&state.pool, app.id, email)
        .await
        .map_err(|_| ApiError::Internal)?
        .ok_or(ApiError::NotFound)?;
    let rows =
        passkeys::list_for_user(&state.pool, user.id).await.map_err(|_| ApiError::Internal)?;
    let mut keys = Vec::with_capacity(rows.len());
    for r in rows {
        let pk: webauthn_rs::prelude::Passkey =
            serde_json::from_slice(&r.public_key).map_err(|_| ApiError::Internal)?;
        keys.push(pk);
    }
    if keys.is_empty() {
        return Err(ApiError::NotFound);
    }
    let (rcr, auth_state) =
        webauthn.start_passkey_authentication(&keys).map_err(|_| ApiError::Internal)?;
    let challenge_json = serde_json::to_value(rcr).map_err(|_| ApiError::Internal)?;
    let user_id = Some(user.id);

    let flow_id = new_flow_id()?;
    auth_put(&flow_id, auth_state, user_id, FLOW_TTL_SECS);
    if let Some(uid) = user_id {
        let _ = state
            .cache_set(&format!("{REDIS_PREFIX_AUTH}:{uid}:{flow_id}"), "1", FLOW_TTL_SECS as u64)
            .await;
    }

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({ "flow_id": flow_id, "challenge": challenge_json })),
    )
        .into_response())
}

pub async fn authenticate_finish(
    State(state): State<AppState>,
    Extension(Tenant(app)): Extension<Tenant>,
    headers: HeaderMap,
    Json(body): Json<AuthenticateFinishBody>,
) -> Result<Response, ApiError> {
    body.validate().map_err(|e| ApiError::Validation(e.to_string()))?;
    let (rp_id, origin) = rp_from_app(&app.origins).map_err(|_| ApiError::Internal)?;
    let webauthn = build_webauthn(&rp_id, &origin).map_err(|_| ApiError::Internal)?;

    let (auth_state, user_id) = auth_take(&body.flow_id)
        .ok_or(ApiError::BadRequest("passkey authentication flow expired or unknown".into()))?;
    if let Some(uid) = user_id {
        let _ = state.cache_del(&format!("{REDIS_PREFIX_AUTH}:{uid}:{}", body.flow_id)).await;
    }

    let result =
        webauthn.finish_passkey_authentication(&body.credential, &auth_state).map_err(|e| {
            tracing::warn!(error = ?e, "finish_passkey_authentication failed");
            ApiError::InvalidCredentials
        })?;

    let cred_id_bytes: Vec<u8> = result.cred_id().as_ref().to_vec();
    let row = passkeys::find_by_credential(&state.pool, &cred_id_bytes)
        .await
        .map_err(|_| ApiError::Internal)?
        .ok_or(ApiError::InvalidCredentials)?;
    let user = users::get_by_id(&state.pool, row.user_id)
        .await
        .map_err(|_| ApiError::Internal)?
        .ok_or(ApiError::InvalidCredentials)?;
    if user.app_id != app.id {
        return Err(ApiError::Forbidden);
    }

    // Anti-clone check + persist updated counter.
    let new_counter = result.counter() as i64;
    if new_counter > 0 && new_counter <= row.sign_count {
        return Err(ApiError::InvalidCredentials);
    }
    passkeys::update_passkey_after_auth(
        &state.pool,
        &cred_id_bytes,
        new_counter,
        result.backup_state(),
    )
    .await
    .map_err(|_| ApiError::Internal)?;

    let (session, token, csrf) = issue_session(&state, user.id, &headers).await?;
    audit::log_event(
        &state.pool,
        Some(app.id),
        Some(user.id),
        "signin",
        serde_json::json!({ "method": "passkey" }),
        client_ip(&headers),
        client_ua(&headers),
    )
    .await
    .map_err(|_| ApiError::Internal)?;

    let mut res = (StatusCode::OK, Json(serde_json::json!({ "session": session }))).into_response();
    append_session_csrf_cookies(&mut res, &token, &csrf, state.config.cookie_secure)?;
    Ok(res)
}

#[derive(Serialize)]
struct PasskeyOut {
    id: Uuid,
    credential_id_b64: String,
    friendly_name: Option<String>,
    backup_eligible: bool,
    backup_state: bool,
    last_used_at: Option<chrono::DateTime<chrono::Utc>>,
    created_at: chrono::DateTime<chrono::Utc>,
}

pub async fn list_passkeys(
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
    let rows =
        passkeys::list_for_user(&state.pool, user.id).await.map_err(|_| ApiError::Internal)?;
    let out: Vec<_> = rows
        .into_iter()
        .map(|r| PasskeyOut {
            id: r.id,
            credential_id_b64: URL_SAFE_NO_PAD.encode(&r.credential_id),
            friendly_name: r.friendly_name,
            backup_eligible: r.backup_eligible,
            backup_state: r.backup_state,
            last_used_at: r.last_used_at,
            created_at: r.created_at,
        })
        .collect();
    Ok(Json(serde_json::json!({ "passkeys": out })))
}

pub async fn delete_passkey(
    State(state): State<AppState>,
    Extension(Tenant(app)): Extension<Tenant>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let Some((_sess, user)) = load_session_user(&state, &headers).await? else {
        return Err(ApiError::Unauthorized);
    };
    if user.app_id != app.id {
        return Err(ApiError::Forbidden);
    }
    let affected =
        passkeys::delete_passkey(&state.pool, id, user.id).await.map_err(|_| ApiError::Internal)?;
    if affected == 0 {
        return Err(ApiError::NotFound);
    }
    audit::log_event(
        &state.pool,
        Some(app.id),
        Some(user.id),
        "passkey-remove",
        serde_json::json!({ "passkey_id": id }),
        client_ip(&headers),
        client_ua(&headers),
    )
    .await
    .map_err(|_| ApiError::Internal)?;
    Ok(StatusCode::NO_CONTENT)
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/passkey/register/begin", post(register_begin))
        .route("/passkey/register/finish", post(register_finish))
        .route("/passkey/authenticate/begin", post(authenticate_begin))
        .route("/passkey/authenticate/finish", post(authenticate_finish))
        .route("/passkey", get(list_passkeys))
        .route("/passkey/{id}", delete(delete_passkey))
}
