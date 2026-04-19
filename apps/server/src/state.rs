use std::sync::Arc;

use fred::prelude::*;
use sqlx::PgPool;
use uuid::Uuid;

use crate::config::Config;
use crate::email::transport::EmailTransport;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub redis: RedisPool,
    pub config: Arc<Config>,
    pub mail: Arc<dyn EmailTransport>,
}

impl AppState {
    pub async fn connect_redis(redis_url: &str) -> Result<RedisPool, fred::error::RedisError> {
        let cfg = RedisConfig::from_url(redis_url)?;
        let pool = Builder::from_config(cfg).build_pool(4)?;
        pool.init().await?;
        Ok(pool)
    }

    pub async fn mark_session_revoked(
        &self,
        session_id: Uuid,
        ttl_secs: u64,
    ) -> Result<(), RedisError> {
        let key = format!("revo:revoked:{session_id}");
        self.redis
            .set::<(), _, _>(key, "1", Some(Expiration::EX(ttl_secs as i64)), None, false)
            .await?;
        Ok(())
    }

    pub async fn is_session_revoked(&self, session_id: Uuid) -> Result<bool, RedisError> {
        let key = format!("revo:revoked:{session_id}");
        let v: Option<String> = self.redis.get(key).await?;
        Ok(v.is_some())
    }

    pub async fn cache_set(&self, key: &str, val: &str, ttl_secs: u64) -> Result<(), RedisError> {
        self.redis
            .set::<(), _, _>(key, val, Some(Expiration::EX(ttl_secs as i64)), None, false)
            .await?;
        Ok(())
    }

    pub async fn cache_get(&self, key: &str) -> Result<Option<String>, RedisError> {
        let v: Option<String> = self.redis.get(key).await?;
        Ok(v)
    }

    pub async fn cache_del(&self, key: &str) -> Result<(), RedisError> {
        self.redis.del::<(), _>(key).await?;
        Ok(())
    }
}
