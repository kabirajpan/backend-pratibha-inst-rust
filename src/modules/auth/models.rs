// backend-rust/src/modules/auth/models.rs
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use super::dto::*;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "user_role", rename_all = "snake_case")]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    Admin,
    Staff,
    Student,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "user_sub_role", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum UserSubRole {
    LibraryManager,
    InventoryManager,
    HrManager,
    FinanceManager,
    TransportManager,
    HostelManager,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub password_hash: String,
    pub role: UserRole,
    pub sub_role: Option<UserSubRole>,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
