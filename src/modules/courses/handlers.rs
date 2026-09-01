use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use uuid::Uuid;
use crate::AppState;
use crate::errors::AppError;
use crate::middleware::AuthUser;
use crate::modules::auth::models::{UserRole, ApiResponse};
use crate::utils::activity::log_activity;
use super::models::{Course, CreateCoursePayload, UpdateCoursePayload};

pub async fn get_courses(
    State(state): State<AppState>,
    _auth_user: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    let _ = sqlx::query(
        "CREATE TABLE IF NOT EXISTS courses (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            name VARCHAR UNIQUE NOT NULL,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            updated_at TIMESTAMPTZ DEFAULT NOW()
        );"
    )
    .execute(&state.db)
    .await;

    let courses = sqlx::query_as::<_, Course>("SELECT id, name, created_at, updated_at FROM courses ORDER BY name ASC")
        .fetch_all(&state.db)
        .await?;

    Ok(Json(ApiResponse {
        success: true,
        data: courses,
    }))
}

pub async fn create_course(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(payload): Json<CreateCoursePayload>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize(&[UserRole::Admin])?;
    payload.validate()?;

    let name = payload.name.trim();

    let _ = sqlx::query(
        "CREATE TABLE IF NOT EXISTS courses (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            name VARCHAR UNIQUE NOT NULL,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            updated_at TIMESTAMPTZ DEFAULT NOW()
        );"
    )
    .execute(&state.db)
    .await;

    let duplicate = sqlx::query("SELECT id FROM courses WHERE name = $1")
        .bind(name)
        .fetch_optional(&state.db)
        .await?;

    if duplicate.is_some() {
        return Err(AppError::Conflict("Course name already exists".to_string()));
    }

    let new_course = sqlx::query_as::<_, Course>(
        "INSERT INTO courses (name) VALUES ($1) RETURNING id, name, created_at, updated_at"
    )
    .bind(name)
    .fetch_one(&state.db)
    .await?;

    log_activity(
        &state.db,
        Some(auth_user.id),
        "COURSE_ADDED",
        "course",
        Some(new_course.id),
        Some(json!({ "name": new_course.name })),
    )
    .await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse {
            success: true,
            data: new_course,
        }),
    ))
}

pub async fn edit_course(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateCoursePayload>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize(&[UserRole::Admin])?;
    payload.validate()?;

    let name = payload.name.trim();

    let duplicate = sqlx::query("SELECT id FROM courses WHERE name = $1 AND id != $2")
        .bind(name)
        .bind(id)
        .fetch_optional(&state.db)
        .await?;

    if duplicate.is_some() {
        return Err(AppError::Conflict("Course name already exists".to_string()));
    }

    let updated_course = sqlx::query_as::<_, Course>(
        "UPDATE courses SET name = $1, updated_at = now() WHERE id = $2 RETURNING id, name, created_at, updated_at"
    )
    .bind(name)
    .bind(id)
    .fetch_optional(&state.db)
    .await?;

    let updated_course = updated_course.ok_or_else(|| AppError::NotFound("Course not found".to_string()))?;

    log_activity(
        &state.db,
        Some(auth_user.id),
        "COURSE_UPDATED",
        "course",
        Some(updated_course.id),
        Some(json!({ "name": updated_course.name })),
    )
    .await?;

    Ok(Json(ApiResponse {
        success: true,
        data: updated_course,
    }))
}

pub async fn remove_course(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize(&[UserRole::Admin])?;

    let deleted_course = sqlx::query_as::<_, Course>(
        "DELETE FROM courses WHERE id = $1 RETURNING id, name, created_at, updated_at"
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?;

    let deleted_course = deleted_course.ok_or_else(|| AppError::NotFound("Course not found".to_string()))?;

    log_activity(
        &state.db,
        Some(auth_user.id),
        "COURSE_DELETED",
        "course",
        Some(deleted_course.id),
        Some(json!({ "name": deleted_course.name })),
    )
    .await?;

    Ok(Json(ApiResponse {
        success: true,
        data: deleted_course,
    }))
}
