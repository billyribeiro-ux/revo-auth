//! GitHub OAuth2 + `/user` API.
//!
//! GitHub's primary `/user` endpoint does not always return a verified email,
//! so we also hit `/user/emails` to find the primary verified address.

use serde::Deserialize;

use crate::error::ApiError;

use super::{expires_at_from_in, ProviderCreds, ProviderProfile, TokenResponse};

const AUTHORIZE: &str = "https://github.com/login/oauth/authorize";
const TOKEN: &str = "https://github.com/login/oauth/access_token";
const USER: &str = "https://api.github.com/user";
const USER_EMAILS: &str = "https://api.github.com/user/emails";
const DEFAULT_SCOPE: &str = "read:user user:email";
const UA: &str = "revo-auth/1.0";

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
        ],
    )
}

#[derive(Debug, Deserialize)]
struct GhUser {
    id: i64,
    #[serde(default)]
    email: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    login: Option<String>,
    #[serde(default)]
    avatar_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GhEmail {
    email: String,
    #[serde(default)]
    primary: bool,
    #[serde(default)]
    verified: bool,
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
        .header(reqwest::header::ACCEPT, "application/json")
        .header(reqwest::header::USER_AGENT, UA)
        .form(&form)
        .send()
        .await
        .map_err(|e| {
            tracing::warn!(error = %e, "github token exchange failed");
            ApiError::BadRequest("oauth token exchange failed".into())
        })?
        .error_for_status()
        .map_err(|_| ApiError::BadRequest("oauth token exchange failed".into()))?
        .json()
        .await
        .map_err(|_| ApiError::Internal)?;

    let user: GhUser = http
        .get(USER)
        .bearer_auth(&token.access_token)
        .header(reqwest::header::USER_AGENT, UA)
        .header(reqwest::header::ACCEPT, "application/vnd.github+json")
        .send()
        .await
        .map_err(|_| ApiError::Internal)?
        .error_for_status()
        .map_err(|_| ApiError::Internal)?
        .json()
        .await
        .map_err(|_| ApiError::Internal)?;

    // Try /user/emails for a verified primary, falling back to /user.email.
    let (email, verified) = match http
        .get(USER_EMAILS)
        .bearer_auth(&token.access_token)
        .header(reqwest::header::USER_AGENT, UA)
        .header(reqwest::header::ACCEPT, "application/vnd.github+json")
        .send()
        .await
    {
        Ok(resp) => match resp.error_for_status() {
            Ok(ok) => match ok.json::<Vec<GhEmail>>().await {
                Ok(list) => list
                    .into_iter()
                    .find(|e| e.primary && e.verified)
                    .map(|e| (Some(e.email), true))
                    .unwrap_or((user.email.clone(), false)),
                Err(_) => (user.email.clone(), false),
            },
            Err(_) => (user.email.clone(), false),
        },
        Err(_) => (user.email.clone(), false),
    };

    Ok(ProviderProfile {
        provider_account: user.id.to_string(),
        email,
        email_verified: verified,
        name: user.name.or(user.login),
        image: user.avatar_url,
        access_token: token.access_token,
        refresh_token: token.refresh_token,
        id_token: token.id_token,
        scope: token.scope,
        expires_at: expires_at_from_in(token.expires_in),
    })
}
