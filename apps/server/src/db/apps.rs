use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct AppRow {
    pub id: Uuid,
    pub slug: String,
    pub name: String,
    pub origins: Vec<String>,
    pub public_key: String,
    pub secret_key_hash: String,
    pub settings: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub async fn get_by_id(pool: &sqlx::PgPool, id: Uuid) -> Result<Option<AppRow>, sqlx::Error> {
    sqlx::query_as::<_, AppRow>(
		r#"select id, slug, name, origins, public_key, secret_key_hash, settings, created_at, updated_at from apps where id = $1"#,
	)
	.bind(id)
	.fetch_optional(pool)
	.await
}

pub async fn get_by_public_key(
    pool: &sqlx::PgPool,
    id: Uuid,
    public_key: &str,
) -> Result<Option<AppRow>, sqlx::Error> {
    sqlx::query_as::<_, AppRow>(
		r#"select id, slug, name, origins, public_key, secret_key_hash, settings, created_at, updated_at from apps where id = $1 and public_key = $2"#,
	)
	.bind(id)
	.bind(public_key)
	.fetch_optional(pool)
	.await
}

pub async fn list_all(pool: &sqlx::PgPool) -> Result<Vec<AppRow>, sqlx::Error> {
    sqlx::query_as::<_, AppRow>(
		r#"select id, slug, name, origins, public_key, secret_key_hash, settings, created_at, updated_at from apps order by created_at"#,
	)
	.fetch_all(pool)
	.await
}

pub async fn insert_app(
    pool: &sqlx::PgPool,
    id: Uuid,
    slug: &str,
    name: &str,
    origins: &[String],
    public_key: &str,
    secret_key_hash: &str,
) -> Result<AppRow, sqlx::Error> {
    sqlx::query_as::<_, AppRow>(
		r#"insert into apps (id, slug, name, origins, public_key, secret_key_hash)
     values ($1,$2,$3,$4,$5,$6)
     returning id, slug, name, origins, public_key, secret_key_hash, settings, created_at, updated_at"#,
	)
	.bind(id)
	.bind(slug)
	.bind(name)
	.bind(origins)
	.bind(public_key)
	.bind(secret_key_hash)
	.fetch_one(pool)
	.await
}

pub async fn update_app(
    pool: &sqlx::PgPool,
    id: Uuid,
    name: Option<&str>,
    origins: Option<&[String]>,
) -> Result<Option<AppRow>, sqlx::Error> {
    if name.is_none() && origins.is_none() {
        return get_by_id(pool, id).await;
    }
    let row = sqlx::query_as::<_, AppRow>(
		r#"update apps set
      name = coalesce($2, name),
      origins = coalesce($3, origins),
      updated_at = now()
    where id = $1
    returning id, slug, name, origins, public_key, secret_key_hash, settings, created_at, updated_at"#,
	)
	.bind(id)
	.bind(name)
	.bind(origins)
	.fetch_optional(pool)
	.await?;
    Ok(row)
}
