use chrono::{DateTime, Utc};
use uuid::Uuid;

pub async fn upsert_unconfirmed(
    pool: &sqlx::PgPool,
    user_id: Uuid,
    secret_enc: &[u8],
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"insert into totp_secrets (user_id, secret_enc) values ($1,$2)
     on conflict (user_id) do update set secret_enc = excluded.secret_enc, confirmed_at = null"#,
    )
    .bind(user_id)
    .bind(secret_enc)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn confirm(pool: &sqlx::PgPool, user_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query(r#"update totp_secrets set confirmed_at = now() where user_id = $1"#)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn get_secret_enc(
    pool: &sqlx::PgPool,
    user_id: Uuid,
) -> Result<Option<Vec<u8>>, sqlx::Error> {
    sqlx::query_scalar(r#"select secret_enc from totp_secrets where user_id = $1"#)
        .bind(user_id)
        .fetch_optional(pool)
        .await
}

pub async fn get_confirmed_at(
    pool: &sqlx::PgPool,
    user_id: Uuid,
) -> Result<Option<DateTime<Utc>>, sqlx::Error> {
    sqlx::query_scalar(r#"select confirmed_at from totp_secrets where user_id = $1"#)
        .bind(user_id)
        .fetch_optional(pool)
        .await
}

pub async fn delete_for_user(pool: &sqlx::PgPool, user_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query(r#"delete from totp_secrets where user_id = $1"#)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
}
