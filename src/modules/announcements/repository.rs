// backend-rust/src/modules/announcements/repository.rs
//! Announcements & Notifications Data Access Layer (Repository)

use sqlx::PgPool;
use uuid::Uuid;
use super::models::{Announcement, AnnouncementPermission, UserNotification};

// ─── Announcement Queries ─────────────────────────────────────────────────────

pub async fn find_permission(pool: &PgPool, sub_role: &str) -> Result<Option<bool>, sqlx::Error> {
    let row = sqlx::query_scalar::<_, bool>(
        "SELECT can_broadcast FROM announcement_permissions WHERE LOWER(sub_role) = LOWER($1)"
    )
    .bind(sub_role)
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

pub async fn insert_announcement(
    pool: &PgPool,
    title: &str,
    content: &str,
    priority: &str,
    target_roles: &serde_json::Value,
    target_sub_roles: &serde_json::Value,
    target_class_names: &serde_json::Value,
    send_email: bool,
    send_sms: bool,
    created_by: Uuid,
) -> Result<Announcement, sqlx::Error> {
    sqlx::query_as::<_, Announcement>(
        r#"
        INSERT INTO announcements (
            title, content, priority, target_roles, target_sub_roles, target_class_names, send_email, send_sms, created_by
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9
        ) RETURNING *
        "#
    )
    .bind(title)
    .bind(content)
    .bind(priority)
    .bind(target_roles)
    .bind(target_sub_roles)
    .bind(target_class_names)
    .bind(send_email)
    .bind(send_sms)
    .bind(created_by)
    .fetch_one(pool)
    .await
}

pub async fn find_announcements_for_admin(pool: &PgPool) -> Result<Vec<Announcement>, sqlx::Error> {
    sqlx::query_as::<_, Announcement>(
        "SELECT * FROM announcements ORDER BY created_at DESC LIMIT 50"
    )
    .fetch_all(pool)
    .await
}

pub async fn find_announcements_for_user(
    pool: &PgPool,
    role_str: &str,
    sub_role_str: &str,
    class_name: &str,
) -> Result<Vec<Announcement>, sqlx::Error> {
    sqlx::query_as::<_, Announcement>(
        r#"
        SELECT * FROM announcements
        WHERE (
            target_roles ? 'all' OR target_roles ? $1
        ) AND (
            target_sub_roles ? 'all' OR target_sub_roles ? $2
        ) AND (
            target_class_names ? 'all' OR target_class_names ? $3
        )
        ORDER BY created_at DESC
        LIMIT 50
        "#
    )
    .bind(role_str)
    .bind(sub_role_str)
    .bind(class_name)
    .fetch_all(pool)
    .await
}

pub async fn delete_announcement(pool: &PgPool, id: Uuid) -> Result<u64, sqlx::Error> {
    let res = sqlx::query("DELETE FROM announcements WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(res.rows_affected())
}

pub async fn find_broadcast_permissions(pool: &PgPool) -> Result<Vec<AnnouncementPermission>, sqlx::Error> {
    sqlx::query_as::<_, AnnouncementPermission>(
        "SELECT * FROM announcement_permissions ORDER BY sub_role ASC"
    )
    .fetch_all(pool)
    .await
}

pub async fn upsert_broadcast_permission(
    pool: &PgPool,
    sub_role: &str,
    can_broadcast: bool,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO announcement_permissions (sub_role, can_broadcast)
        VALUES ($1, $2)
        ON CONFLICT (sub_role) DO UPDATE SET can_broadcast = EXCLUDED.can_broadcast, updated_at = NOW()
        "#
    )
    .bind(sub_role)
    .bind(can_broadcast)
    .execute(pool)
    .await?;

    Ok(())
}

// ─── Notification Queries ─────────────────────────────────────────────────────

pub async fn insert_notification(
    pool: &PgPool,
    user_id: Uuid,
    announcement_id: Option<Uuid>,
    title: &str,
    message: &str,
    priority: &str,
) -> Result<UserNotification, sqlx::Error> {
    sqlx::query_as::<_, UserNotification>(
        r#"
        INSERT INTO user_notifications (user_id, announcement_id, title, message, priority)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING *
        "#
    )
    .bind(user_id)
    .bind(announcement_id)
    .bind(title)
    .bind(message)
    .bind(priority)
    .fetch_one(pool)
    .await
}

pub async fn find_user_notifications(pool: &PgPool, user_id: Uuid) -> Result<Vec<UserNotification>, sqlx::Error> {
    sqlx::query_as::<_, UserNotification>(
        "SELECT * FROM user_notifications WHERE user_id = $1 ORDER BY created_at DESC LIMIT 50"
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

pub async fn count_unread_notifications(pool: &PgPool, user_id: Uuid) -> Result<i64, sqlx::Error> {
    let row = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*)::bigint FROM user_notifications WHERE user_id = $1 AND is_read = false"
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(row)
}

pub async fn mark_notification_read(pool: &PgPool, id: Uuid, user_id: Uuid) -> Result<u64, sqlx::Error> {
    let res = sqlx::query("UPDATE user_notifications SET is_read = true WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user_id)
        .execute(pool)
        .await?;

    Ok(res.rows_affected())
}

pub async fn mark_all_notifications_read(pool: &PgPool, user_id: Uuid) -> Result<u64, sqlx::Error> {
    let res = sqlx::query("UPDATE user_notifications SET is_read = true WHERE user_id = $1 AND is_read = false")
        .bind(user_id)
        .execute(pool)
        .await?;

    Ok(res.rows_affected())
}
