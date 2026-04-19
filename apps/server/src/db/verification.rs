use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct VerificationRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub kind: String,
    pub token_hash: Vec<u8>,
    pub expires_at: DateTime<Utc>,
    pub used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    #[sqlx(default)]
    pub metadata: Value,
}

pub async fn insert_token(
    pool: &sqlx::PgPool,
    id: Uuid,
    user_id: Uuid,
    kind: &str,
    token_hash: &[u8],
    expires_at: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"insert into verification_tokens (id, user_id, kind, token_hash, expires_at) values ($1,$2,$3,$4,$5)"#,
    )
    .bind(id)
    .bind(user_id)
    .bind(kind)
    .bind(token_hash)
    .bind(expires_at)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn insert_token_with_metadata(
    pool: &sqlx::PgPool,
    id: Uuid,
    user_id: Uuid,
    kind: &str,
    token_hash: &[u8],
    expires_at: DateTime<Utc>,
    metadata: &Value,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"insert into verification_tokens (id, user_id, kind, token_hash, expires_at, metadata)
           values ($1,$2,$3,$4,$5,$6)"#,
    )
    .bind(id)
    .bind(user_id)
    .bind(kind)
    .bind(token_hash)
    .bind(expires_at)
    .bind(metadata)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn take_by_hash(
    pool: &sqlx::PgPool,
    kind: &str,
    token_hash: &[u8],
) -> Result<Option<VerificationRow>, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let row = sqlx::query_as::<_, VerificationRow>(
        r#"select id, user_id, kind, token_hash, expires_at, used_at, created_at,
                  coalesce(metadata, '{}'::jsonb) as metadata
           from verification_tokens
           where kind = $1 and token_hash = $2 and used_at is null and expires_at > now() for update"#,
    )
    .bind(kind)
    .bind(token_hash)
    .fetch_optional(&mut *tx)
    .await?;
    if let Some(ref r) = row {
        sqlx::query(r#"update verification_tokens set used_at = now() where id = $1"#)
            .bind(r.id)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;
    Ok(row)
}
