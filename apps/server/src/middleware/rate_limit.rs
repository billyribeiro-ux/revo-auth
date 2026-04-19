//! Rate-limit layers per the security spec:
//!   /v1/signin                          — 5 req/min per IP
//!   /v1/signup                          — 3 req/hour per IP
//!   /v1/password/reset/request          — 3 req/hour per IP
//!   /v1/magic/request                   — 5 req/hour per IP
//!   /v1/oauth/:provider/callback        — 10 req/min per IP
//!   /v1/*                               — 60 req/min global fallback (IP)
//!
//! Per-(IP,email) compounding is enforced in-handler via Redis using
//! `pair_bucket_check` below. tower-governor handles the IP layer; Redis
//! handles the (IP,email) pair layer where spec requires it.

use std::sync::Arc;

use axum::body::Body;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use governor::middleware::NoOpMiddleware;
use tower_governor::governor::{GovernorConfig, GovernorConfigBuilder};
use tower_governor::key_extractor::SmartIpKeyExtractor;
use tower_governor::{GovernorError, GovernorLayer};

use crate::error::ApiError;
use crate::state::AppState;

type Layer = GovernorLayer<SmartIpKeyExtractor, NoOpMiddleware, Body>;

fn cfg(per_secs: u64, burst: u32) -> Arc<GovernorConfig<SmartIpKeyExtractor, NoOpMiddleware>> {
    // governor v0.10 uses non-zero quota internally; saturate on bad inputs rather
    // than panic — these values are all static and known-good in practice.
    let builder = GovernorConfigBuilder::default()
        .per_second(per_secs.max(1))
        .burst_size(burst.max(1))
        .key_extractor(SmartIpKeyExtractor)
        .finish();
    Arc::new(builder.unwrap_or_else(|| {
        // Unreachable for static inputs above; construct a 1/s / burst=1 fallback
        // via the same builder so we never panic in a request path.
        GovernorConfigBuilder::default()
            .per_second(1)
            .burst_size(1)
            .key_extractor(SmartIpKeyExtractor)
            .finish()
            .expect("fallback governor config with per_second=1 burst=1 is always valid")
    }))
}

fn build(per_secs: u64, burst: u32) -> Layer {
    GovernorLayer::new(cfg(per_secs, burst)).error_handler(error_handler)
}

fn error_handler(err: GovernorError) -> Response {
    match err {
        GovernorError::TooManyRequests { .. } => ApiError::RateLimited.into_response(),
        GovernorError::UnableToExtractKey => {
            (StatusCode::BAD_REQUEST, "missing client address").into_response()
        }
        GovernorError::Other { msg, .. } => {
            tracing::warn!(?msg, "rate limiter internal error");
            ApiError::Internal.into_response()
        }
    }
}

/// Coarse global limit applied to `/v1/*` — 60 req/min per IP.
pub fn v1_global_layer() -> Layer {
    build(1, 60)
}

/// Signin: 5 req/min per IP.
pub fn signin_layer() -> Layer {
    build(12, 5)
}

/// Signup: 3 req/hour per IP.
pub fn signup_layer() -> Layer {
    build(1200, 3)
}

/// Password reset request: 3 req/hour per IP.
pub fn password_reset_request_layer() -> Layer {
    build(1200, 3)
}

/// Magic-link request: 5 req/hour per IP.
pub fn magic_request_layer() -> Layer {
    build(720, 5)
}

/// OAuth callback: 10 req/min per IP.
pub fn oauth_callback_layer() -> Layer {
    build(6, 10)
}

/// Enforce a `(IP, identifier)` pair bucket via Redis INCR + EXPIRE. Used inside
/// handlers where tower-governor's key extractor can't express the compound key.
pub async fn pair_bucket_check(
    state: &AppState,
    bucket: &str,
    ip: Option<&str>,
    identifier: &str,
    max: u32,
    window_secs: i64,
) -> Result<(), ApiError> {
    let ip = ip.unwrap_or("unknown");
    let key = format!("revo:rl:{bucket}:{ip}:{}", identifier.to_ascii_lowercase());
    use fred::prelude::*;
    let count: i64 = state.redis.incr(&key).await.map_err(|_| ApiError::Internal)?;
    if count == 1 {
        let _ = state.redis.expire::<(), _>(&key, window_secs).await;
    }
    if (count as u32) > max {
        return Err(ApiError::RateLimited);
    }
    Ok(())
}
