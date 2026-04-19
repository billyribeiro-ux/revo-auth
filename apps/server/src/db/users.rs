use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct UserRow {
    pub id: Uuid,
    pub app_id: Uuid,
    pub email: Option<String>,
    pub email_verified_at: Option<DateTime<Utc>>,
    pub password_hash: Option<String>,
    pub name: Option<String>,
    pub image_url: Option<String>,
    pub custom_fields: Value,
    pub banned_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub async fn find_by_email(
    pool: &sqlx::PgPool,
    app_id: Uuid,
    email: &str,
) -> Result<Option<UserRow>, sqlx::Error> {
    sqlx::query_as::<_, UserRow>(
		r#"select id, app_id, email, email_verified_at, password_hash, name, image_url, custom_fields, banned_at, created_at, updated_at
     from users where app_id = $1 and email = $2::citext"#,
	)
	.bind(app_id)
	.bind(email)
	.fetch_optional(pool)
	.await
}

pub async fn get_by_id(pool: &sqlx::PgPool, id: Uuid) -> Result<Option<UserRow>, sqlx::Error> {
    sqlx::query_as::<_, UserRow>(
		r#"select id, app_id, email, email_verified_at, password_hash, name, image_url, custom_fields, banned_at, created_at, updated_at from users where id = $1"#,
	)
	.bind(id)
	.fetch_optional(pool)
	.await
}

pub async fn insert_user(
    pool: &sqlx::PgPool,
    id: Uuid,
    app_id: Uuid,
    email: &str,
    password_hash: Option<&str>,
    name: Option<&str>,
) -> Result<UserRow, sqlx::Error> {
    sqlx::query_as::<_, UserRow>(
		r#"insert into users (id, app_id, email, password_hash, name)
     values ($1,$2,$3::citext,$4,$5)
     returning id, app_id, email, email_verified_at, password_hash, name, image_url, custom_fields, banned_at, created_at, updated_at"#,
	)
	.bind(id)
	.bind(app_id)
	.bind(email)
	.bind(password_hash)
	.bind(name)
	.fetch_one(pool)
	.await
}

pub async fn set_password_hash(
    pool: &sqlx::PgPool,
    user_id: Uuid,
    hash: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(r#"update users set password_hash = $2, updated_at = now() where id = $1"#)
        .bind(user_id)
        .bind(hash)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn verify_email(pool: &sqlx::PgPool, user_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query(r#"update users set email_verified_at = now(), updated_at = now() where id = $1"#)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
}
