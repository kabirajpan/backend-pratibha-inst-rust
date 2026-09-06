// backend-rust/src/modules/admin/handlers.rs
//! Admin & Students HTTP Presentation Layer (Axum Handlers)

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use uuid::Uuid;
use crate::AppState;
use crate::errors::AppError;
use crate::middleware::AuthUser;
use crate::modules::auth::models::{ApiResponse, ApiMessageResponse, UserRole};
use super::models::*;
use super::service;

// ─── STUDENTS ─────────────────────────────────────────────

pub async fn get_students(
    State(state): State<AppState>,
    _auth_user: AuthUser,
    Query(q): Query<GetStudentsQuery>,
) -> Result<impl IntoResponse, AppError> {
    let (list, total, page, limit) = service::list_students(&state.db, &q).await?;

    Ok(Json(json!({
        "success": true,
        "data": list,
        "pagination": {
            "total": total,
            "page": page,
            "limit": limit,
            "pages": (total as f64 / limit as f64).ceil() as i32
        }
    })))
}

pub async fn get_student(
    State(state): State<AppState>,
    _auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let student = service::get_student(&state.db, id).await?;
    Ok(Json(ApiResponse { success: true, data: student }))
}

pub async fn create_student(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(payload): Json<CreateStudentPayload>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize(&[UserRole::Admin])?;

    let resp = service::create_student(&state.db, payload).await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse {
            success: true,
            data: resp,
        }),
    ))
}

pub async fn edit_student(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateStudentPayload>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize(&[UserRole::Admin])?;

    let updated = service::update_student(&state.db, id, payload).await?;

    Ok(Json(ApiResponse {
        success: true,
        data: updated,
    }))
}

pub async fn remove_student(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize(&[UserRole::Admin])?;

    let deleted = service::delete_student(&state.db, id).await?;

    Ok(Json(ApiResponse {
        success: true,
        data: deleted,
    }))
}

pub async fn import_students(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(payload): Json<ImportStudentsPayload>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize(&[UserRole::Admin])?;

    let results = service::import_students(&state.db, payload.students).await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse { success: true, data: results }),
    ))
}

// ─── USER & STAFF MANAGEMENT ──────────────────────────────

pub async fn get_all_users(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize(&[UserRole::Admin])?;

    let users = service::list_all_users(&state.db).await?;

    Ok(Json(ApiResponse { success: true, data: users }))
}

pub async fn toggle_user_active(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize(&[UserRole::Admin])?;

    let is_active = service::toggle_user_active(&state.db, auth_user.id, id).await?;

    Ok(Json(json!({
        "success": true,
        "data": {
            "id": id,
            "is_active": is_active
        }
    })))
}

pub async fn delete_user(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize(&[UserRole::Admin])?;

    service::delete_user(&state.db, auth_user.id, id).await?;

    Ok(Json(ApiMessageResponse {
        success: true,
        message: "User deleted".to_string(),
    }))
}

// ─── AUDIT LOGS ───────────────────────────────────────────

pub async fn get_audit_logs(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Query(q): Query<GetAuditLogsQuery>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize(&[UserRole::Admin])?;

    let logs = service::list_audit_logs(&state.db, &q).await?;

    Ok(Json(ApiResponse {
        success: true,
        data: logs,
    }))
}

pub async fn create_audit_log(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(payload): Json<CreateAuditLogPayload>,
) -> Result<impl IntoResponse, AppError> {
    let user_role_str = format!("{:?}", auth_user.role);
    service::record_audit_log(&state.db, &user_role_str, payload).await?;

    Ok(Json(json!({ "success": true })))
}
