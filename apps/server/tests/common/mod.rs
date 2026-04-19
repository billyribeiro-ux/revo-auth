//! Shared integration-test harness for the Revo auth server.
//!
//! Spins up a single Postgres and single Redis container per test process
//! (via `tokio::sync::OnceCell`), runs migrations, and exposes helpers that
//! drive the Axum router in-process using `tower::ServiceExt::oneshot`.
//!
//! All tests that use this harness are expected to be annotated with
//! `#[serial_test::serial]` to avoid container/port contention.

#![allow(dead_code)]

use std::sync::Arc;

use axum::body::{to_bytes, Body};
use axum::http::{header, HeaderName, HeaderValue, Method, Request, StatusCode};
use axum::Router;
use fred::prelude::RedisPool;
use http_body_util::BodyExt;
use serde_json::Value;
use sqlx::PgPool;
use testcontainers::ContainerAsync;
use testcontainers_modules::postgres::Postgres as PgImage;
use testcontainers_modules::redis::Redis as RedisImage;
use testcontainers_modules::testcontainers::runners::AsyncRunner;
use tokio::sync::OnceCell;
use tower::ServiceExt;
use uuid::Uuid;

use revo_auth_server::config::Config;
use revo_auth_server::db::AppRow;
use revo_auth_server::email::transport::LoggingTransport;
use revo_auth_server::{app_router, AppState};

/// A single Set-Cookie header, returned raw so individual tests can split
/// on `=` / `;` for assertions without a cookie-parsing dep.
pub type SetCookie = String;

/// Shared context returned by [`setup`]: router + live infra + a freshly
/// provisioned tenant. Integration tests typically only need `router`,
/// `pool`, and the `app`/`public_key`/`secret` tuple.
pub struct TestCtx {
    pub router: Router,
    pub pool: PgPool,
    pub redis: RedisPool,
    pub app: AppRow,
    pub public_key: String,
    pub secret: String,
    pub state: AppState,
}

struct SharedInfra {
    pool: PgPool,
    redis: RedisPool,
    // Kept alive for the lifetime of the process so the container does
    // not get torn down while tests are still running.
    _pg_container: ContainerAsync<PgImage>,
    _redis_container: ContainerAsync<RedisImage>,
}

static INFRA: OnceCell<Arc<SharedInfra>> = OnceCell::const_new();

async fn init_infra() -> Arc<SharedInfra> {
    let pg = PgImage::default()
        .with_user("postgres")
        .with_password("postgres")
        .with_db_name("revo_auth_test")
        .start()
        .await
        .expect("testcontainers: postgres start");

    let pg_host = pg.get_host().await.expect("pg host");
    let pg_port = pg.get_host_port_ipv4(5432).await.expect("pg port");
    let db_url =
        format!("postgres://postgres:postgres@{pg_host}:{pg_port}/revo_auth_test");
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(8)
        .connect(&db_url)
        .await
        .expect("testcontainers: pg connect");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("testcontainers: migrate");

    let redis = RedisImage::default()
        .start()
        .await
        .expect("testcontainers: redis start");
    let r_host = redis.get_host().await.expect("redis host");
    let r_port =
        redis.get_host_port_ipv4(6379).await.expect("redis port");
    let redis_url = format!("redis://{r_host}:{r_port}");
    let redis_pool = AppState::connect_redis(&redis_url)
        .await
        .expect("testcontainers: redis connect");

    Arc::new(SharedInfra {
        pool,
        redis: redis_pool,
        _pg_container: pg,
        _redis_container: redis,
    })
}

async fn infra() -> Arc<SharedInfra> {
    INFRA.get_or_init(init_infra).await.clone()
}

/// Build a [`Config`] that is self-consistent for tests. Cookie `Secure=true`
/// stays on because the server uses the `__Host-` prefix — tests just need
/// not to reject it, they don't care about TLS.
fn test_config(database_url: &str, redis_url: &str) -> Config {
    // ES256 PEMs hand-generated out-of-band; these are *only* used for
    // integration tests and never leave this repo.
    let private_pem = "-----BEGIN PRIVATE KEY-----\n\
        MIGHAgEAMBMGByqGSM49AgEGCCqGSM49AwEHBG0wawIBAQQgevZzL1gdAFr88hb2\n\
        OF/2NxApJCzGCEDdfSp6VQO30hyhRANCAAQRWz+jn65BtOMvdyHKcvjBeBSDZH2r\n\
        1RTwjmYSi9R/zpBnuQ4EiMnCqfMPWiZqB4QdbAd0E7oH50VpuZ1P087G\n\
        -----END PRIVATE KEY-----\n";
    let public_pem = "-----BEGIN PUBLIC KEY-----\n\
        MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEEVs/o5+uQbTjL3chynL4wXgUg2R9\n\
        q9UU8I5mEovUf86QZ7kOBIjJwqnzD1omageEHWwHdBO6B+dFabmdT9POxg==\n\
        -----END PUBLIC KEY-----\n";
    Config {
        host: "127.0.0.1".into(),
        port: 0,
        database_url: database_url.into(),
        redis_url: redis_url.into(),
        master_key: "test-master-key".into(),
        encryption_key: "test-encryption-key-thirty-two-b!".into(),
        jwt_issuer: "revo-auth-test".into(),
        jwt_es256_private_pem: private_pem.into(),
        jwt_es256_public_pem: public_pem.into(),
        cookie_secure: true,
    }
}

