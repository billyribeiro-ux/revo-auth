//! Google OAuth2 / OIDC.
//!
//! OIDC discovery is not currently wired in; endpoints are the well-known
//! Google constants. Userinfo is fetched via `reqwest` since the full
//! `openidconnect` crate isn't a dependency.

use serde::Deserialize;

use crate::error::ApiError;

use super::{expires_at_from_in, ProviderCreds, ProviderProfile, TokenResponse};

const AUTHORIZE: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const TOKEN: &str = "https://oauth2.googleapis.com/token";
const USERINFO: &str = "https://openidconnect.googleapis.com/v1/userinfo";
const DEFAULT_SCOPE: &str = "openid email profile";

pub fn authorize_url(creds: &ProviderCreds, state: &str, code_challenge: &str) -> String {
    let scope = creds.scope.as_deref().unwrap_or(DEFAULT_SCOPE);
    super::build_authorize_url(
        AUTHORIZE,
        &[
            ("response_type", "code"),
            ("client_id", &creds.client_id),
            ("redirect_uri", &creds.redirect_uri),
            ("scope", scope),
            ("state", state),
            ("code_challenge", code_challenge),
            ("code_challenge_method", "S256"),
            ("access_type", "offline"),
            ("prompt", "consent"),
        ],
    )
}

#[derive(Debug, Deserialize)]
struct GoogleUser {
    sub: String,
    #[serde(default)]
    email: Option<String>,
    #[serde(default)]
    email_verified: Option<bool>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    picture: Option<String>,
}

pub async fn exchange(
    http: &reqwest::Client,
    creds: &ProviderCreds,
    code: &str,
    verifier: &str,
) -> Result<ProviderProfile, ApiError> {
    let form = [
        ("grant_type", "authorization_code"),
        ("code", code),
        ("redirect_uri", creds.redirect_uri.as_str()),
        ("client_id", creds.client_id.as_str()),
        ("client_secret", creds.client_secret.as_str()),
        ("code_verifier", verifier),
    ];
    let token: TokenResponse = http
        .post(TOKEN)
        .form(&form)
        .send()
        .await
        .map_err(|e| {
            tracing::warn!(error = %e, "google token exchange failed");
            ApiError::BadRequest("oauth token exchange failed".into())
        })?
        .error_for_status()
        .map_err(|e| {
            tracing::warn!(error = %e, "google token non-2xx");
            ApiError::BadRequest("oauth token exchange failed".into())
        })?
        .json()
        .await
        .map_err(|_| ApiError::Internal)?;

    let user: GoogleUser = http
        .get(USERINFO)
        .bearer_auth(&token.access_token)
        .send()
        .await
        .map_err(|_| ApiError::Internal)?
        .error_for_status()
        .map_err(|_| ApiError::Internal)?
        .json()
        .await
        .map_err(|_| ApiError::Internal)?;

    Ok(ProviderProfile {
        provider_account: user.sub,
        email: user.email,
        email_verified: user.email_verified.unwrap_or(false),
        name: user.name,
        image: user.picture,
        access_token: token.access_token,
        refresh_token: token.refresh_token,
        id_token: token.id_token,
        scope: token.scope,
        expires_at: expires_at_from_in(token.expires_in),
    })
}
