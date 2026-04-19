use async_trait::async_trait;

use super::OAuthProvider;

pub struct MicrosoftOAuth;

#[async_trait]
impl OAuthProvider for MicrosoftOAuth {
    fn provider_id(&self) -> &'static str {
        "microsoft"
    }

    fn enabled(&self) -> bool {
        false
    }
}
