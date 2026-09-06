// backend-rust/src/modules/announcements/service.rs
//! Announcements & Notifications Business Logic & Domain Service Layer

use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;
use crate::config::Config;
use crate::errors::AppError;
use crate::middleware::AuthUser;
use crate::modules::auth::models::UserRole;
use super::models::{
    Announcement, AnnouncementPermission, CreateAnnouncementPayload,
    NotificationListResponse, UpdatePermissionsPayload,
};
use super::repository;

// ─── Announcement Services ───────────────────────────────────────────────────

pub async fn create_announcement(
    pool: &PgPool,
    config: &Config,
    auth_user: &AuthUser,
    payload: CreateAnnouncementPayload,
) -> Result<Announcement, AppError> {
    // 0. Permission check: Admin can always broadcast. Staff sub-roles require explicit permission.
    if auth_user.role != UserRole::Admin {
        let sub_role = auth_user.sub_role.as_ref().map(|s| format!("{:?}", s).to_lowercase()).unwrap_or_default();
        let can_broadcast = repository::find_permission(pool, &sub_role)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?
            .unwrap_or(false);

        if !can_broadcast {
            return Err(AppError::Forbidden(
                "Only administrators and authorized staff roles are permitted to broadcast announcements.".to_string(),
            ));
        }
    }

    if payload.title.trim().is_empty() || payload.content.trim().is_empty() {
        return Err(AppError::BadRequest("Title and content cannot be empty".to_string()));
    }

    let priority = payload.priority.unwrap_or_else(|| "normal".to_string());
    let target_roles = payload.target_roles.unwrap_or_else(|| vec!["all".to_string()]);
    let target_sub_roles = payload.target_sub_roles.unwrap_or_else(|| vec!["all".to_string()]);
    let target_class_names = payload.target_class_names.unwrap_or_else(|| vec!["all".to_string()]);
    let send_email = payload.send_email.unwrap_or(false);
    let send_sms = payload.send_sms.unwrap_or(false);

    let target_roles_json = serde_json::to_value(&target_roles).unwrap_or(json!(["all"]));
    let target_sub_roles_json = serde_json::to_value(&target_sub_roles).unwrap_or(json!(["all"]));
    let target_class_names_json = serde_json::to_value(&target_class_names).unwrap_or(json!(["all"]));

    let announcement = repository::insert_announcement(
        pool,
        &payload.title,
        &payload.content,
        &priority,
        &target_roles_json,
        &target_sub_roles_json,
        &target_class_names_json,
        send_email,
        send_sms,
        auth_user.id,
    )
    .await
    .map_err(|e| AppError::Internal(format!("Failed to create announcement: {}", e)))?;

    // Resolve Target Users for in-app Notifications & External Dispatch
    let users_query = r#"
        SELECT u.id, u.email, u.role::text, u.sub_role::text, s.phone, s.class_name
        FROM users u
        LEFT JOIN students s ON s.email = u.email
        WHERE u.is_active = true
    "#;

    if let Ok(users) = sqlx::query_as::<_, (Uuid, String, String, Option<String>, Option<String>, Option<String>)>(users_query)
        .fetch_all(pool)
        .await
    {
        for (user_id, email, role, sub_role, phone, class_name) in users {
            let role_clean = role.to_lowercase();
            let sub_role_clean = sub_role.unwrap_or_default().to_lowercase();
            let class_clean = class_name.unwrap_or_default();

            let role_match = target_roles.contains(&"all".to_string()) || target_roles.iter().any(|r| r.to_lowercase() == role_clean);
            let sub_role_match = target_sub_roles.contains(&"all".to_string()) || target_sub_roles.iter().any(|sr| sr.to_lowercase() == sub_role_clean);
            let class_match = target_class_names.contains(&"all".to_string()) || target_class_names.iter().any(|c| c.to_lowercase() == class_clean.to_lowercase());

            if role_match && sub_role_match && class_match {
                let _ = repository::insert_notification(
                    pool,
                    user_id,
                    Some(announcement.id),
                    &payload.title,
                    &payload.content,
                    &priority,
                )
                .await;

                if send_email && !email.is_empty() {
                    let html = format!(
                        "<h2>{}</h2><p>{}</p><hr><small>Priority: {} | Pratibha ERP Notification</small>",
                        payload.title, payload.content, priority
                    );
                    crate::modules::email::service::send_email_async(
                        config.clone(),
                        email,
                        format!("[Announcement] {}", payload.title),
                        html,
                    );
                }

                if send_sms {
                    if let Some(ph) = phone {
                        if !ph.is_empty() {
                            crate::modules::sms::service::send_sms_async(
                                config.clone(),
                                ph,
                                format!("{}: {}", payload.title, payload.content),
                            );
                        }
                    }
                }
            }
        }
    }

    Ok(announcement)
}

pub async fn list_announcements(pool: &PgPool, auth_user: &AuthUser) -> Result<Vec<Announcement>, AppError> {
    if auth_user.role == UserRole::Admin {
        repository::find_announcements_for_admin(pool)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to fetch announcements: {}", e)))
    } else {
        let role_str = format!("{:?}", auth_user.role).to_lowercase();
        let sub_role_str = auth_user.sub_role.as_ref().map(|s| format!("{:?}", s).to_lowercase()).unwrap_or_default();

        let class_name: Option<String> = if auth_user.role == UserRole::Student {
            sqlx::query_scalar("SELECT class_name FROM students WHERE id = (SELECT id FROM users WHERE id = $1) LIMIT 1")
                .bind(auth_user.id)
                .fetch_optional(pool)
                .await
                .unwrap_or(None)
        } else {
            None
        };
        let class_str = class_name.unwrap_or_default();

        repository::find_announcements_for_user(pool, &role_str, &sub_role_str, &class_str)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to fetch announcements: {}", e)))
    }
}

pub async fn delete_announcement(pool: &PgPool, id: Uuid) -> Result<(), AppError> {
    let affected = repository::delete_announcement(pool, id)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to delete announcement: {}", e)))?;

    if affected == 0 {
        return Err(AppError::NotFound("Announcement not found".to_string()));
    }

    Ok(())
}

pub async fn list_permissions(pool: &PgPool) -> Result<Vec<AnnouncementPermission>, AppError> {
    repository::find_broadcast_permissions(pool)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to fetch permissions: {}", e)))
}

pub async fn update_permissions(pool: &PgPool, payload: UpdatePermissionsPayload) -> Result<(), AppError> {
    for perm in payload.permissions {
        repository::upsert_broadcast_permission(pool, &perm.sub_role.to_lowercase(), perm.can_broadcast)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to update permission: {}", e)))?;
    }
    Ok(())
}

// ─── Notification Services ───────────────────────────────────────────────────

pub async fn get_user_notifications(pool: &PgPool, user_id: Uuid) -> Result<NotificationListResponse, AppError> {
    let notifications = repository::find_user_notifications(pool, user_id)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to fetch notifications: {}", e)))?;

    let unread_count = repository::count_unread_notifications(pool, user_id)
        .await
        .unwrap_or(0);

    Ok(NotificationListResponse {
        unread_count,
        notifications,
    })
}

pub async fn mark_notification_read(pool: &PgPool, id: Uuid, user_id: Uuid) -> Result<(), AppError> {
    repository::mark_notification_read(pool, id, user_id)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to mark notification read: {}", e)))?;

    Ok(())
}

pub async fn mark_all_notifications_read(pool: &PgPool, user_id: Uuid) -> Result<(), AppError> {
    repository::mark_all_notifications_read(pool, user_id)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to mark all notifications read: {}", e)))?;

    Ok(())
}
