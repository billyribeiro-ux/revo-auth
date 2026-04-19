use async_trait::async_trait;

use super::OAuthProvider;

pub struct DiscordOAuth;

#[async_trait]
impl OAuthProvider for DiscordOAuth {
    fn provider_id(&self) -> &'static str {
        "discord"
    }

    fn enabled(&self) -> bool {
        false
    }
}
