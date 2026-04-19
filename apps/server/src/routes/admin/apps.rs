use axum::extract::State;
use axum::http::{header, HeaderMap, StatusCode};
use axum::routing::{get, patch};
use axum::{Json, Router};
use garde::Validate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::crypto::password;
use crate::db::{apps, audit};
use crate::error::ApiError;
use crate::state::AppState;

pub(super) fn require_master(headers: &HeaderMap, master: &str) -> Result<(), ApiError> {
    if master.is_empty() {
        return Err(ApiError::Forbidden);
    }
    let Some(ah) = headers.get("x-revo-master-key").and_then(|v| v.to_str().ok()) else {
        return Err(ApiError::Forbidden);
    };
    use subtle::ConstantTimeEq;
    let a = ah.as_bytes();
    let b = master.as_bytes();
    if a.len() != b.len() || !bool::from(a.ct_eq(b)) {
        return Err(ApiError::Forbidden);
    }
    Ok(())
}

pub async fn list_apps(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, ApiError> {
    require_master(&headers, &state.config.master_key)?;
    let rows = apps::list_all(&state.pool).await.map_err(|_| ApiError::Internal)?;
    let out: Vec<_> = rows
        .iter()
        .map(|a| {
            serde_json::json!({
                "id": a.id,
                "slug": a.slug,
                "name": a.name,
                "origins": a.origins,
                "public_key": a.public_key,
                "created_at": a.created_at,
            })
        })
        .collect();
    Ok(Json(serde_json::json!({ "apps": out })))
}

#[derive(Deserialize, Validate)]
pub struct CreateAppBody {
    #[garde(length(min = 1, max = 64))]
    pub slug: String,
    #[garde(length(min = 1, max = 128))]
    pub name: String,
    #[garde(skip)]
    pub origins: Vec<String>,
}

#[derive(Serialize)]
pub struct CreateAppResponse {
    pub id: Uuid,
    pub slug: String,
    pub name: String,
    pub origins: Vec<String>,
    pub public_key: String,
    pub secret_key: String,
}

pub async fn create_app(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<CreateAppBody>,
) -> Result<(StatusCode, Json<CreateAppResponse>), ApiError> {
    require_master(&headers, &state.config.master_key)?;
    body.validate().map_err(|e| ApiError::Validation(e.to_string()))?;
    let id = Uuid::now_v7();
    let pk = format!(
        "pk_{}",
        hex::encode(crate::crypto::tokens::random_token_32().map_err(|_| ApiError::Internal)?)
    );
    let sk = format!(
        "sk_{}",
        hex::encode(crate::crypto::tokens::random_token_32().map_err(|_| ApiError::Internal)?)
    );
    let sk_hash = password::hash_app_secret(&sk).map_err(|_| ApiError::Internal)?;
    let row =
        apps::insert_app(&state.pool, id, &body.slug, &body.name, &body.origins, &pk, &sk_hash)
            .await
            .map_err(|_| ApiError::Conflict)?;
    audit::log_event(
        &state.pool,
        Some(row.id),
        None,
        "admin-app-create",
        serde_json::json!({ "slug": body.slug }),
        headers
            .get("x-forwarded-for")
            .and_then(|v| v.to_str().ok())
            .or_else(|| headers.get("x-real-ip").and_then(|v| v.to_str().ok())),
        headers.get(header::USER_AGENT).and_then(|v| v.to_str().ok()),
    )
    .await
    .map_err(|_| ApiError::Internal)?;
    Ok((
        StatusCode::CREATED,
        Json(CreateAppResponse {
            id: row.id,
            slug: row.slug,
            name: row.name,
            origins: row.origins,
            public_key: pk,
            secret_key: sk,
        }),
    ))
}

#[derive(Deserialize, Validate)]
pub struct PatchAppBody {
    #[garde(skip)]
    pub name: Option<String>,
    #[garde(skip)]
    pub origins: Option<Vec<String>>,
}

pub async fn patch_app(
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::extract::Path(aid): axum::extract::Path<Uuid>,
    Json(body): Json<PatchAppBody>,
) -> Result<Json<serde_json::Value>, ApiError> {
    require_master(&headers, &state.config.master_key)?;
    body.validate().map_err(|e| ApiError::Validation(e.to_string()))?;
    let row = apps::update_app(&state.pool, aid, body.name.as_deref(), body.origins.as_deref())
        .await
        .map_err(|_| ApiError::Internal)?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(serde_json::json!({
        "app": {
            "id": row.id,
            "slug": row.slug,
            "name": row.name,
            "origins": row.origins,
            "public_key": row.public_key,
        }
    })))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/apps", get(list_apps).post(create_app))
        .route("/apps/{id}", patch(patch_app))
}
