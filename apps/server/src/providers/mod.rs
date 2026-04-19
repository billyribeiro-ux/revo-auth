//! OAuth2 provider adapters.

pub mod apple;
pub mod discord;
pub mod github;
pub mod google;
pub mod microsoft;

use async_trait::async_trait;

#[async_trait]
pub trait OAuthProvider: Send + Sync {
    fn provider_id(&self) -> &'static str;
    fn enabled(&self) -> bool;
}