/// Spin up (or reuse) the shared infra, provision a fresh app, and return
/// a ready-to-drive router plus the tenant's credentials.
pub async fn setup() -> TestCtx {
    let infra = infra().await;
    let (app, public_key, secret) =
        setup_tenant(&infra.pool, &["https://test.localhost"])
            .await
            .expect("setup_tenant");

    let cfg = test_config("inmem-unused", "inmem-unused");
    let state = AppState {
        pool: infra.pool.clone(),
        redis: infra.redis.clone(),
        config: Arc::new(cfg),
        mail: Arc::new(LoggingTransport),
    };
    let router = app_router(state.clone());

    TestCtx {
        router,
        pool: infra.pool.clone(),
        redis: infra.redis.clone(),
        app,
        public_key,
        secret,
        state,
    }
}

/// Insert a new app row with a known public key / secret so tests can
/// attach the `X-Revo-App-Id` / `X-Revo-App-Public-Key` tenant headers.
pub async fn setup_tenant(
    pool: &PgPool,
    origins: &[&str],
) -> anyhow::Result<(AppRow, String, String)> {
    use revo_auth_server::crypto::password::hash_app_secret;
    use revo_auth_server::db::apps;

    let id = Uuid::now_v7();
    let slug = format!("test-{}", &id.to_string()[..8]);
    let name = "test app".to_string();
    let pk = format!("pk_test_{}", id.simple());
    let secret = format!("sk_test_{}", id.simple());
    let secret_hash = hash_app_secret(&secret)
        .map_err(|_| anyhow::anyhow!("hash app secret"))?;
    let origins_owned: Vec<String> =
        origins.iter().map(|o| (*o).to_string()).collect();
    let row = apps::insert_app(
        pool,
        id,
        &slug,
        &name,
        &origins_owned,
        &pk,
        &secret_hash,
    )
    .await?;
    Ok((row, pk, secret))
}

fn tenant_headers(app_id: Uuid, public_key: &str) -> Vec<(HeaderName, HeaderValue)> {
    vec![
        (
            HeaderName::from_static("x-revo-app-id"),
            HeaderValue::from_str(&app_id.to_string()).expect("valid app-id header"),
        ),
        (
            HeaderName::from_static("x-revo-app-public-key"),
            HeaderValue::from_str(public_key).expect("valid public-key header"),
        ),
    ]
}

/// Drive the router and collect the JSON body plus every `Set-Cookie` line.
async fn send(
    router: &Router,
    req: Request<Body>,
) -> anyhow::Result<(StatusCode, Value, Vec<SetCookie>)> {
    let res = router.clone().oneshot(req).await?;
    let status = res.status();
    let set_cookies: Vec<SetCookie> = res
        .headers()
        .get_all(header::SET_COOKIE)
        .into_iter()
        .filter_map(|v| v.to_str().ok().map(str::to_string))
        .collect();
    let (_parts, body) = res.into_parts();
    let collected = body.collect().await?.to_bytes();
    let body_json: Value = if collected.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&collected).unwrap_or(Value::Null)
    };
    Ok((status, body_json, set_cookies))
}

/// Consume a response directly (for tests that need raw bytes or headers).
pub async fn collect_bytes(
    router: &Router,
    req: Request<Body>,
) -> anyhow::Result<(StatusCode, Vec<SetCookie>, Vec<u8>, axum::http::HeaderMap)> {
    let res = router.clone().oneshot(req).await?;
    let status = res.status();
    let headers = res.headers().clone();
    let set_cookies: Vec<SetCookie> = headers
        .get_all(header::SET_COOKIE)
        .into_iter()
        .filter_map(|v| v.to_str().ok().map(str::to_string))
        .collect();
    let bytes = to_bytes(res.into_body(), 1024 * 1024).await?.to_vec();
    Ok((status, set_cookies, bytes, headers))
}

/// GET `path` with tenant headers attached.
pub async fn get(
    ctx: &TestCtx,
    path: &str,
) -> anyhow::Result<(StatusCode, Value, Vec<SetCookie>)> {
    let mut req = Request::builder().method(Method::GET).uri(path);
    for (k, v) in tenant_headers(ctx.app.id, &ctx.public_key) {
        req = req.header(k, v);
    }
    let req = req.body(Body::empty())?;
    send(&ctx.router, req).await
}

