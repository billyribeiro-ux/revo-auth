use async_trait::async_trait;

use super::OAuthProvider;

pub struct AppleOAuth;

#[async_trait]
impl OAuthProvider for AppleOAuth {
    fn provider_id(&self) -> &'static str {
        "apple"
    }

    fn enabled(&self) -> bool {
        false
    }
}
