use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct OrgRow {
    pub id: Uuid,
    pub app_id: Uuid,
    pub slug: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

pub async fn create_org(
    pool: &sqlx::PgPool,
    id: Uuid,
    app_id: Uuid,
    slug: &str,
    name: &str,
) -> Result<OrgRow, sqlx::Error> {
    sqlx::query_as::<_, OrgRow>(
        r#"insert into organizations (id, app_id, slug, name) values ($1,$2,$3,$4)
     returning id, app_id, slug, name, created_at"#,
    )
    .bind(id)
    .bind(app_id)
    .bind(slug)
    .bind(name)
    .fetch_one(pool)
    .await
}

pub async fn list_orgs_for_user(
    pool: &sqlx::PgPool,
    user_id: Uuid,
) -> Result<Vec<OrgRow>, sqlx::Error> {
    sqlx::query_as::<_, OrgRow>(
        r#"select o.id, o.app_id, o.slug, o.name, o.created_at from organizations o
     inner join memberships m on m.org_id = o.id where m.user_id = $1 order by o.name"#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

pub async fn get_by_id(pool: &sqlx::PgPool, id: Uuid) -> Result<Option<OrgRow>, sqlx::Error> {
    sqlx::query_as::<_, OrgRow>(
        r#"select id, app_id, slug, name, created_at from organizations where id = $1"#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}
