//! Signup / signin / signout happy-path and enumeration-safety checks.

mod common;

use std::time::Instant;

use serde_json::json;
use serial_test::serial;

use crate::common::{cookie_value, get_with_headers, post_json, post_json_with_session, setup};

fn unique_email(label: &str) -> String {
    format!("{label}-{}@example.test", uuid::Uuid::now_v7().simple())
}

#[tokio::test]
#[serial]
async fn signup_returns_session_cookie_pair() -> anyhow::Result<()> {
    let ctx = setup().await;
    let email = unique_email("signup");
    let (status, body, cookies) = post_json(
        &ctx,
        "/v1/signup",
        &json!({ "email": email, "password": "Sup3r-strong-p@ssw0rd!" }),
    )
    .await?;
    assert_eq!(status, 201, "signup status: body={body}");
    assert!(body["session"]["id"].is_string(), "body: {body}");
    assert!(
        cookie_value(&cookies, "__Host-revoauth.session").is_some(),
        "missing session cookie in {cookies:?}",
    );
    assert!(
        cookie_value(&cookies, "__Host-revoauth.csrf").is_some(),
        "missing csrf cookie in {cookies:?}",
    );
    Ok(())
}

#[tokio::test]
#[serial]
async fn session_endpoint_returns_user_after_signup() -> anyhow::Result<()> {
    let ctx = setup().await;
    let email = unique_email("sess");
    let (_s, _b, cookies) = post_json(
        &ctx,
        "/v1/signup",
        &json!({ "email": email, "password": "Sup3r-strong-p@ssw0rd!" }),
    )
    .await?;
    let cookie_hdr = common::cookies_to_header(&cookies);
    let (status, body, _c) =
        get_with_headers(&ctx, "/v1/session", &[(axum::http::header::COOKIE, cookie_hdr)]).await?;
    assert_eq!(status, 200, "session status: body={body}");
    assert_eq!(body["session"]["user"]["email"], email);
    Ok(())
}

#[tokio::test]
#[serial]
async fn signin_wrong_password_matches_unknown_user_shape_and_timing() -> anyhow::Result<()> {
    let ctx = setup().await;
    let known = unique_email("enum-known");
    let unknown = unique_email("enum-unknown");

    let (_s, _b, _c) = post_json(
        &ctx,
        "/v1/signup",
        &json!({ "email": known, "password": "Sup3r-strong-p@ssw0rd!" }),
    )
    .await?;

    let t0 = Instant::now();
    let (s_known, body_known, _c1) = post_json(
        &ctx,
        "/v1/signin",
        &json!({ "email": known, "password": "not-the-right-password" }),
    )
    .await?;
    let known_elapsed = t0.elapsed();

    let t1 = Instant::now();
    let (s_unknown, body_unknown, _c2) = post_json(
        &ctx,
        "/v1/signin",
        &json!({ "email": unknown, "password": "not-the-right-password" }),
    )
    .await?;
    let unknown_elapsed = t1.elapsed();

    assert_eq!(s_known, 401);
    assert_eq!(s_unknown, 401);
    assert_eq!(body_known["error"]["code"], "INVALID_CREDENTIALS");
    assert_eq!(body_unknown["error"]["code"], "INVALID_CREDENTIALS");
    // Bodies must be indistinguishable apart from the per-request id.
    assert_eq!(body_known["error"]["message"], body_unknown["error"]["message"]);
    // Both branches must exceed the 200 ms floor — enforced by the
    // handler's `pad_verify_elapsed`.
    assert!(
        known_elapsed.as_millis() > 200,
        "known-email signin returned too quickly: {known_elapsed:?}",
    );
    assert!(
        unknown_elapsed.as_millis() > 200,
        "unknown-email signin returned too quickly: {unknown_elapsed:?}",
    );
    Ok(())
}

#[tokio::test]
#[serial]
async fn signout_revokes_session() -> anyhow::Result<()> {
    let ctx = setup().await;
    let email = unique_email("signout");
    let (_s, _b, cookies) = post_json(
        &ctx,
        "/v1/signup",
        &json!({ "email": email, "password": "Sup3r-strong-p@ssw0rd!" }),
    )
    .await?;
    let csrf =
        cookie_value(&cookies, "__Host-revoauth.csrf").map(str::to_string).unwrap_or_default();

    let (s_out, _b, _c) =
        post_json_with_session(&ctx, "/v1/signout", &json!({}), &cookies, Some(&csrf)).await?;
    assert_eq!(s_out, 204);

    let cookie_hdr = common::cookies_to_header(&cookies);
    let (s_sess, body, _c) =
        get_with_headers(&ctx, "/v1/session", &[(axum::http::header::COOKIE, cookie_hdr)]).await?;
    assert_eq!(s_sess, 401, "expected 401 after signout, got body={body}");
    Ok(())
}

#[tokio::test]
#[serial]
async fn signup_rejects_short_password() -> anyhow::Result<()> {
    let ctx = setup().await;
    let (status, body, _c) = post_json(
        &ctx,
        "/v1/signup",
        &json!({ "email": unique_email("short"), "password": "short" }),
    )
    .await?;
    assert_eq!(status, 400);
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
    Ok(())
}

#[tokio::test]
#[serial]
async fn signup_rejects_common_password() -> anyhow::Result<()> {
    let ctx = setup().await;
    let (status, body, _c) = post_json(
        &ctx,
        "/v1/signup",
        &json!({ "email": unique_email("common"), "password": "password123456" }),
    )
    .await?;
    assert_eq!(status, 400);
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
    Ok(())
}
