// backend-rust/src/modules/classes/service.rs
//! Classes Business Logic & Domain Service Layer

use sqlx::PgPool;
use serde_json::json;
use uuid::Uuid;
use crate::errors::AppError;
use crate::utils::activity::log_activity;
use super::models::{Class, CreateClassPayload, UpdateClassPayload};
use super::repository;

/// List all classes
pub async fn list_classes(pool: &PgPool) -> Result<Vec<Class>, AppError> {
    repository::find_all(pool)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to fetch classes: {}", e)))
}

/// Create a new class with uniqueness validation and activity logging
pub async fn create_class(
    pool: &PgPool,
    actor_id: Uuid,
    payload: CreateClassPayload,
) -> Result<Class, AppError> {
    payload.validate()?;

    let name = payload.name.trim();

    // Check for duplicate class name
    if repository::find_by_name(pool, name).await.map_err(|e| AppError::Internal(e.to_string()))?.is_some() {
        return Err(AppError::Conflict("Class name already exists".to_string()));
    }

    let new_class = repository::create(pool, name)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to create class: {}", e)))?;

    log_activity(
        pool,
        Some(actor_id),
        "CLASS_ADDED",
        "class",
        Some(new_class.id),
        Some(json!({ "name": new_class.name })),
    )
    .await?;

    Ok(new_class)
}

/// Update an existing class with duplicate validation and activity logging
pub async fn update_class(
    pool: &PgPool,
    actor_id: Uuid,
    id: Uuid,
    payload: UpdateClassPayload,
) -> Result<Class, AppError> {
    payload.validate()?;

    let name = payload.name.trim();

    // Check duplicate name on other records
    if repository::find_duplicate(pool, name, id).await.map_err(|e| AppError::Internal(e.to_string()))?.is_some() {
        return Err(AppError::Conflict("Class name already exists".to_string()));
    }

    let updated_class = repository::update(pool, id, name)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to update class: {}", e)))?
        .ok_or_else(|| AppError::NotFound("Class not found".to_string()))?;

    log_activity(
        pool,
        Some(actor_id),
        "CLASS_UPDATED",
        "class",
        Some(updated_class.id),
        Some(json!({ "name": updated_class.name })),
    )
    .await?;

    Ok(updated_class)
}

/// Delete a class by ID and log activity
pub async fn delete_class(
    pool: &PgPool,
    actor_id: Uuid,
    id: Uuid,
) -> Result<Class, AppError> {
    let deleted_class = repository::delete(pool, id)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to delete class: {}", e)))?
        .ok_or_else(|| AppError::NotFound("Class not found".to_string()))?;

    log_activity(
        pool,
        Some(actor_id),
        "CLASS_DELETED",
        "class",
        Some(deleted_class.id),
        Some(json!({ "name": deleted_class.name })),
    )
    .await?;

    Ok(deleted_class)
}
