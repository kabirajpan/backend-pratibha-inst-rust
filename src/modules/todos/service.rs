// backend-rust/src/modules/todos/service.rs
//! Todos Business Logic & Domain Service Layer

use sqlx::PgPool;
use uuid::Uuid;
use crate::errors::AppError;
use super::models::{Todo, CreateTodoPayload, UpdateTodoPayload};
use super::repository;

/// List all todos for a module
pub async fn list_todos(pool: &PgPool, module: &str) -> Result<Vec<Todo>, AppError> {
    repository::find_by_module(pool, module)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to fetch todos: {}", e)))
}

/// Create a new todo with validation
pub async fn create_todo(
    pool: &PgPool,
    user_id: Uuid,
    module: &str,
    payload: CreateTodoPayload,
) -> Result<Todo, AppError> {
    payload.validate()?;

    let parsed_due_date = match &payload.due_date {
        Some(d) if !d.is_empty() => Some(
            chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d")
                .map_err(|_| AppError::BadRequest("Invalid due date format".to_string()))?
        ),
        _ => None,
    };

    let priority = payload.priority.unwrap_or_else(|| "medium".to_string());

    repository::create(
        pool,
        Some(user_id),
        module,
        payload.text.trim(),
        &priority,
        parsed_due_date,
        payload.category.as_deref(),
    )
    .await
    .map_err(|e| AppError::Internal(format!("Failed to create todo: {}", e)))
}

/// Update an existing todo
pub async fn update_todo(
    pool: &PgPool,
    id: Uuid,
    module: &str,
    payload: UpdateTodoPayload,
) -> Result<Todo, AppError> {
    payload.validate()?;

    let existing = repository::find_by_id_and_module(pool, id, module)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Todo not found".to_string()))?;

    let text = payload.text.unwrap_or(existing.text);
    let completed = payload.completed.unwrap_or(existing.completed);
    let priority = payload.priority.unwrap_or(existing.priority);
    let due_date = match payload.due_date {
        Some(ref d) if !d.is_empty() => Some(
            chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d")
                .map_err(|_| AppError::BadRequest("Invalid due date format".to_string()))?
        ),
        Some(_) => None,
        None => existing.due_date,
    };
    let category = match payload.category {
        Some(ref c) if !c.is_empty() => Some(c.clone()),
        Some(_) => None,
        None => existing.category,
    };

    repository::update(
        pool,
        id,
        module,
        text.trim(),
        completed,
        &priority,
        due_date,
        category.as_deref(),
    )
    .await
    .map_err(|e| AppError::Internal(format!("Failed to update todo: {}", e)))?
    .ok_or_else(|| AppError::NotFound("Todo not found".to_string()))
}

/// Delete a todo
pub async fn delete_todo(pool: &PgPool, id: Uuid, module: &str) -> Result<(), AppError> {
    let deleted = repository::delete(pool, id, module)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to delete todo: {}", e)))?;

    if deleted.is_none() {
        return Err(AppError::NotFound("Todo not found".to_string()));
    }

    Ok(())
}

/// Clear all completed todos for a module
pub async fn clear_completed(pool: &PgPool, module: &str) -> Result<u64, AppError> {
    repository::delete_completed(pool, module)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to clear completed todos: {}", e)))
}
