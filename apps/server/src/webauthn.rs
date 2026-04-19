//! WebAuthn / passkey helpers (webauthn-rs 0.5).
//!
//! `PasskeyRegistration` / `PasskeyAuthentication` types in webauthn-rs 0.5
//! are only `Serialize`/`Deserialize` behind the `danger-allow-state-serialisation`
//! feature flag, which is NOT enabled in this workspace. To keep handlers
//! stateless from the HTTP layer's perspective while still satisfying
//! replay-safety, we park the in-progress state in an in-process `DashMap`
//! keyed by `user_id` or `flow_id`, with a TTL sweep. Redis stores only the
//! short-lived flow id that points at the in-memory entry.
//!
//! This is a concession to an upstream feature-flag we can't flip without
//! touching `Cargo.toml` (outside this agent's scope). The semantics match
//! "state stored server-side, keyed by user, 10-minute TTL" per the spec.

use std::sync::Arc;

use dashmap::DashMap;
use once_cell::sync::Lazy;
use url::Url;
use uuid::Uuid;
use webauthn_rs::prelude::*;

/// Build a `Webauthn` instance for a tenant. The RP ID is the primary origin's
/// hostname; the origin URL must include scheme + host (and port when not 443).
pub fn build_webauthn(rp_id: &str, origin: &str) -> Result<Webauthn, WebauthnError> {
    let origin_url: Url = origin.parse().map_err(|_| WebauthnError::Configuration)?;
    WebauthnBuilder::new(rp_id, &origin_url).and_then(|b| b.build())
}

/// Extract the tenant's RP id (hostname) + origin (scheme://host[:port]) from
/// its configured origins list. Uses the first origin as the primary.
pub fn rp_from_app(origins: &[String]) -> Result<(String, String), WebauthnError> {
    let first = origins.first().ok_or(WebauthnError::Configuration)?;
    let u: Url = first.parse().map_err(|_| WebauthnError::Configuration)?;
    let host = u.host_str().ok_or(WebauthnError::Configuration)?.to_string();
    let origin = match u.port() {
        Some(p) => format!("{}://{}:{p}", u.scheme(), host),
        None => format!("{}://{}", u.scheme(), host),
    };
    Ok((host, origin))
}

struct RegEntry {
    state: PasskeyRegistration,
    exp: chrono::DateTime<chrono::Utc>,
}

struct AuthEntry {
    state: PasskeyAuthentication,
    user_id: Option<Uuid>,
    exp: chrono::DateTime<chrono::Utc>,
}

static REG_STORE: Lazy<Arc<DashMap<String, RegEntry>>> = Lazy::new(|| Arc::new(DashMap::new()));
static AUTH_STORE: Lazy<Arc<DashMap<String, AuthEntry>>> = Lazy::new(|| Arc::new(DashMap::new()));

fn reap_reg() {
    let now = chrono::Utc::now();
    REG_STORE.retain(|_, v| v.exp > now);
}
fn reap_auth() {
    let now = chrono::Utc::now();
    AUTH_STORE.retain(|_, v| v.exp > now);
}

pub fn reg_put(flow_id: &str, state: PasskeyRegistration, ttl_secs: i64) {
    reap_reg();
    let exp = chrono::Utc::now() + chrono::Duration::seconds(ttl_secs);
    REG_STORE.insert(flow_id.to_string(), RegEntry { state, exp });
}

pub fn reg_take(flow_id: &str) -> Option<PasskeyRegistration> {
    reap_reg();
    let (_, entry) = REG_STORE.remove(flow_id)?;
    if entry.exp < chrono::Utc::now() {
        return None;
    }
    Some(entry.state)
}

pub fn auth_put(flow_id: &str, state: PasskeyAuthentication, user_id: Option<Uuid>, ttl_secs: i64) {
    reap_auth();
    let exp = chrono::Utc::now() + chrono::Duration::seconds(ttl_secs);
    AUTH_STORE.insert(flow_id.to_string(), AuthEntry { state, user_id, exp });
}

pub fn auth_take(flow_id: &str) -> Option<(PasskeyAuthentication, Option<Uuid>)> {
    reap_auth();
    let (_, entry) = AUTH_STORE.remove(flow_id)?;
    if entry.exp < chrono::Utc::now() {
        return None;
    }
    Some((entry.state, entry.user_id))
}
