use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct AccountRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub provider: String,
    pub provider_account: String,
    pub access_token_enc: Option<Vec<u8>>,
    pub refresh_token_enc: Option<Vec<u8>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub scope: Option<String>,
    pub id_token: Option<String>,
    pub created_at: DateTime<Utc>,
}

pub async fn list_for_user(
    pool: &sqlx::PgPool,
    user_id: Uuid,
) -> Result<Vec<AccountRow>, sqlx::Error> {
    sqlx::query_as::<_, AccountRow>(
		r#"select id, user_id, provider, provider_account, access_token_enc, refresh_token_enc,
            expires_at, scope, id_token, created_at from accounts where user_id = $1 order by created_at"#,
	)
	.bind(user_id)
	.fetch_all(pool)
	.await
}

pub async fn find_by_provider_account(
    pool: &sqlx::PgPool,
    provider: &str,
    provider_account: &str,
) -> Result<Option<AccountRow>, sqlx::Error> {
    sqlx::query_as::<_, AccountRow>(
        r#"select id, user_id, provider, provider_account, access_token_enc, refresh_token_enc,
            expires_at, scope, id_token, created_at from accounts
     where provider = $1 and provider_account = $2"#,
    )
    .bind(provider)
    .bind(provider_account)
    .fetch_optional(pool)
    .await
}

pub async fn find_for_user_provider(
    pool: &sqlx::PgPool,
    user_id: Uuid,
    provider: &str,
) -> Result<Option<AccountRow>, sqlx::Error> {
    sqlx::query_as::<_, AccountRow>(
        r#"select id, user_id, provider, provider_account, access_token_enc, refresh_token_enc,
            expires_at, scope, id_token, created_at from accounts
           where user_id = $1 and provider = $2"#,
    )
    .bind(user_id)
    .bind(provider)
    .fetch_optional(pool)
    .await
}

pub async fn delete_for_user_provider(
    pool: &sqlx::PgPool,
    user_id: Uuid,
    provider: &str,
) -> Result<u64, sqlx::Error> {
    let r = sqlx::query(r#"delete from accounts where user_id = $1 and provider = $2"#)
        .bind(user_id)
        .bind(provider)
        .execute(pool)
        .await?;
    Ok(r.rows_affected())
}

pub async fn count_for_user(pool: &sqlx::PgPool, user_id: Uuid) -> Result<i64, sqlx::Error> {
    let row: (i64,) = sqlx::query_as(r#"select count(*) from accounts where user_id = $1"#)
        .bind(user_id)
        .fetch_one(pool)
        .await?;
    Ok(row.0)
}

#[allow(clippy::too_many_arguments)]
pub async fn upsert_account(
    pool: &sqlx::PgPool,
    id: Uuid,
    user_id: Uuid,
    provider: &str,
    provider_account: &str,
    access_token_enc: Option<&[u8]>,
    refresh_token_enc: Option<&[u8]>,
    expires_at: Option<DateTime<Utc>>,
    scope: Option<&str>,
    id_token: Option<&str>,
) -> Result<AccountRow, sqlx::Error> {
    sqlx::query_as::<_, AccountRow>(
        r#"insert into accounts
            (id, user_id, provider, provider_account, access_token_enc, refresh_token_enc,
             expires_at, scope, id_token)
         values ($1,$2,$3,$4,$5,$6,$7,$8,$9)
         on conflict (provider, provider_account) do update set
            user_id = excluded.user_id,
            access_token_enc = excluded.access_token_enc,
            refresh_token_enc = excluded.refresh_token_enc,
            expires_at = excluded.expires_at,
            scope = excluded.scope,
            id_token = excluded.id_token
         returning id, user_id, provider, provider_account, access_token_enc, refresh_token_enc,
            expires_at, scope, id_token, created_at"#,
    )
    .bind(id)
    .bind(user_id)
    .bind(provider)
    .bind(provider_account)
    .bind(access_token_enc)
    .bind(refresh_token_enc)
    .bind(expires_at)
    .bind(scope)
    .bind(id_token)
    .fetch_one(pool)
    .await
}
