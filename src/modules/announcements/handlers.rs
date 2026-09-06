// backend-rust/src/modules/announcements/handlers.rs
//! Announcements & Notifications HTTP Presentation Layer (Axum Handlers)

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
use crate::modules::auth::models::{ApiResponse, UserRole};
use super::models::*;
use super::service;

// ─── ANNOUNCEMENT HANDLERS ───────────────────────────────────────────────────

pub async fn create_announcement(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(payload): Json<CreateAnnouncementPayload>,
) -> Result<impl IntoResponse, AppError> {
    let announcement = service::create_announcement(&state.db, &state.config, &auth_user, payload).await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse {
            success: true,
            data: announcement,
        }),
    ))
}

pub async fn get_announcements(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    let announcements = service::list_announcements(&state.db, &auth_user).await?;

    Ok(Json(ApiResponse {
        success: true,
        data: announcements,
    }))
}

pub async fn delete_announcement(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize(&[UserRole::Admin])?;

    service::delete_announcement(&state.db, id).await?;

    Ok(Json(ApiResponse {
        success: true,
        data: "Announcement deleted successfully",
    }))
}

pub async fn get_broadcast_permissions(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize(&[UserRole::Admin])?;

    let perms = service::list_permissions(&state.db).await?;

    Ok(Json(ApiResponse {
        success: true,
        data: perms,
    }))
}

pub async fn update_broadcast_permissions(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(payload): Json<UpdatePermissionsPayload>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize(&[UserRole::Admin])?;

    service::update_permissions(&state.db, payload).await?;

    Ok(Json(ApiResponse {
        success: true,
        data: "Permissions updated successfully",
    }))
}

// ─── NOTIFICATION HANDLERS ───────────────────────────────────────────────────

pub async fn get_user_notifications(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    let resp = service::get_user_notifications(&state.db, auth_user.id).await?;

    Ok(Json(ApiResponse {
        success: true,
        data: resp,
    }))
}

pub async fn mark_notification_read(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    service::mark_notification_read(&state.db, id, auth_user.id).await?;

    Ok(Json(ApiResponse {
        success: true,
        data: "Marked as read",
    }))
}

pub async fn mark_all_notifications_read(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    service::mark_all_notifications_read(&state.db, auth_user.id).await?;

    Ok(Json(ApiResponse {
        success: true,
        data: "All marked as read",
    }))
}
