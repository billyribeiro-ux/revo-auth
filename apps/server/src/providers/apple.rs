//! Apple Sign-In — scaffolded but disabled.
//!
//! Apple requires a JWT client-secret signed with an ES256 developer key and
//! additional attestation handling. The adapter is kept for API shape parity;
//! `Provider::Apple.enabled()` is `false` and `authorize`/`callback` paths
//! return `ApiError::NotFound` before reaching this module.

use crate::error::ApiError;

use super::{ProviderCreds, ProviderProfile};

#[allow(dead_code)]
pub fn authorize_url(_creds: &ProviderCreds, _state: &str, _code_challenge: &str) -> String {
    String::new()
}

#[allow(dead_code)]
pub async fn exchange(
    _http: &reqwest::Client,
    _creds: &ProviderCreds,
    _code: &str,
    _verifier: &str,
) -> Result<ProviderProfile, ApiError> {
    Err(ApiError::NotFound)
}
