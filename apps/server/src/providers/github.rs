use async_trait::async_trait;

use super::OAuthProvider;

pub struct GithubOAuth;

#[async_trait]
impl OAuthProvider for GithubOAuth {
    fn provider_id(&self) -> &'static str {
        "github"
    }

    fn enabled(&self) -> bool {
        false
    }
}
