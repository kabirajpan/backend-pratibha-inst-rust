// src/modules/inventory/handlers.rs
//! Inventory HTTP Presentation Layer (Axum Handlers)

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
use crate::modules::auth::models::UserSubRole;
use super::models::*;
use super::service;

// ─── STATS & LOW STOCK ────────────────────────────────────

pub async fn get_stats(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize_sub_role(&[UserSubRole::InventoryManager])?;

    let row = service::get_stats(&state.db).await?;
    Ok((StatusCode::OK, Json(json!({ "success": true, "data": row }))))
}

pub async fn get_low_stock(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize_sub_role(&[UserSubRole::InventoryManager])?;

    let items = service::get_low_stock(&state.db).await?;
    Ok((StatusCode::OK, Json(json!({ "success": true, "data": items }))))
}

// ─── CATEGORIES ──────────────────────────────────────────

pub async fn get_categories(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize_sub_role(&[UserSubRole::InventoryManager])?;

    let categories = service::list_categories(&state.db).await?;
    Ok((StatusCode::OK, Json(json!({ "success": true, "data": categories }))))
}

pub async fn create_category(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(payload): Json<CreateCategoryPayload>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize_sub_role(&[UserSubRole::InventoryManager])?;

    let cat = service::create_category(&state.db, payload).await?;
    Ok((StatusCode::CREATED, Json(json!({ "success": true, "data": cat }))))
}

pub async fn edit_category(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateCategoryPayload>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize_sub_role(&[UserSubRole::InventoryManager])?;

    let updated = service::update_category(&state.db, id, payload).await?;
    Ok((StatusCode::OK, Json(json!({ "success": true, "data": updated }))))
}

pub async fn remove_category(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize_sub_role(&[UserSubRole::InventoryManager])?;

    service::delete_category(&state.db, id).await?;
    Ok((StatusCode::OK, Json(json!({ "success": true, "message": "Category deleted successfully" }))))
}

// ─── ITEMS ───────────────────────────────────────────────

pub async fn get_items(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Query(q): Query<GetItemsQuery>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize_sub_role(&[UserSubRole::InventoryManager])?;

    let (items, total, page, limit) = service::list_items(&state.db, &q).await?;

    Ok((StatusCode::OK, Json(json!({
        "success": true,
        "data": items,
        "pagination": {
            "total": total,
            "page": page,
            "limit": limit,
            "pages": (total as f64 / limit as f64).ceil() as i32
        }
    }))))
}

pub async fn get_item(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize_sub_role(&[UserSubRole::InventoryManager])?;

    let item = service::get_item(&state.db, id).await?;
    Ok((StatusCode::OK, Json(json!({ "success": true, "data": item }))))
}

pub async fn create_item(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(payload): Json<CreateItemPayload>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize_sub_role(&[UserSubRole::InventoryManager])?;

    let item = service::create_item(&state.db, auth_user.id, payload).await?;
    Ok((StatusCode::CREATED, Json(json!({ "success": true, "data": item }))))
}

pub async fn edit_item(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateItemPayload>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize_sub_role(&[UserSubRole::InventoryManager])?;

    let item = service::update_item(&state.db, id, payload).await?;
    Ok((StatusCode::OK, Json(json!({ "success": true, "data": item }))))
}

pub async fn remove_item(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize_sub_role(&[UserSubRole::InventoryManager])?;

    service::delete_item(&state.db, id).await?;
    Ok((StatusCode::OK, Json(json!({ "success": true, "message": "Item deleted successfully" }))))
}

pub async fn import_items(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(payload): Json<ImportItemsPayload>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize_sub_role(&[UserSubRole::InventoryManager])?;

    let results = service::import_items(&state.db, auth_user.id, payload.items).await?;
    Ok((StatusCode::OK, Json(json!({ "success": true, "data": results }))))
}

// ─── ISSUES & RETURNS ────────────────────────────────────

pub async fn get_issues(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize_sub_role(&[UserSubRole::InventoryManager])?;

    let issues = service::list_issues(&state.db).await?;
    Ok((StatusCode::OK, Json(json!({ "success": true, "data": issues }))))
}

pub async fn issue_item(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(payload): Json<IssueItemPayload>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize_sub_role(&[UserSubRole::InventoryManager])?;

    let issue = service::issue_item(&state.db, auth_user.id, payload).await?;
    Ok((StatusCode::CREATED, Json(json!({ "success": true, "data": issue }))))
}

pub async fn return_item(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(payload): Json<ReturnItemPayload>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize_sub_role(&[UserSubRole::InventoryManager])?;

    let updated = service::return_item(&state.db, auth_user.id, payload).await?;
    Ok((StatusCode::OK, Json(json!({ "success": true, "data": updated }))))
}

pub async fn import_issues(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(payload): Json<ImportIssuesPayload>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize_sub_role(&[UserSubRole::InventoryManager])?;

    let results = service::import_issues(&state.db, auth_user.id, payload.issues).await?;
    Ok((StatusCode::OK, Json(json!({ "success": true, "data": results }))))
}
