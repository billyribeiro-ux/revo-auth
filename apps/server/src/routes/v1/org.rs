use axum::extract::{Path, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::routing::{get, post};
use axum::{Extension, Json, Router};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use chrono::Duration;
use garde::Validate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::crypto::tokens::{hash_session_token, random_token_32, token_b64url};
use crate::db::{audit, memberships, orgs, users, verification};
use crate::error::ApiError;
use crate::middleware::auth::load_session_user;
use crate::middleware::tenant::Tenant;
use crate::state::AppState;

fn validate_slug(value: &str, _ctx: &()) -> garde::Result {
    let len = value.len();
    if !(2..=40).contains(&len) {
        return Err(garde::Error::new("slug length must be 2..=40"));
    }
    if !value
        .chars()
        .all(|c| matches!(c, 'a'..='z' | '0'..='9' | '-'))
    {
        return Err(garde::Error::new("slug must match [a-z0-9-]"));
    }
    Ok(())
}

fn validate_role(value: &str, _ctx: &()) -> garde::Result {
    if !matches!(value, "owner" | "admin" | "member") {
        return Err(garde::Error::new("role must be owner|admin|member"));
    }
    Ok(())
}

#[derive(Serialize)]
struct OrgOut {
    id: Uuid,
    slug: String,
    name: String,
    role: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
}

fn ip_from_headers(headers: &HeaderMap) -> Option<&str> {
    headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .or_else(|| headers.get("x-real-ip").and_then(|v| v.to_str().ok()))
}

pub async fn list_orgs(
    State(state): State<AppState>,
    Extension(Tenant(app)): Extension<Tenant>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, ApiError> {
    let (_sess, user) = load_session_user(&state, &headers)
        .await?
        .ok_or(ApiError::Unauthorized)?;
    if user.app_id != app.id {
        return Err(ApiError::Forbidden);
    }
    let rows =
        orgs::list_orgs_for_user(&state.pool, user.id).await.map_err(|_| ApiError::Internal)?;
    let mut out = Vec::with_capacity(rows.len());
    for r in rows {
        let role = memberships::role_of(&state.pool, r.id, user.id)
            .await
            .map_err(|_| ApiError::Internal)?;
        out.push(OrgOut {
            id: r.id,
            slug: r.slug,
            name: r.name,
            role,
            created_at: r.created_at,
        });
    }
    Ok(Json(serde_json::json!({ "orgs": out })))
}

#[derive(Deserialize, Validate)]
pub struct CreateOrgBody {
    #[garde(custom(validate_slug))]
    pub slug: String,
    #[garde(length(min = 1, max = 128))]
    pub name: String,
}

pub async fn create_org(
    State(state): State<AppState>,
    Extension(Tenant(app)): Extension<Tenant>,
    headers: HeaderMap,
    Json(body): Json<CreateOrgBody>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    body.validate().map_err(|e| ApiError::Validation(e.to_string()))?;
    let (_sess, user) = load_session_user(&state, &headers)
        .await?
        .ok_or(ApiError::Unauthorized)?;
    if user.app_id != app.id {
        return Err(ApiError::Forbidden);
    }
    let oid = Uuid::now_v7();
    let row = orgs::create_org(&state.pool, oid, app.id, &body.slug, &body.name)
        .await
        .map_err(|_| ApiError::Conflict)?;
    let mid = Uuid::now_v7();
    memberships::add_member(&state.pool, mid, row.id, user.id, "owner")
        .await
        .map_err(|_| ApiError::Internal)?;
    audit::log_event(
        &state.pool,
        Some(app.id),
        Some(user.id),
        "org-create",
        serde_json::json!({ "org_id": row.id, "slug": row.slug }),
        ip_from_headers(&headers),
        headers.get(header::USER_AGENT).and_then(|v| v.to_str().ok()),
    )
    .await
    .ok();
    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({
            "org": {
                "id": row.id,
                "slug": row.slug,
                "name": row.name,
                "role": "owner",
                "created_at": row.created_at,
            }
        })),
    ))
}

#[derive(Deserialize, Validate)]
pub struct InviteBody {
    #[garde(email)]
    pub email: String,
    #[garde(custom(validate_role))]
    pub role: String,
}

fn role_can_invite(role: &str) -> bool {
    matches!(role, "owner" | "admin")
}

