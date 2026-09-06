// backend-rust/src/modules/todos/handlers.rs
//! Todos HTTP Presentation Layer (Axum Handlers)

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use uuid::Uuid;
use crate::AppState;
use crate::errors::AppError;
use crate::middleware::AuthUser;
use crate::modules::auth::models::{ApiResponse, ApiMessageResponse};
use super::models::{CreateTodoPayload, UpdateTodoPayload};
use super::service;

pub async fn get_todos(
    State(state): State<AppState>,
    _auth_user: AuthUser,
    Path(module): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let todos = service::list_todos(&state.db, &module).await?;

    Ok(Json(ApiResponse {
        success: true,
        data: todos,
    }))
}

pub async fn create_todo(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(module): Path<String>,
    Json(payload): Json<CreateTodoPayload>,
) -> Result<impl IntoResponse, AppError> {
    let todo = service::create_todo(&state.db, auth_user.id, &module, payload).await?;

    Ok(Json(ApiResponse {
        success: true,
        data: todo,
    }))
}

pub async fn edit_todo(
    State(state): State<AppState>,
    _auth_user: AuthUser,
    Path((module, id)): Path<(String, Uuid)>,
    Json(payload): Json<UpdateTodoPayload>,
) -> Result<impl IntoResponse, AppError> {
    let updated = service::update_todo(&state.db, id, &module, payload).await?;

    Ok(Json(ApiResponse {
        success: true,
        data: updated,
    }))
}

pub async fn remove_todo(
    State(state): State<AppState>,
    _auth_user: AuthUser,
    Path((module, id)): Path<(String, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    service::delete_todo(&state.db, id, &module).await?;

    Ok(Json(ApiMessageResponse {
        success: true,
        message: "Todo deleted".to_string(),
    }))
}

pub async fn clear_completed_todos(
    State(state): State<AppState>,
    _auth_user: AuthUser,
    Path(module): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    service::clear_completed(&state.db, &module).await?;

    Ok(Json(ApiMessageResponse {
        success: true,
        message: "Completed todos cleared".to_string(),
    }))
}
