use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct MagicLinkRow {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub email: String,
    pub app_id: Uuid,
    pub token_hash: Vec<u8>,
    pub expires_at: DateTime<Utc>,
    pub used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

pub async fn insert(
    pool: &sqlx::PgPool,
    id: Uuid,
    user_id: Option<Uuid>,
    email: &str,
    app_id: Uuid,
    token_hash: &[u8],
    expires_at: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"insert into magic_links (id, user_id, email, app_id, token_hash, expires_at)
     values ($1,$2,$3::citext,$4,$5,$6)"#,
    )
    .bind(id)
    .bind(user_id)
    .bind(email)
    .bind(app_id)
    .bind(token_hash)
    .bind(expires_at)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn take_by_hash(
    pool: &sqlx::PgPool,
    app_id: Uuid,
    token_hash: &[u8],
) -> Result<Option<MagicLinkRow>, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let row = sqlx::query_as::<_, MagicLinkRow>(
		r#"select id, user_id, email, app_id, token_hash, expires_at, used_at, created_at from magic_links
     where app_id = $1 and token_hash = $2 and used_at is null and expires_at > now() for update"#,
	)
	.bind(app_id)
	.bind(token_hash)
	.fetch_optional(&mut *tx)
	.await?;
    if let Some(ref r) = row {
        sqlx::query(r#"update magic_links set used_at = now() where id = $1"#)
            .bind(r.id)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;
    Ok(row)
}
