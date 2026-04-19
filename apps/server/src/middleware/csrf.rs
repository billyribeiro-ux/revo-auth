use axum::body::Body;
use axum::http::Request;
use axum::middleware::Next;
use axum::response::Response;
use http::Method;

use crate::error::ApiError;
use crate::middleware::auth::{extract_csrf_token, extract_session_token};

pub async fn csrf_middleware(req: Request<Body>, next: Next) -> Result<Response, ApiError> {
    let method = req.method().clone();
    if !matches!(method, Method::POST | Method::PUT | Method::PATCH | Method::DELETE) {
        return Ok(next.run(req).await);
    }
    if extract_session_token(req.headers()).is_none() {
        return Ok(next.run(req).await);
    }
    let Some(cookie_csrf) = extract_csrf_token(req.headers()) else {
        return Err(ApiError::Forbidden);
    };
    let Some(hdr) = req.headers().get("x-csrf-token").and_then(|v| v.to_str().ok()) else {
        return Err(ApiError::Forbidden);
    };
    use subtle::ConstantTimeEq;
    let a = cookie_csrf.as_bytes();
    let b = hdr.as_bytes();
    if a.len() != b.len() || !bool::from(a.ct_eq(b)) {
        return Err(ApiError::Forbidden);
    }
    Ok(next.run(req).await)
}
