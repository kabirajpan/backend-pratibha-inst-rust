// backend-rust/src/modules/courses/service.rs
//! Courses Business Logic & Domain Service Layer

use sqlx::PgPool;
use serde_json::json;
use uuid::Uuid;
use crate::errors::AppError;
use crate::utils::activity::log_activity;
use super::models::{Course, CreateCoursePayload, UpdateCoursePayload};
use super::repository;

/// List all courses
pub async fn list_courses(pool: &PgPool) -> Result<Vec<Course>, AppError> {
    repository::find_all(pool)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to fetch courses: {}", e)))
}

/// Create a new course with duplicate validation and activity logging
pub async fn create_course(
    pool: &PgPool,
    actor_id: Uuid,
    payload: CreateCoursePayload,
) -> Result<Course, AppError> {
    payload.validate()?;

    let name = payload.name.trim();

    if repository::find_by_name(pool, name).await.map_err(|e| AppError::Internal(e.to_string()))?.is_some() {
        return Err(AppError::Conflict("Course name already exists".to_string()));
    }

    let new_course = repository::create(pool, name)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to create course: {}", e)))?;

    log_activity(
        pool,
        Some(actor_id),
        "COURSE_ADDED",
        "course",
        Some(new_course.id),
        Some(json!({ "name": new_course.name })),
    )
    .await?;

    Ok(new_course)
}

/// Update an existing course with duplicate validation and activity logging
pub async fn update_course(
    pool: &PgPool,
    actor_id: Uuid,
    id: Uuid,
    payload: UpdateCoursePayload,
) -> Result<Course, AppError> {
    payload.validate()?;

    let name = payload.name.trim();

    if repository::find_duplicate(pool, name, id).await.map_err(|e| AppError::Internal(e.to_string()))?.is_some() {
        return Err(AppError::Conflict("Course name already exists".to_string()));
    }

    let updated_course = repository::update(pool, id, name)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to update course: {}", e)))?
        .ok_or_else(|| AppError::NotFound("Course not found".to_string()))?;

    log_activity(
        pool,
        Some(actor_id),
        "COURSE_UPDATED",
        "course",
        Some(updated_course.id),
        Some(json!({ "name": updated_course.name })),
    )
    .await?;

    Ok(updated_course)
}

/// Delete a course by ID and log activity
pub async fn delete_course(
    pool: &PgPool,
    actor_id: Uuid,
    id: Uuid,
) -> Result<Course, AppError> {
    let deleted_course = repository::delete(pool, id)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to delete course: {}", e)))?
        .ok_or_else(|| AppError::NotFound("Course not found".to_string()))?;

    log_activity(
        pool,
        Some(actor_id),
        "COURSE_DELETED",
        "course",
        Some(deleted_course.id),
        Some(json!({ "name": deleted_course.name })),
    )
    .await?;

    Ok(deleted_course)
}