pub async fn invite(
    State(state): State<AppState>,
    Extension(Tenant(app)): Extension<Tenant>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(body): Json<InviteBody>,
) -> Result<StatusCode, ApiError> {
    body.validate().map_err(|e| ApiError::Validation(e.to_string()))?;
    let (_sess, user) = load_session_user(&state, &headers)
        .await?
        .ok_or(ApiError::Unauthorized)?;
    if user.app_id != app.id {
        return Err(ApiError::Forbidden);
    }
    let org = orgs::get_by_id(&state.pool, id)
        .await
        .map_err(|_| ApiError::Internal)?
        .ok_or(ApiError::NotFound)?;
    if org.app_id != app.id {
        return Err(ApiError::Forbidden);
    }
    let role = memberships::role_of(&state.pool, org.id, user.id)
        .await
        .map_err(|_| ApiError::Internal)?
        .ok_or(ApiError::Forbidden)?;
    if !role_can_invite(&role) {
        return Err(ApiError::Forbidden);
    }

    // Invite target user must exist in this tenant so we can attach the
    // verification token to their user id. If they don't, create a shell
    // user with no password (signup completes on accept).
    let invitee = match users::find_by_email(&state.pool, app.id, &body.email)
        .await
        .map_err(|_| ApiError::Internal)?
    {
        Some(u) => u,
        None => {
            let uid = Uuid::now_v7();
            users::insert_user(&state.pool, uid, app.id, &body.email, None, None)
                .await
                .map_err(|_| ApiError::Internal)?
        }
    };

    let raw = random_token_32().map_err(|_| ApiError::Internal)?;
    let tok = token_b64url(&raw);
    let th = hash_session_token(&raw);
    let vid = Uuid::now_v7();
    let meta = serde_json::json!({ "org_id": org.id, "role": body.role });
    verification::insert_token_with_metadata(
        &state.pool,
        vid,
        invitee.id,
        "org_invite",
        &th,
        chrono::Utc::now() + Duration::days(7),
        &meta,
    )
    .await
    .map_err(|_| ApiError::Internal)?;

    let url = format!(
        "{}/orgs/{}/accept?token={}",
        state.config.app_base_url.trim_end_matches('/'),
        org.id,
        urlencoding::encode(&tok),
    );
    crate::email::send_org_invite(state.mail.as_ref(), &state.config, &body.email, &org.name, &url)
        .await
        .map_err(|_| ApiError::Internal)?;

    audit::log_event(
        &state.pool,
        Some(app.id),
        Some(user.id),
        "org-invite",
        serde_json::json!({ "org_id": org.id, "email": body.email, "role": body.role }),
        ip_from_headers(&headers),
        headers.get(header::USER_AGENT).and_then(|v| v.to_str().ok()),
    )
    .await
    .ok();

    Ok(StatusCode::ACCEPTED)
}

#[derive(Deserialize)]
pub struct AcceptBody {
    pub token: String,
}

pub async fn accept(
    State(state): State<AppState>,
    Extension(Tenant(app)): Extension<Tenant>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(body): Json<AcceptBody>,
) -> Result<StatusCode, ApiError> {
    let (_sess, user) = load_session_user(&state, &headers)
        .await?
        .ok_or(ApiError::Unauthorized)?;
    if user.app_id != app.id {
        return Err(ApiError::Forbidden);
    }
    let org = orgs::get_by_id(&state.pool, id)
        .await
        .map_err(|_| ApiError::Internal)?
        .ok_or(ApiError::NotFound)?;
    if org.app_id != app.id {
        return Err(ApiError::Forbidden);
    }

    let bytes = URL_SAFE_NO_PAD
        .decode(body.token.as_bytes())
        .map_err(|_| ApiError::BadRequest("invalid token".into()))?;
    let th = hash_session_token(&bytes);
    let row = verification::take_by_hash(&state.pool, "org_invite", &th)
        .await
        .map_err(|_| ApiError::Internal)?
        .ok_or(ApiError::BadRequest("invalid token".into()))?;

    let meta_org_id = row
        .metadata
        .get("org_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or(ApiError::BadRequest("invalid invite".into()))?;
    if meta_org_id != org.id {
        return Err(ApiError::BadRequest("invalid invite".into()));
    }
    let role = row.metadata.get("role").and_then(|v| v.as_str()).unwrap_or("member").to_string();
    if !matches!(role.as_str(), "owner" | "admin" | "member") {
        return Err(ApiError::BadRequest("invalid role".into()));
    }

    // The token was minted for row.user_id. Accept requires the session user
    // to match (email-bound invite) OR, if the invitee user has no password
    // and no other auth methods, allow session user to claim it. Keep strict:
    // session user must match invitee to avoid cross-account hijacking.
    if row.user_id != user.id {
        return Err(ApiError::Forbidden);
    }

    let mid = Uuid::now_v7();
    memberships::add_member(&state.pool, mid, org.id, user.id, &role)
        .await
        .map_err(|_| ApiError::Conflict)?;

    audit::log_event(
        &state.pool,
        Some(app.id),
        Some(user.id),
        "org-accept",
        serde_json::json!({ "org_id": org.id, "role": role }),
        ip_from_headers(&headers),
        headers.get(header::USER_AGENT).and_then(|v| v.to_str().ok()),
    )
    .await
    .ok();

    Ok(StatusCode::NO_CONTENT)
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/orgs", get(list_orgs).post(create_org))
        .route("/orgs/{id}/invite", post(invite))
        .route("/orgs/{id}/accept", post(accept))
}
