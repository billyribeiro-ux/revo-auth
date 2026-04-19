use async_trait::async_trait;

use super::OAuthProvider;

pub struct GoogleOAuth;

#[async_trait]
impl OAuthProvider for GoogleOAuth {
    fn provider_id(&self) -> &'static str {
        "google"
    }

    fn enabled(&self) -> bool {
        false
    }
}
