//! Microsoft Identity Platform v2 (OIDC).

use serde::Deserialize;

use crate::error::ApiError;

use super::{expires_at_from_in, ProviderCreds, ProviderProfile, TokenResponse};

const AUTHORIZE: &str = "https://login.microsoftonline.com/common/oauth2/v2.0/authorize";
const TOKEN: &str = "https://login.microsoftonline.com/common/oauth2/v2.0/token";
const USERINFO: &str = "https://graph.microsoft.com/oidc/userinfo";
const DEFAULT_SCOPE: &str = "openid email profile offline_access";

pub fn authorize_url(creds: &ProviderCreds, state: &str, code_challenge: &str) -> String {
    let scope = creds.scope.as_deref().unwrap_or(DEFAULT_SCOPE);
    super::build_authorize_url(
        AUTHORIZE,
        &[
            ("response_type", "code"),
            ("response_mode", "query"),
            ("client_id", &creds.client_id),
            ("redirect_uri", &creds.redirect_uri),
            ("scope", scope),
            ("state", state),
            ("code_challenge", code_challenge),
            ("code_challenge_method", "S256"),
        ],
    )
}

#[derive(Debug, Deserialize)]
struct MsUser {
    sub: String,
    #[serde(default)]
    email: Option<String>,
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
            tracing::warn!(error = %e, "microsoft token exchange failed");
            ApiError::BadRequest("oauth token exchange failed".into())
        })?
        .error_for_status()
        .map_err(|_| ApiError::BadRequest("oauth token exchange failed".into()))?
        .json()
        .await
        .map_err(|_| ApiError::Internal)?;

    let user: MsUser = http
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

    // Microsoft does not include `email_verified`; assume verified because the
    // user successfully authenticated against a Microsoft account tenant.
    Ok(ProviderProfile {
        provider_account: user.sub,
        email: user.email,
        email_verified: true,
        name: user.name,
        image: user.picture,
        access_token: token.access_token,
        refresh_token: token.refresh_token,
        id_token: token.id_token,
        scope: token.scope,
        expires_at: expires_at_from_in(token.expires_in),
    })
}
