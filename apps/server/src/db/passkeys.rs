use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct PasskeyRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub credential_id: Vec<u8>,
    pub public_key: Vec<u8>,
    pub sign_count: i64,
    pub transports: Vec<String>,
    pub backup_eligible: bool,
    pub backup_state: bool,
    pub friendly_name: Option<String>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

pub async fn list_for_user(
    pool: &sqlx::PgPool,
    user_id: Uuid,
) -> Result<Vec<PasskeyRow>, sqlx::Error> {
    sqlx::query_as::<_, PasskeyRow>(
		r#"select id, user_id, credential_id, public_key, sign_count, transports, backup_eligible, backup_state, friendly_name, last_used_at, created_at from passkeys where user_id = $1 order by created_at"#,
	)
	.bind(user_id)
	.fetch_all(pool)
	.await
}

pub async fn find_by_credential(
    pool: &sqlx::PgPool,
    credential_id: &[u8],
) -> Result<Option<PasskeyRow>, sqlx::Error> {
    sqlx::query_as::<_, PasskeyRow>(
		r#"select id, user_id, credential_id, public_key, sign_count, transports, backup_eligible, backup_state, friendly_name, last_used_at, created_at from passkeys where credential_id = $1"#,
	)
	.bind(credential_id)
	.fetch_optional(pool)
	.await
}

#[allow(clippy::too_many_arguments)]
pub async fn insert_passkey(
    pool: &sqlx::PgPool,
    id: Uuid,
    user_id: Uuid,
    credential_id: &[u8],
    public_key: &[u8],
    transports: &[String],
    backup_eligible: bool,
    backup_state: bool,
    name: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
		r#"insert into passkeys (id, user_id, credential_id, public_key, transports, backup_eligible, backup_state, friendly_name)
     values ($1,$2,$3,$4,$5,$6,$7,$8)"#,
	)
	.bind(id)
	.bind(user_id)
	.bind(credential_id)
	.bind(public_key)
	.bind(transports)
	.bind(backup_eligible)
	.bind(backup_state)
	.bind(name)
	.execute(pool)
	.await?;
    Ok(())
}

pub async fn delete_passkey(
    pool: &sqlx::PgPool,
    id: Uuid,
    user_id: Uuid,
) -> Result<u64, sqlx::Error> {
    let r = sqlx::query(r#"delete from passkeys where id = $1 and user_id = $2"#)
        .bind(id)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(r.rows_affected())
}

pub async fn update_sign_count(
    pool: &sqlx::PgPool,
    id: Uuid,
    count: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(r#"update passkeys set sign_count = $2, last_used_at = now() where id = $1"#)
        .bind(id)
        .bind(count)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn update_passkey_after_auth(
    pool: &sqlx::PgPool,
    credential_id: &[u8],
    sign_count: i64,
    backup_state: bool,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"update passkeys
           set sign_count = $2, backup_state = $3, last_used_at = now()
           where credential_id = $1"#,
    )
    .bind(credential_id)
    .bind(sign_count)
    .bind(backup_state)
    .execute(pool)
    .await?;
    Ok(())
}
