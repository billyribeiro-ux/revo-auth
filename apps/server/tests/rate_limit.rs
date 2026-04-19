//! Confirm that tight hammering of /v1/signin from a single IP trips the
//! tower-governor layer and returns the `RATE_LIMITED` ApiError envelope.

mod common;

use axum::http::header::HeaderName;
use serde_json::json;
use serial_test::serial;

use crate::common::{post_json_with_headers, setup};

#[tokio::test]
#[serial]
async fn signin_rate_limit_returns_429_envelope() -> anyhow::Result<()> {
    let ctx = setup().await;
    let xff = HeaderName::from_static("x-forwarded-for");
    let ip = "203.0.113.77".to_string();
    let mut saw_429 = false;
    let mut last_body = serde_json::Value::Null;

    // Spec: /v1/signin limit = 5/min — 10 rapid requests guarantees trips.
    for _ in 0..10 {
        let (status, body, _cookies) = post_json_with_headers(
            &ctx,
            "/v1/signin",
            &json!({ "email": "nobody@example.test", "password": "whatever1234" }),
            &[(xff.clone(), ip.clone())],
        )
        .await?;
        if status == 429 {
            saw_429 = true;
            last_body = body;
            break;
        }
    }

    assert!(saw_429, "expected at least one 429 across 10 rapid /signin calls");
    assert_eq!(
        last_body["error"]["code"], "RATE_LIMITED",
        "body must use ApiError envelope: {last_body}",
    );
    Ok(())
}
