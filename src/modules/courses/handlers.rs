// backend-rust/src/modules/courses/handlers.rs
//! Courses HTTP Presentation Layer (Axum Handlers)

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use uuid::Uuid;
use crate::AppState;
use crate::errors::AppError;
use crate::middleware::AuthUser;
use crate::modules::auth::models::{UserRole, ApiResponse};
use super::models::{CreateCoursePayload, UpdateCoursePayload};
use super::service;

pub async fn get_courses(
    State(state): State<AppState>,
    _auth_user: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    let courses = service::list_courses(&state.db).await?;

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

    let new_course = service::create_course(&state.db, auth_user.id, payload).await?;

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

    let updated_course = service::update_course(&state.db, auth_user.id, id, payload).await?;

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

    let deleted_course = service::delete_course(&state.db, auth_user.id, id).await?;

    Ok(Json(ApiResponse {
        success: true,
        data: deleted_course,
    }))
}
