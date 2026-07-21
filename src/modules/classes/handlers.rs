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
use super::models::{Class, CreateClassPayload, UpdateClassPayload};

pub async fn get_classes(
    State(state): State<AppState>,
    _auth_user: AuthUser, // Requires authentication (can be any logged-in user)
) -> Result<impl IntoResponse, AppError> {
    let classes = sqlx::query_as::<_, Class>("SELECT * FROM classes ORDER BY name ASC")
        .fetch_all(&state.db)
        .await?;

    Ok(Json(ApiResponse {
        success: true,
        data: classes,
    }))
}

pub async fn create_class(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(payload): Json<CreateClassPayload>,
) -> Result<impl IntoResponse, AppError> {
    // Requires Admin role
    auth_user.authorize(&[UserRole::Admin])?;
    payload.validate()?;

    // Check duplicate name
    let duplicate = sqlx::query("SELECT id FROM classes WHERE name = $1")
        .bind(&payload.name)
        .fetch_optional(&state.db)
        .await?;

    if duplicate.is_some() {
        return Err(AppError::Conflict("Class name already exists".to_string()));
    }

    let new_class = sqlx::query_as::<_, Class>(
        "INSERT INTO classes (name) VALUES ($1) RETURNING *"
    )
    .bind(&payload.name)
    .fetch_one(&state.db)
    .await?;

    // Log audit activity
    log_activity(
        &state.db,
        Some(auth_user.id),
        "CLASS_ADDED",
        "class",
        Some(new_class.id),
        Some(json!({ "name": new_class.name })),
    )
    .await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse {
            success: true,
            data: new_class,
        }),
    ))
}

pub async fn edit_class(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateClassPayload>,
) -> Result<impl IntoResponse, AppError> {
    // Requires Admin role
    auth_user.authorize(&[UserRole::Admin])?;
    payload.validate()?;

    // Check duplicate name excluding current class ID
    let duplicate = sqlx::query("SELECT id FROM classes WHERE name = $1 AND id != $2")
        .bind(&payload.name)
        .bind(id)
        .fetch_optional(&state.db)
        .await?;

    if duplicate.is_some() {
        return Err(AppError::Conflict("Class name already exists".to_string()));
    }

    let updated_class = sqlx::query_as::<_, Class>(
        "UPDATE classes SET name = $1, updated_at = now() WHERE id = $2 RETURNING *"
    )
    .bind(&payload.name)
    .bind(id)
    .fetch_optional(&state.db)
    .await?;

    let updated_class = updated_class.ok_or_else(|| AppError::NotFound("Class not found".to_string()))?;

    // Log audit activity
    log_activity(
        &state.db,
        Some(auth_user.id),
        "CLASS_UPDATED",
        "class",
        Some(updated_class.id),
        Some(json!({ "name": updated_class.name })),
    )
    .await?;

    Ok(Json(ApiResponse {
        success: true,
        data: updated_class,
    }))
}

pub async fn remove_class(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    // Requires Admin role
    auth_user.authorize(&[UserRole::Admin])?;

    let deleted_class = sqlx::query_as::<_, Class>(
        "DELETE FROM classes WHERE id = $1 RETURNING *"
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?;

    let deleted_class = deleted_class.ok_or_else(|| AppError::NotFound("Class not found".to_string()))?;

    // Log audit activity
    log_activity(
        &state.db,
        Some(auth_user.id),
        "CLASS_DELETED",
        "class",
        Some(deleted_class.id),
        Some(json!({ "name": deleted_class.name })),
    )
    .await?;

    Ok(Json(ApiResponse {
        success: true,
        data: deleted_class,
    }))
}
