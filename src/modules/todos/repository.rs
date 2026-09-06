// backend-rust/src/modules/todos/repository.rs
//! Todos Data Access Layer (Repository)

use sqlx::PgPool;
use uuid::Uuid;
use super::models::Todo;

/// Fetch all todos for a given module ordered by created_at DESC
pub async fn find_by_module(pool: &PgPool, module: &str) -> Result<Vec<Todo>, sqlx::Error> {
    sqlx::query_as::<_, Todo>(
        "SELECT * FROM todos WHERE module = $1 ORDER BY created_at DESC"
    )
    .bind(module)
    .fetch_all(pool)
    .await
}

/// Find a single todo by id and module
pub async fn find_by_id_and_module(pool: &PgPool, id: Uuid, module: &str) -> Result<Option<Todo>, sqlx::Error> {
    sqlx::query_as::<_, Todo>(
        "SELECT * FROM todos WHERE id = $1 AND module = $2"
    )
    .bind(id)
    .bind(module)
    .fetch_optional(pool)
    .await
}

/// Insert a new todo record
pub async fn create(
    pool: &PgPool,
    user_id: Option<Uuid>,
    module: &str,
    text: &str,
    priority: &str,
    due_date: Option<chrono::NaiveDate>,
    category: Option<&str>,
) -> Result<Todo, sqlx::Error> {
    sqlx::query_as::<_, Todo>(
        r#"
        INSERT INTO todos (user_id, module, text, priority, due_date, category)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING *
        "#
    )
    .bind(user_id)
    .bind(module)
    .bind(text)
    .bind(priority)
    .bind(due_date)
    .bind(category)
    .fetch_one(pool)
    .await
}

/// Update an existing todo record
pub async fn update(
    pool: &PgPool,
    id: Uuid,
    module: &str,
    text: &str,
    completed: bool,
    priority: &str,
    due_date: Option<chrono::NaiveDate>,
    category: Option<&str>,
) -> Result<Option<Todo>, sqlx::Error> {
    sqlx::query_as::<_, Todo>(
        r#"
        UPDATE todos
        SET text = $1, completed = $2, priority = $3, due_date = $4, category = $5, updated_at = NOW()
        WHERE id = $6 AND module = $7
        RETURNING *
        "#
    )
    .bind(text)
    .bind(completed)
    .bind(priority)
    .bind(due_date)
    .bind(category)
    .bind(id)
    .bind(module)
    .fetch_optional(pool)
    .await
}

/// Delete a todo by ID and module
pub async fn delete(pool: &PgPool, id: Uuid, module: &str) -> Result<Option<Uuid>, sqlx::Error> {
    let row = sqlx::query_scalar::<_, Uuid>(
        "DELETE FROM todos WHERE id = $1 AND module = $2 RETURNING id"
    )
    .bind(id)
    .bind(module)
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

/// Delete all completed todos for a given module
pub async fn delete_completed(pool: &PgPool, module: &str) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("DELETE FROM todos WHERE module = $1 AND completed = true")
        .bind(module)
        .execute(pool)
        .await?;

    Ok(result.rows_affected())
}
