use std::sync::Arc;

use axum::http::StatusCode;
use governor::middleware::NoOpMiddleware;
use tower_governor::governor::GovernorConfigBuilder;
use tower_governor::key_extractor::PeerIpKeyExtractor;
use tower_governor::{GovernorError, GovernorLayer};

pub fn v1_rate_limit_layer() -> GovernorLayer<PeerIpKeyExtractor, NoOpMiddleware, axum::body::Body>
{
    let config = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(50)
            .burst_size(100)
            .finish()
            .expect("governor config"),
    );
    GovernorLayer::new(config).error_handler(|e: GovernorError| match e {
        GovernorError::TooManyRequests { .. } => axum::response::Response::builder()
            .status(StatusCode::TOO_MANY_REQUESTS)
            .body(axum::body::Body::empty())
            .unwrap(),
        _ => axum::response::Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(axum::body::Body::empty())
            .unwrap(),
    })
}
