use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use tracing::info;
use uuid::Uuid;

use crate::AppState;
use crate::errors::AppError;
use crate::middleware::AuthUser;
use crate::modules::auth::models::{ApiResponse, UserRole};
use super::models::*;

// ─── ANNOUNCEMENT HANDLERS ───────────────────────────────────────────────────

/// Create & Broadcast Announcement to Targeted Audience (Admin & Staff)
pub async fn create_announcement(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(payload): Json<CreateAnnouncementPayload>,
) -> Result<impl IntoResponse, AppError> {
    // 0. Permission check: Admin can always broadcast. Staff sub-roles require explicit permission.
    if auth_user.role != UserRole::Admin {
        let sub_role = auth_user.sub_role.map(|s| format!("{:?}", s).to_lowercase()).unwrap_or_default();
        let perm: Option<(bool,)> = sqlx::query_as(
            "SELECT can_broadcast FROM announcement_permissions WHERE sub_role = $1"
        )
        .bind(&sub_role)
        .fetch_optional(&state.db)
        .await?;

        let can_broadcast = perm.map(|p| p.0).unwrap_or(false);
        if !can_broadcast {
            return Err(AppError::Forbidden("Only administrators and authorized staff roles are permitted to broadcast announcements.".to_string()));
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

    // 1. Insert announcement record
    let announcement: Announcement = sqlx::query_as(
        r#"
        INSERT INTO announcements (
            title, content, priority, target_roles, target_sub_roles, target_class_names, send_email, send_sms, created_by
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9
        ) RETURNING *
        "#
    )
    .bind(&payload.title)
    .bind(&payload.content)
    .bind(&priority)
    .bind(&target_roles_json)
    .bind(&target_sub_roles_json)
    .bind(&target_class_names_json)
    .bind(send_email)
    .bind(send_sms)
    .bind(auth_user.id)
    .fetch_one(&state.db)
    .await?;

    // 2. Resolve Target Users for Notification & Dispatch
    let users_query = r#"
        SELECT u.id, u.email, u.role::text, u.sub_role::text, s.phone, s.class_name
        FROM users u
        LEFT JOIN students s ON LOWER(TRIM(s.email)) = LOWER(TRIM(u.email))
        WHERE u.is_active = true
    "#;
    
    let target_users = sqlx::query_as::<_, TargetUserInfo>(users_query)
        .fetch_all(&state.db)
        .await?;

    let is_all_roles = target_roles.iter().any(|r| r == "all" || r == "everyone");
    let is_all_sub_roles = target_sub_roles.iter().any(|sr| sr == "all" || sr == "everyone");
    let is_all_classes = target_class_names.iter().any(|c| c == "all" || c == "everyone");

    let mut notification_count = 0usize;

    for u in target_users {
        let matches_role = is_all_roles || target_roles.iter().any(|r| r.eq_ignore_ascii_case(&u.role));
        if !matches_role {
            continue;
        }

        // Sub-role matching for staff
        if u.role.eq_ignore_ascii_case("staff") || u.role.eq_ignore_ascii_case("admin") {
            let user_sub_role = u.sub_role.as_deref().unwrap_or("general");
            let matches_sub_role = is_all_sub_roles || target_sub_roles.iter().any(|sr| sr.eq_ignore_ascii_case(user_sub_role));
            if !matches_sub_role {
                continue;
            }
        }

        // Class matching for students
        if u.role.eq_ignore_ascii_case("student") {
            let user_class = u.class_name.as_deref().unwrap_or("");
            let matches_class = is_all_classes || target_class_names.iter().any(|c| c.eq_ignore_ascii_case(user_class));
            if !matches_class {
                continue;
            }
        }

        // Insert in-app user notification
        let _ = sqlx::query(
            r#"
            INSERT INTO user_notifications (user_id, announcement_id, title, message, priority)
            VALUES ($1, $2, $3, $4, $5)
            "#
        )
        .bind(u.id)
        .bind(announcement.id)
        .bind(&payload.title)
        .bind(&payload.content)
        .bind(&priority)
        .execute(&state.db)
        .await;

        notification_count += 1;

        // Email Dispatch
        if send_email {
            let html_body = format!(
                r#"<!DOCTYPE html>
<html>
<head><meta charset="utf-8"><title>{}</title></head>
<body style="font-family: Arial, sans-serif; background-color: #f8fafc; margin: 0; padding: 20px; color: #1e293b;">
    <div style="max-width: 600px; margin: 0 auto; background: #ffffff; border-radius: 12px; border: 1px solid #e2e8f0; overflow: hidden;">
        <div style="background-color: #0f172a; padding: 24px; text-align: center;">
            <h1 style="color: #ffffff; font-size: 18px; margin: 0;">Pratibha Institute Announcement</h1>
        </div>
        <div style="padding: 28px;">
            <h2 style="font-size: 16px; color: #0f172a; margin-top: 0;">{}</h2>
            <p style="font-size: 14px; line-height: 1.6; color: #334155; white-space: pre-wrap;">{}</p>
        </div>
        <div style="background-color: #f8fafc; padding: 16px; text-align: center; font-size: 12px; color: #94a3b8;">
            Pratibha Institute of Nursing ERP • Official Broadcast
        </div>
    </div>
</body>
</html>"#,
                payload.title, payload.title, payload.content
            );
            crate::modules::email::service::send_email_async(
                state.config.clone(),
                u.email.clone(),
                format!("[Announcement] {}", payload.title),
                html_body
            );
        }

        // SMS Dispatch
        if send_sms {
            if let Some(ref phone) = u.phone {
                if !phone.trim().is_empty() {
                    let sms_text = format!("Pratibha Inst Announcement: {} - {}", payload.title, payload.content);
                    let truncated_sms = if sms_text.len() > 155 {
                        format!("{}...", &sms_text[..150])
                    } else {
                        sms_text
                    };
                    crate::modules::sms::service::send_sms_async(
                        state.config.clone(),
                        phone.clone(),
                        truncated_sms
                    );
                }
            }
        }
    }

    info!("✅ Announcement '{}' created and dispatched to {} users", payload.title, notification_count);

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "success": true,
            "data": announcement,
            "recipients_notified": notification_count
        })),
    ))
}

