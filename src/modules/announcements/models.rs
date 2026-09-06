// backend-rust/src/modules/announcements/models.rs
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use super::dto::*;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Announcement {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub priority: String,
    pub target_roles: serde_json::Value,
    pub target_sub_roles: serde_json::Value,
    pub target_class_names: serde_json::Value,
    pub send_email: bool,
    pub send_sms: bool,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserNotification {
    pub id: Uuid,
    pub user_id: Uuid,
    pub announcement_id: Option<Uuid>,
    pub title: String,
    pub message: String,
    pub priority: String,
    pub is_read: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AnnouncementPermission {
    pub id: Uuid,
    pub sub_role: String,
    pub can_broadcast: bool,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
