// backend-rust/src/modules/hostel/handlers.rs
//! Hostel HTTP Presentation Layer (Axum Handlers)

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
use crate::modules::auth::models::{ApiResponse, UserSubRole};
use super::models::*;
use super::service;

// ─── HOSTEL ROOMS ──────────────────────────────────────────────────────────

pub async fn get_hostel_rooms(
    State(state): State<AppState>,
    _auth_user: AuthUser,
    Query(q): Query<GetHostelRoomsQuery>,
) -> Result<impl IntoResponse, AppError> {
    let rooms = service::list_rooms(&state.db, &q).await?;
    Ok(Json(ApiResponse { success: true, data: rooms }))
}

pub async fn create_hostel_room(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(payload): Json<CreateHostelRoomPayload>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize_sub_role(&[UserSubRole::HostelManager, UserSubRole::FinanceManager, UserSubRole::TransportManager])?;

    let record = service::create_or_update_room(&state.db, payload).await?;
    Ok((StatusCode::CREATED, Json(ApiResponse { success: true, data: record })))
}

pub async fn edit_hostel_room(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateHostelRoomPayload>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize_sub_role(&[UserSubRole::HostelManager, UserSubRole::FinanceManager, UserSubRole::TransportManager])?;

    let record = service::update_room(&state.db, id, payload).await?;
    Ok(Json(ApiResponse { success: true, data: record }))
}

pub async fn remove_hostel_room(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize_sub_role(&[UserSubRole::HostelManager, UserSubRole::FinanceManager, UserSubRole::TransportManager])?;

    service::delete_room(&state.db, id).await?;
    Ok(Json(json!({ "success": true, "message": "Hostel room deleted successfully" })))
}

// ─── HOSTEL STUDENTS / RESIDENTS ──────────────────────────────────────────

pub async fn get_hostel_students(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Query(q): Query<GetHostelStudentsQuery>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize_sub_role(&[UserSubRole::HostelManager, UserSubRole::FinanceManager, UserSubRole::TransportManager])?;

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

pub async fn create_hostel_student(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(payload): Json<AddHostelStudentPayload>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize_sub_role(&[UserSubRole::HostelManager, UserSubRole::FinanceManager, UserSubRole::TransportManager])?;

    let record = service::allocate_student(&state.db, payload).await?;
    Ok((StatusCode::CREATED, Json(ApiResponse { success: true, data: record })))
}

pub async fn edit_hostel_student(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateHostelStudentPayload>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize_sub_role(&[UserSubRole::HostelManager, UserSubRole::FinanceManager, UserSubRole::TransportManager])?;

    let record = service::update_student(&state.db, id, payload).await?;
    Ok(Json(ApiResponse { success: true, data: record }))
}

pub async fn remove_hostel_student(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize_sub_role(&[UserSubRole::HostelManager, UserSubRole::FinanceManager, UserSubRole::TransportManager])?;

    service::delete_student(&state.db, id).await?;
    Ok(Json(json!({ "success": true, "message": "Resident record deleted successfully" })))
}
