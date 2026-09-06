// backend-rust/src/modules/classes/handlers.rs
//! Classes HTTP Presentation Layer (Axum Handlers)

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
use super::models::{CreateClassPayload, UpdateClassPayload};
use super::service;

pub async fn get_classes(
    State(state): State<AppState>,
    _auth_user: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    let classes = service::list_classes(&state.db).await?;

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
    auth_user.authorize(&[UserRole::Admin])?;

    let new_class = service::create_class(&state.db, auth_user.id, payload).await?;

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
    auth_user.authorize(&[UserRole::Admin])?;

    let updated_class = service::update_class(&state.db, auth_user.id, id, payload).await?;

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
    auth_user.authorize(&[UserRole::Admin])?;

    let deleted_class = service::delete_class(&state.db, auth_user.id, id).await?;

    Ok(Json(ApiResponse {
        success: true,
        data: deleted_class,
    }))
}
