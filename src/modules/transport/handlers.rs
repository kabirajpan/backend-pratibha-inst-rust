// backend-rust/src/modules/transport/handlers.rs
//! Transport HTTP Presentation Layer (Axum Handlers)

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

// ─── VEHICLES ─────────────────────────────────────────────

pub async fn get_vehicles(
    State(state): State<AppState>,
    _auth_user: AuthUser,
    Query(q): Query<GetVehiclesQuery>,
) -> Result<impl IntoResponse, AppError> {
    let (list, total, page, limit) = service::list_vehicles(&state.db, &q).await?;

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

pub async fn get_vehicle(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize_sub_role(&[UserSubRole::TransportManager, UserSubRole::FinanceManager])?;

    let vehicle = service::get_vehicle(&state.db, id).await?;
    Ok(Json(ApiResponse { success: true, data: vehicle }))
}

pub async fn create_vehicle(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(payload): Json<AddVehiclePayload>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize_sub_role(&[UserSubRole::TransportManager, UserSubRole::FinanceManager])?;

    let vehicle = service::create_vehicle(&state.db, payload).await?;
    Ok((StatusCode::CREATED, Json(ApiResponse { success: true, data: vehicle })))
}

pub async fn edit_vehicle(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateVehiclePayload>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize_sub_role(&[UserSubRole::TransportManager, UserSubRole::FinanceManager])?;

    let vehicle = service::update_vehicle(&state.db, id, payload).await?;
    Ok(Json(ApiResponse { success: true, data: vehicle }))
}

pub async fn remove_vehicle(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize_sub_role(&[UserSubRole::TransportManager, UserSubRole::FinanceManager])?;

    let vehicle = service::delete_vehicle(&state.db, id).await?;
    Ok(Json(ApiResponse { success: true, data: vehicle }))
}

// ─── EXPENSES ─────────────────────────────────────────────

pub async fn get_expenses(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Query(q): Query<GetExpensesQuery>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize_sub_role(&[UserSubRole::TransportManager, UserSubRole::FinanceManager])?;

    let (list, total, page, limit) = service::list_expenses(&state.db, &q).await?;

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

pub async fn get_expense(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize_sub_role(&[UserSubRole::TransportManager, UserSubRole::FinanceManager])?;

    let expense = service::get_expense(&state.db, id).await?;
    Ok(Json(ApiResponse { success: true, data: expense }))
}

pub async fn create_expense(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(payload): Json<AddExpensePayload>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize_sub_role(&[UserSubRole::TransportManager, UserSubRole::FinanceManager])?;

    let user_role_str = format!("{:?}", auth_user.role);
    let expense = service::create_expense(&state.db, &user_role_str, payload).await?;
    Ok((StatusCode::CREATED, Json(ApiResponse { success: true, data: expense })))
}

pub async fn edit_expense(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateExpensePayload>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize_sub_role(&[UserSubRole::TransportManager, UserSubRole::FinanceManager])?;

    let expense = service::update_expense(&state.db, id, payload).await?;
    Ok(Json(ApiResponse { success: true, data: expense }))
}

pub async fn remove_expense(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize_sub_role(&[UserSubRole::TransportManager, UserSubRole::FinanceManager])?;

    let expense = service::delete_expense(&state.db, id).await?;
    Ok(Json(ApiResponse { success: true, data: expense }))
}

// ─── TRANSPORT STUDENTS ───────────────────────────────────

pub async fn get_transport_students(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Query(q): Query<GetTransportStudentsQuery>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize_sub_role(&[UserSubRole::TransportManager, UserSubRole::FinanceManager])?;

    let (list, total, page, limit) = service::list_transport_students(&state.db, &q).await?;

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

pub async fn get_transport_student(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize_sub_role(&[UserSubRole::TransportManager, UserSubRole::FinanceManager])?;

    let student = service::get_transport_student(&state.db, id).await?;
    Ok(Json(ApiResponse { success: true, data: student }))
}

pub async fn create_transport_student(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(payload): Json<AddTransportStudentPayload>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize_sub_role(&[UserSubRole::TransportManager, UserSubRole::FinanceManager])?;

    let record = service::create_transport_student(&state.db, payload).await?;
    Ok((StatusCode::CREATED, Json(ApiResponse { success: true, data: record })))
}

pub async fn edit_transport_student(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateTransportStudentPayload>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize_sub_role(&[UserSubRole::TransportManager, UserSubRole::FinanceManager])?;

    let record = service::update_transport_student(&state.db, id, payload).await?;
    Ok(Json(ApiResponse { success: true, data: record }))
}

pub async fn remove_transport_student(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize_sub_role(&[UserSubRole::TransportManager, UserSubRole::FinanceManager])?;

    let record = service::delete_transport_student(&state.db, id).await?;
    Ok(Json(ApiResponse { success: true, data: record }))
}

pub async fn import_transport_students(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(payload): Json<Vec<ImportTransportStudentRow>>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize_sub_role(&[UserSubRole::TransportManager, UserSubRole::FinanceManager])?;

    let results = service::import_transport_students(&state.db, payload).await?;
    Ok(Json(json!({ "success": true, "count": results.len(), "data": results })))
}
