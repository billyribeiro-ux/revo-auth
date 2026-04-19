use serde_json::Value;
use uuid::Uuid;

pub async fn log_event(
    pool: &sqlx::PgPool,
    app_id: Option<Uuid>,
    user_id: Option<Uuid>,
    event: &str,
    meta: Value,
    ip: Option<&str>,
    user_agent: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"insert into audit_log (app_id, user_id, event, meta, ip, user_agent)
     values ($1,$2,$3,$4,$5::inet,$6)"#,
    )
    .bind(app_id)
    .bind(user_id)
    .bind(event)
    .bind(meta)
    .bind(ip)
    .bind(user_agent)
    .execute(pool)
    .await?;
    Ok(())
}
