//! OAuth2 provider adapters.
//!
//! Each provider exposes a small, enum-dispatched surface:
//! * `authorize_url` — build the provider's redirect URL (S256 PKCE).
//! * `exchange` — exchange `code` + `verifier` for tokens and fetch the
//!   normalised `ProviderProfile`.
//!
//! Provider client credentials are read from the tenant's `settings.oauth.<id>`
//! JSON blob, e.g.:
//!
//! ```json
//! {
//!   "oauth": {
//!     "google": { "client_id": "...", "client_secret": "...", "redirect_uri": "https://..." }
//!   }
//! }
//! ```

pub mod apple;
pub mod discord;
pub mod github;
pub mod google;
pub mod microsoft;

use serde::Deserialize;
use serde_json::Value;

use crate::db::AppRow;
use crate::error::ApiError;

/// Normalised user profile returned by every provider adapter.
#[derive(Debug, Clone)]
pub struct ProviderProfile {
    pub provider_account: String,
    pub email: Option<String>,
    pub email_verified: bool,
    pub name: Option<String>,
    pub image: Option<String>,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub id_token: Option<String>,
    pub scope: Option<String>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Credentials loaded from `apps.settings` for a given provider.
#[derive(Debug, Clone, Deserialize)]
pub struct ProviderCreds {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    #[serde(default)]
    pub scope: Option<String>,
}

impl ProviderCreds {
    pub fn from_app(app: &AppRow, provider: &str) -> Result<Self, ApiError> {
        let oauth = app.settings.get("oauth").and_then(Value::as_object);
        let Some(oauth) = oauth else {
            return Err(ApiError::NotFound);
        };
        let Some(v) = oauth.get(provider) else {
            return Err(ApiError::NotFound);
        };
        serde_json::from_value::<ProviderCreds>(v.clone())
            .map_err(|e| ApiError::Validation(format!("invalid provider config: {e}")))
    }
}

/// Enum-dispatched provider selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Provider {
    Google,
    Github,
    Microsoft,
    Discord,
    Apple,
}

impl Provider {
    pub fn parse(id: &str) -> Option<Self> {
        match id {
            "google" => Some(Self::Google),
            "github" => Some(Self::Github),
            "microsoft" => Some(Self::Microsoft),
            "discord" => Some(Self::Discord),
            "apple" => Some(Self::Apple),
            _ => None,
        }
    }

    pub fn id(&self) -> &'static str {
        match self {
            Self::Google => "google",
            Self::Github => "github",
            Self::Microsoft => "microsoft",
            Self::Discord => "discord",
            Self::Apple => "apple",
        }
    }

    pub fn enabled(&self) -> bool {
        !matches!(self, Self::Apple)
    }

    /// Build the provider's authorize URL including PKCE (S256) + state.
    pub fn authorize_url(
        &self,
        creds: &ProviderCreds,
        state: &str,
        code_challenge: &str,
    ) -> String {
        match self {
            Self::Google => google::authorize_url(creds, state, code_challenge),
            Self::Github => github::authorize_url(creds, state, code_challenge),
            Self::Microsoft => microsoft::authorize_url(creds, state, code_challenge),
            Self::Discord => discord::authorize_url(creds, state, code_challenge),
            Self::Apple => String::new(),
        }
    }

    /// Exchange `code` (with PKCE verifier) for tokens, then fetch userinfo.
    pub async fn exchange(
        &self,
        http: &reqwest::Client,
        creds: &ProviderCreds,
        code: &str,
        code_verifier: &str,
    ) -> Result<ProviderProfile, ApiError> {
        match self {
            Self::Google => google::exchange(http, creds, code, code_verifier).await,
            Self::Github => github::exchange(http, creds, code, code_verifier).await,
            Self::Microsoft => microsoft::exchange(http, creds, code, code_verifier).await,
            Self::Discord => discord::exchange(http, creds, code, code_verifier).await,
            Self::Apple => Err(ApiError::NotFound),
        }
    }
}

/// OAuth2 standard token response.
#[derive(Debug, Deserialize)]
pub(crate) struct TokenResponse {
    pub access_token: String,
    #[serde(default)]
    pub refresh_token: Option<String>,
    #[serde(default)]
    pub id_token: Option<String>,
    #[serde(default)]
    pub scope: Option<String>,
    #[serde(default)]
    pub expires_in: Option<i64>,
}

pub(crate) fn expires_at_from_in(
    expires_in: Option<i64>,
) -> Option<chrono::DateTime<chrono::Utc>> {
    expires_in.map(|s| chrono::Utc::now() + chrono::Duration::seconds(s))
}

/// Percent-encode + concatenate query parameters without panicking.
pub(crate) fn build_authorize_url(base: &str, params: &[(&str, &str)]) -> String {
    use std::fmt::Write as _;
    let mut out = String::with_capacity(base.len() + 64);
    out.push_str(base);
    let mut first = !base.contains('?');
    for (k, v) in params {
        if first {
            out.push('?');
            first = false;
        } else {
            out.push('&');
        }
        // `write!` into a `String` never fails — but we swallow the Result
        // rather than unwrap, for strict safety.
        let _ = write!(out, "{}={}", urlencoding::encode(k), urlencoding::encode(v));
    }
    out
}
