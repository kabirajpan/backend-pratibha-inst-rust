use sqlx::PgExecutor;
use uuid::Uuid;
use crate::errors::AppError;

pub async fn log_activity<'a, E>(
    executor: E,
    user_id: Option<Uuid>,
    action: &str,
    entity_type: &str,
    entity_id: Option<Uuid>,
    meta: Option<serde_json::Value>,
) -> Result<(), AppError>
where
    E: PgExecutor<'a>,
{
    sqlx::query(
        "INSERT INTO activity_log (user_id, action, entity_type, entity_id, meta) VALUES ($1, $2, $3, $4, $5)"
    )
    .bind(user_id)
    .bind(action)
    .bind(entity_type)
    .bind(entity_id)
    .bind(meta.map(sqlx::types::Json))
    .execute(executor)
    .await?;

    Ok(())
}
