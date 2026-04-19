//! Passkey (WebAuthn) endpoint smoke tests.
//!
//! A full authenticator ceremony is prohibitively complex to simulate
//! faithfully inside a unit test, so the full round-trip is `#[ignore]`'d.
//! What we *can* validate cheaply: begin-register returns a challenge, a
//! malformed finish payload is rejected, and the list endpoint returns
//! an empty array when the caller has no passkeys yet.

mod common;

use serde_json::json;
use serial_test::serial;

use crate::common::{
    cookie_value, cookies_to_header, get_with_headers, post_json, post_json_with_session, setup,
};

fn unique_email(label: &str) -> String {
    format!("{label}-{}@example.test", uuid::Uuid::now_v7().simple())
}

async fn signup_and_session(
    ctx: &common::TestCtx,
) -> anyhow::Result<(Vec<String>, String)> {
    let (_s, _b, cookies) = post_json(
        ctx,
        "/v1/signup",
        &json!({
            "email": unique_email("pk"),
            "password": "Sup3r-strong-p@ssw0rd!",
        }),
    )
    .await?;
    let csrf = cookie_value(&cookies, "__Host-revoauth.csrf")
        .map(str::to_string)
        .unwrap_or_default();
    Ok((cookies, csrf))
}

#[tokio::test]
#[serial]
async fn list_passkeys_empty_for_new_user() -> anyhow::Result<()> {
    let ctx = setup().await;
    let (cookies, _csrf) = signup_and_session(&ctx).await?;
    let (status, body, _c) = get_with_headers(
        &ctx,
        "/v1/passkey",
        &[(axum::http::header::COOKIE, cookies_to_header(&cookies))],
    )
    .await?;
    // Either the endpoint returns 200 with an empty array, or scaffold 501
    // (treated as a soft-pass until the handler is wired).
    if status == 200 {
        assert!(
            body.get("passkeys").map(|v| v.is_array()).unwrap_or(false),
            "expected passkeys array, got {body}",
        );
        assert_eq!(body["passkeys"].as_array().map(|a| a.len()), Some(0));
    } else {
        assert_eq!(status, 501, "unexpected passkey list status: {status}");
    }
    Ok(())
}

#[tokio::test]
#[serial]
async fn register_begin_returns_challenge_shape() -> anyhow::Result<()> {
    let ctx = setup().await;
    let (cookies, csrf) = signup_and_session(&ctx).await?;
    let (status, body, _c) = post_json_with_session(
        &ctx,
        "/v1/passkey/register/begin",
        &json!({}),
        &cookies,
        Some(&csrf),
    )
    .await?;
    if status == 200 {
        // Real handler must produce a WebAuthn `PublicKeyCredentialCreationOptions`
        // JSON body — we only assert the envelope shape here.
        assert!(
            body.get("publicKey").is_some() || body.get("options").is_some(),
            "unexpected register/begin body: {body}",
        );
    } else {
        assert_eq!(
            status, 501,
            "expected 501 from scaffold register/begin, got {status}",
        );
    }
    Ok(())
}

#[tokio::test]
#[serial]
async fn register_finish_rejects_malformed_credential() -> anyhow::Result<()> {
    let ctx = setup().await;
    let (cookies, csrf) = signup_and_session(&ctx).await?;
    let (status, _body, _c) = post_json_with_session(
        &ctx,
        "/v1/passkey/register/finish",
        &json!({ "id": "not-a-real-credential", "response": { "foo": "bar" } }),
        &cookies,
        Some(&csrf),
    )
    .await?;
    // 400 once implemented, 501 while scaffolded.
    assert!(
        matches!(status.as_u16(), 400 | 501),
        "expected 400 or 501, got {status}",
    );
    Ok(())
}

#[tokio::test]
#[serial]
#[ignore = "full webauthn ceremony requires an authenticator simulator"]
async fn register_full_ceremony_roundtrip() -> anyhow::Result<()> {
    // Intentionally empty — enable once `webauthn-authenticator-rs` (or
    // equivalent) is in dev-dependencies and we can produce real attestation
    // objects that `webauthn-rs` will accept.
    Ok(())
}
