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
