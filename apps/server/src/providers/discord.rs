//! Discord OAuth2 + `/users/@me`.

use serde::Deserialize;

use crate::error::ApiError;

use super::{expires_at_from_in, ProviderCreds, ProviderProfile, TokenResponse};

const AUTHORIZE: &str = "https://discord.com/oauth2/authorize";
const TOKEN: &str = "https://discord.com/api/oauth2/token";
const USER: &str = "https://discord.com/api/users/@me";
const DEFAULT_SCOPE: &str = "identify email";

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
            ("prompt", "consent"),
        ],
    )
}

#[derive(Debug, Deserialize)]
struct DcUser {
    id: String,
    #[serde(default)]
    email: Option<String>,
    #[serde(default)]
    verified: Option<bool>,
    #[serde(default)]
    username: Option<String>,
    #[serde(default)]
    global_name: Option<String>,
    #[serde(default)]
    avatar: Option<String>,
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
            tracing::warn!(error = %e, "discord token exchange failed");
            ApiError::BadRequest("oauth token exchange failed".into())
        })?
        .error_for_status()
        .map_err(|_| ApiError::BadRequest("oauth token exchange failed".into()))?
        .json()
        .await
        .map_err(|_| ApiError::Internal)?;

    let user: DcUser = http
        .get(USER)
        .bearer_auth(&token.access_token)
        .send()
        .await
        .map_err(|_| ApiError::Internal)?
        .error_for_status()
        .map_err(|_| ApiError::Internal)?
        .json()
        .await
        .map_err(|_| ApiError::Internal)?;

    let image = match (&user.avatar, &user.id) {
        (Some(a), id) => Some(format!("https://cdn.discordapp.com/avatars/{id}/{a}.png")),
        _ => None,
    };

    Ok(ProviderProfile {
        provider_account: user.id,
        email: user.email,
        email_verified: user.verified.unwrap_or(false),
        name: user.global_name.or(user.username),
        image,
        access_token: token.access_token,
        refresh_token: token.refresh_token,
        id_token: token.id_token,
        scope: token.scope,
        expires_at: expires_at_from_in(token.expires_in),
    })
}