/// GET with an explicit set of extra headers — useful for cookie tests.
pub async fn get_with_headers(
    ctx: &TestCtx,
    path: &str,
    extra: &[(HeaderName, String)],
) -> anyhow::Result<(StatusCode, Value, Vec<SetCookie>)> {
    let mut req = Request::builder().method(Method::GET).uri(path);
    for (k, v) in tenant_headers(ctx.app.id, &ctx.public_key) {
        req = req.header(k, v);
    }
    for (k, v) in extra {
        req = req.header(k.clone(), v);
    }
    let req = req.body(Body::empty())?;
    send(&ctx.router, req).await
}

/// POST JSON with tenant headers attached. CSRF is only enforced when the
/// request also carries a session cookie, so unauthenticated POSTs work.
pub async fn post_json(
    ctx: &TestCtx,
    path: &str,
    body: &Value,
) -> anyhow::Result<(StatusCode, Value, Vec<SetCookie>)> {
    let mut req = Request::builder()
        .method(Method::POST)
        .uri(path)
        .header(header::CONTENT_TYPE, "application/json");
    for (k, v) in tenant_headers(ctx.app.id, &ctx.public_key) {
        req = req.header(k, v);
    }
    let req = req.body(Body::from(serde_json::to_vec(body)?))?;
    send(&ctx.router, req).await
}

/// POST JSON while reusing an authenticated session: the caller passes the
/// Set-Cookie values from a prior signup/signin plus the CSRF token header.
pub async fn post_json_with_session(
    ctx: &TestCtx,
    path: &str,
    body: &Value,
    cookies: &[SetCookie],
    csrf: Option<&str>,
) -> anyhow::Result<(StatusCode, Value, Vec<SetCookie>)> {
    let cookie_header = cookies_to_header(cookies);
    let mut req = Request::builder()
        .method(Method::POST)
        .uri(path)
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::COOKIE, cookie_header);
    for (k, v) in tenant_headers(ctx.app.id, &ctx.public_key) {
        req = req.header(k, v);
    }
    if let Some(token) = csrf {
        req = req.header("x-csrf-token", token);
    }
    let req = req.body(Body::from(serde_json::to_vec(body)?))?;
    send(&ctx.router, req).await
}

/// Same as [`post_json`] but lets the caller set additional headers
/// (commonly `x-forwarded-for` for rate-limit tests).
pub async fn post_json_with_headers(
    ctx: &TestCtx,
    path: &str,
    body: &Value,
    extra: &[(HeaderName, String)],
) -> anyhow::Result<(StatusCode, Value, Vec<SetCookie>)> {
    let mut req = Request::builder()
        .method(Method::POST)
        .uri(path)
        .header(header::CONTENT_TYPE, "application/json");
    for (k, v) in tenant_headers(ctx.app.id, &ctx.public_key) {
        req = req.header(k, v);
    }
    for (k, v) in extra {
        req = req.header(k.clone(), v);
    }
    let req = req.body(Body::from(serde_json::to_vec(body)?))?;
    send(&ctx.router, req).await
}

/// POST JSON targeting an explicitly-chosen tenant (for cross-app tests).
pub async fn post_json_as_tenant(
    router: &Router,
    app_id: Uuid,
    public_key: &str,
    path: &str,
    body: &Value,
) -> anyhow::Result<(StatusCode, Value, Vec<SetCookie>)> {
    let mut req = Request::builder()
        .method(Method::POST)
        .uri(path)
        .header(header::CONTENT_TYPE, "application/json");
    for (k, v) in tenant_headers(app_id, public_key) {
        req = req.header(k, v);
    }
    let req = req.body(Body::from(serde_json::to_vec(body)?))?;
    send(router, req).await
}

/// GET targeting an explicitly-chosen tenant.
pub async fn get_as_tenant(
    router: &Router,
    app_id: Uuid,
    public_key: &str,
    path: &str,
    extra: &[(HeaderName, String)],
) -> anyhow::Result<(StatusCode, Value, Vec<SetCookie>)> {
    let mut req = Request::builder().method(Method::GET).uri(path);
    for (k, v) in tenant_headers(app_id, public_key) {
        req = req.header(k, v);
    }
    for (k, v) in extra {
        req = req.header(k.clone(), v);
    }
    let req = req.body(Body::empty())?;
    send(router, req).await
}

/// Collapse a `Vec<SetCookie>` (header lines) into a single `Cookie:`
/// request-header value by keeping only the `name=value` prefix of each.
pub fn cookies_to_header(cookies: &[SetCookie]) -> String {
    cookies
        .iter()
        .filter_map(|c| c.split(';').next())
        .map(str::trim)
        .collect::<Vec<_>>()
        .join("; ")
}

/// Pull a specific cookie's raw value by name from a Set-Cookie list.
pub fn cookie_value<'a>(cookies: &'a [SetCookie], name: &str) -> Option<&'a str> {
    for c in cookies {
        let first = c.split(';').next()?.trim();
        if let Some(v) = first.strip_prefix(&format!("{name}=")) {
            return Some(v);
        }
    }
    None
}
