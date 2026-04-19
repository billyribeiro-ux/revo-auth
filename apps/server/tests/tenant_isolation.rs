//! Cross-tenant isolation: same email can coexist across apps; credentials
//! scoped to one app cannot authenticate against another; a session cookie
//! minted for app A is rejected when presented with app B's tenant headers.

mod common;

use serde_json::json;
use serial_test::serial;

use crate::common::{cookies_to_header, get_as_tenant, post_json_as_tenant, setup, setup_tenant};

#[tokio::test]
#[serial]
async fn same_email_across_apps_is_allowed() -> anyhow::Result<()> {
    let ctx = setup().await;
    let (app_b, pk_b, _secret_b) = setup_tenant(&ctx.pool, &["https://app-b.localhost"]).await?;
    let email = format!("shared-{}@example.test", uuid::Uuid::now_v7().simple());

    let (sa, body_a, _c) = post_json_as_tenant(
        &ctx.router,
        ctx.app.id,
        &ctx.public_key,
        "/v1/signup",
        &json!({ "email": email, "password": "Sup3r-strong-p@ssw0rd!" }),
    )
    .await?;
    assert_eq!(sa, 201, "signup on app A failed: {body_a}");

    let (sb, body_b, _c) = post_json_as_tenant(
        &ctx.router,
        app_b.id,
        &pk_b,
        "/v1/signup",
        &json!({ "email": email, "password": "Another-strong-P@ss9999" }),
    )
    .await?;
    assert_eq!(sb, 201, "signup on app B failed: {body_b}");
    Ok(())
}

#[tokio::test]
#[serial]
async fn signin_is_scoped_to_originating_app() -> anyhow::Result<()> {
    let ctx = setup().await;
    let (app_b, pk_b, _secret_b) = setup_tenant(&ctx.pool, &["https://app-b.localhost"]).await?;
    let email = format!("scoped-{}@example.test", uuid::Uuid::now_v7().simple());
    let pw_a = "Sup3r-strong-p@ssw0rd!";

    let (_s, _b, _c) = post_json_as_tenant(
        &ctx.router,
        ctx.app.id,
        &ctx.public_key,
        "/v1/signup",
        &json!({ "email": email, "password": pw_a }),
    )
    .await?;

    // Signin to app A with the original password works.
    let (sa, body_a, _c) = post_json_as_tenant(
        &ctx.router,
        ctx.app.id,
        &ctx.public_key,
        "/v1/signin",
        &json!({ "email": email, "password": pw_a }),
    )
    .await?;
    assert_eq!(sa, 200, "signin app A failed: {body_a}");

    // Signin to app B with app A's password fails — app B has no such user.
    let (sb, body_b, _c) = post_json_as_tenant(
        &ctx.router,
        app_b.id,
        &pk_b,
        "/v1/signin",
        &json!({ "email": email, "password": pw_a }),
    )
    .await?;
    assert_eq!(sb, 401);
    assert_eq!(body_b["error"]["code"], "INVALID_CREDENTIALS");
    Ok(())
}

#[tokio::test]
#[serial]
async fn session_is_rejected_when_presented_to_other_app() -> anyhow::Result<()> {
    let ctx = setup().await;
    let (app_b, pk_b, _secret_b) = setup_tenant(&ctx.pool, &["https://app-b.localhost"]).await?;
    let email = format!("xapp-{}@example.test", uuid::Uuid::now_v7().simple());

    let (_s, _b, cookies) = post_json_as_tenant(
        &ctx.router,
        ctx.app.id,
        &ctx.public_key,
        "/v1/signup",
        &json!({ "email": email, "password": "Sup3r-strong-p@ssw0rd!" }),
    )
    .await?;

    // Reuse app A's session cookie but address app B.
    let cookie_hdr = cookies_to_header(&cookies);
    let (status, body, _c) = get_as_tenant(
        &ctx.router,
        app_b.id,
        &pk_b,
        "/v1/session",
        &[(axum::http::header::COOKIE, cookie_hdr)],
    )
    .await?;
    assert_eq!(status, 403, "cross-tenant session reuse must 403; body={body}");
    assert_eq!(body["error"]["code"], "FORBIDDEN");
    Ok(())
}