/// List All Broadcast Announcements
pub async fn get_announcements(
    State(state): State<AppState>,
    _auth_user: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    let announcements: Vec<Announcement> = sqlx::query_as(
        "SELECT * FROM announcements ORDER BY created_at DESC LIMIT 50"
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(ApiResponse {
        success: true,
        data: announcements,
    }))
}

/// Delete Broadcast Announcement
pub async fn delete_announcement(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    if auth_user.role != UserRole::Admin {
        return Err(AppError::Forbidden("Only admin can delete announcements".to_string()));
    }

    sqlx::query("DELETE FROM announcements WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await?;

    Ok(Json(json!({ "success": true, "message": "Announcement deleted successfully" })))
}

// ─── USER NOTIFICATION FEED HANDLERS ────────────────────────────────────────

/// Get logged in user's in-app portal notifications and unread count
pub async fn get_user_notifications(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    let notifications: Vec<UserNotification> = sqlx::query_as(
        "SELECT * FROM user_notifications WHERE user_id = $1 ORDER BY created_at DESC LIMIT 30"
    )
    .bind(auth_user.id)
    .fetch_all(&state.db)
    .await?;

    let unread_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*)::int8 FROM user_notifications WHERE user_id = $1 AND is_read = false"
    )
    .bind(auth_user.id)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(ApiResponse {
        success: true,
        data: NotificationListResponse {
            unread_count: unread_count.0,
            notifications,
        },
    }))
}

/// Mark single notification as read
pub async fn mark_notification_read(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    sqlx::query(
        "UPDATE user_notifications SET is_read = true WHERE id = $1 AND user_id = $2"
    )
    .bind(id)
    .bind(auth_user.id)
    .execute(&state.db)
    .await?;

    Ok(Json(json!({ "success": true, "message": "Notification marked as read" })))
}

/// Mark all user notifications as read
pub async fn mark_all_notifications_read(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    sqlx::query(
        "UPDATE user_notifications SET is_read = true WHERE user_id = $1"
    )
    .bind(auth_user.id)
    .execute(&state.db)
    .await?;

    Ok(Json(json!({ "success": true, "message": "All notifications marked as read" })))
}

/// GET /api/announcements/permissions (Admin only)
pub async fn get_broadcast_permissions(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    if auth_user.role != UserRole::Admin {
        return Err(AppError::Forbidden("Only administrators can view broadcast permissions".to_string()));
    }

    let permissions: Vec<AnnouncementPermission> = sqlx::query_as(
        "SELECT * FROM announcement_permissions ORDER BY sub_role ASC"
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(json!({ "success": true, "data": permissions })))
}

/// PUT /api/announcements/permissions (Admin only)
pub async fn update_broadcast_permissions(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(payload): Json<UpdatePermissionsPayload>,
) -> Result<impl IntoResponse, AppError> {
    if auth_user.role != UserRole::Admin {
        return Err(AppError::Forbidden("Only administrators can update broadcast permissions".to_string()));
    }

    for item in payload.permissions {
        sqlx::query(
            r#"
            INSERT INTO announcement_permissions (sub_role, can_broadcast, updated_at)
            VALUES ($1, $2, NOW())
            ON CONFLICT (sub_role) DO UPDATE SET can_broadcast = $2, updated_at = NOW()
            "#
        )
        .bind(&item.sub_role)
        .bind(item.can_broadcast)
        .execute(&state.db)
        .await?;
    }

    Ok(Json(json!({ "success": true, "message": "Broadcast permissions updated successfully" })))
}

#[derive(Debug, sqlx::FromRow)]
struct TargetUserInfo {
    id: Uuid,
    email: String,
    role: String,
    sub_role: Option<String>,
    phone: Option<String>,
    class_name: Option<String>,
}
