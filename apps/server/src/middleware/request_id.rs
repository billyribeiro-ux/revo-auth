use axum::body::Body;
use axum::http::Request;
use axum::middleware::Next;
use axum::response::Response;
use uuid::Uuid;

const HDR: &str = "x-request-id";

pub async fn request_id_middleware(mut req: Request<Body>, next: Next) -> Response {
    let id = req
        .headers()
        .get(HDR)
        .and_then(|v| v.to_str().ok())
        .map(String::from)
        .unwrap_or_else(|| Uuid::now_v7().to_string());
    req.extensions_mut().insert(id.clone());
    let mut res = next.run(req).await;
    if let Ok(v) = axum::http::HeaderValue::from_str(&id) {
        res.headers_mut().insert(HDR, v);
    }
    res
}
