use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct SessionRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: Vec<u8>,
    pub user_agent: Option<String>,
    pub ip: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub last_used_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
}

pub async fn insert_session(
    pool: &sqlx::PgPool,
    id: Uuid,
    user_id: Uuid,
    token_hash: &[u8],
    user_agent: Option<&str>,
    ip: Option<&str>,
    expires_at: DateTime<Utc>,
) -> Result<SessionRow, sqlx::Error> {
    sqlx::query_as::<_, SessionRow>(
		r#"insert into sessions (id, user_id, token_hash, user_agent, ip, expires_at)
     values ($1,$2,$3,$4,$5::inet,$6)
     returning id, user_id, token_hash, user_agent, ip::text as ip, expires_at, created_at, last_used_at, revoked_at"#,
	)
	.bind(id)
	.bind(user_id)
	.bind(token_hash)
	.bind(user_agent)
	.bind(ip)
	.bind(expires_at)
	.fetch_one(pool)
	.await
}

pub async fn find_by_token_hash(
    pool: &sqlx::PgPool,
    token_hash: &[u8],
) -> Result<Option<SessionRow>, sqlx::Error> {
    sqlx::query_as::<_, SessionRow>(
		r#"select id, user_id, token_hash, user_agent, ip::text as ip, expires_at, created_at, last_used_at, revoked_at from sessions where token_hash = $1 and revoked_at is null"#,
	)
	.bind(token_hash)
	.fetch_optional(pool)
	.await
}

pub async fn revoke(pool: &sqlx::PgPool, id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query(r#"update sessions set revoked_at = now() where id = $1"#)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn list_for_user(
    pool: &sqlx::PgPool,
    user_id: Uuid,
) -> Result<Vec<SessionRow>, sqlx::Error> {
    sqlx::query_as::<_, SessionRow>(
		r#"select id, user_id, token_hash, user_agent, ip::text as ip, expires_at, created_at, last_used_at, revoked_at from sessions where user_id = $1 and revoked_at is null order by created_at desc"#,
	)
	.bind(user_id)
	.fetch_all(pool)
	.await
}

pub async fn touch(pool: &sqlx::PgPool, id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query(r#"update sessions set last_used_at = now() where id = $1"#)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn revoke_all_for_user(
    pool: &sqlx::PgPool,
    user_id: Uuid,
) -> Result<Vec<SessionRow>, sqlx::Error> {
    sqlx::query_as::<_, SessionRow>(
        r#"update sessions set revoked_at = now()
           where user_id = $1 and revoked_at is null
           returning id, user_id, token_hash, user_agent, ip::text as ip,
                     expires_at, created_at, last_used_at, revoked_at"#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}
