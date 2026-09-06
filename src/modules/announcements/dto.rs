// backend-rust/src/modules/announcements/dto.rs
use serde::{Deserialize, Serialize};
use super::models::UserNotification;

#[derive(Debug, Clone, Deserialize)]
pub struct CreateAnnouncementPayload {
    pub title: String,
    pub content: String,
    pub priority: Option<String>,
    pub target_roles: Option<Vec<String>>,
    pub target_sub_roles: Option<Vec<String>>,
    pub target_class_names: Option<Vec<String>>,
    pub send_email: Option<bool>,
    pub send_sms: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct NotificationListResponse {
    pub unread_count: i64,
    pub notifications: Vec<UserNotification>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdatePermissionItem {
    pub sub_role: String,
    pub can_broadcast: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdatePermissionsPayload {
    pub permissions: Vec<UpdatePermissionItem>,
}
