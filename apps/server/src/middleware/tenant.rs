use axum::body::Body;
use axum::extract::State;
use axum::http::{HeaderMap, Request};
use axum::middleware::Next;
use axum::response::Response;
use uuid::Uuid;

use crate::db;
use crate::error::ApiError;
use crate::state::AppState;

#[derive(Clone)]
pub struct Tenant(pub db::AppRow);

pub async fn resolve_from_headers(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<db::AppRow, ApiError> {
    let app_hdr =
        headers.get("x-revo-app-id").and_then(|v| v.to_str().ok()).ok_or(ApiError::Forbidden)?;
    let pk_hdr = headers
        .get("x-revo-app-public-key")
        .and_then(|v| v.to_str().ok())
        .ok_or(ApiError::Forbidden)?;
    let app_id: Uuid = app_hdr.parse().map_err(|_| ApiError::Forbidden)?;
    let row = db::apps::get_by_public_key(&state.pool, app_id, pk_hdr)
        .await
        .map_err(|_| ApiError::Internal)?
        .ok_or(ApiError::Forbidden)?;
    Ok(row)
}

pub fn origin_allowed(app: &db::AppRow, origin: Option<&str>) -> bool {
    let Some(origin) = origin else {
        return false;
    };
    app.origins.iter().any(|o| o == origin)
}

fn request_origin(headers: &HeaderMap) -> Option<String> {
    if let Some(o) = headers.get(axum::http::header::ORIGIN).and_then(|v| v.to_str().ok()) {
        return Some(o.to_string());
    }
    headers.get(axum::http::header::REFERER).and_then(|v| v.to_str().ok()).and_then(|r| {
        let u = url::Url::parse(r).ok()?;
        let scheme = u.scheme();
        let host = u.host_str()?;
        let port = u.port();
        match port {
            Some(p) if (scheme == "http" && p != 80) || (scheme == "https" && p != 443) => {
                Some(format!("{scheme}://{host}:{p}"))
            }
            _ => Some(format!("{scheme}://{host}")),
        }
    })
}

pub async fn resolve_tenant(state: &AppState, headers: &HeaderMap) -> Result<db::AppRow, ApiError> {
    if let Ok(app) = resolve_from_headers(state, headers).await {
        return Ok(app);
    }
    let origin_owned = request_origin(headers);
    let origin = origin_owned.as_deref();
    let app_hdr = headers
        .get("x-revo-app-id")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or(ApiError::Forbidden)?;
    let row = db::apps::get_by_id(&state.pool, app_hdr)
        .await
        .map_err(|_| ApiError::Internal)?
        .ok_or(ApiError::Forbidden)?;
    if !origin_allowed(&row, origin) {
        return Err(ApiError::Forbidden);
    }
    Ok(row)
}

pub async fn tenant_middleware(
    State(state): State<AppState>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, ApiError> {
    let app = resolve_tenant(&state, req.headers()).await?;
    req.extensions_mut().insert(Tenant(app));
    Ok(next.run(req).await)
}
