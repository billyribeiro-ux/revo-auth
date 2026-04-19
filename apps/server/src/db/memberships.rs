use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct MembershipRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub user_id: Uuid,
    pub role: String,
    pub created_at: DateTime<Utc>,
}

pub async fn add_member(
    pool: &sqlx::PgPool,
    id: Uuid,
    org_id: Uuid,
    user_id: Uuid,
    role: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(r#"insert into memberships (id, org_id, user_id, role) values ($1,$2,$3,$4)"#)
        .bind(id)
        .bind(org_id)
        .bind(user_id)
        .bind(role)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn get_membership(
    pool: &sqlx::PgPool,
    org_id: Uuid,
    user_id: Uuid,
) -> Result<Option<MembershipRow>, sqlx::Error> {
    sqlx::query_as::<_, MembershipRow>(
		r#"select id, org_id, user_id, role, created_at from memberships where org_id = $1 and user_id = $2"#,
	)
	.bind(org_id)
	.bind(user_id)
	.fetch_optional(pool)
	.await
}

pub async fn role_of(
    pool: &sqlx::PgPool,
    org_id: Uuid,
    user_id: Uuid,
) -> Result<Option<String>, sqlx::Error> {
    let row: Option<(String,)> = sqlx::query_as(
        r#"select role from memberships where org_id = $1 and user_id = $2"#,
    )
    .bind(org_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|r| r.0))
}
