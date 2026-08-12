use serde::{Deserialize, Serialize};
use uuid::Uuid;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub role: UserRole,
    #[serde(default)]
    pub sub_role: Option<UserSubRole>,
    pub is_active: bool,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        UserResponse {
            id: user.id,
            name: user.name,
            email: user.email,
            role: user.role,
            sub_role: user.sub_role,
            is_active: user.is_active,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct RegisterPayload {
    pub name: String,
    pub email: String,
    pub password: String,
    #[serde(default)]
    pub role: Option<UserRole>,
    #[serde(default)]
    pub sub_role: Option<UserSubRole>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoginPayload {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChangePasswordPayload {
    #[serde(rename = "currentPassword", default)]
    pub current_password: Option<String>,
    #[serde(rename = "newPassword")]
    pub new_password: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: T,
}

#[derive(Debug, Clone, Serialize)]
pub struct ApiMessageResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct LoginResponseData {
    #[serde(rename = "accessToken")]
    pub access_token: String,
    pub user: UserResponse,
}

#[derive(Debug, Clone, Serialize)]
pub struct RefreshResponseData {
    #[serde(rename = "accessToken")]
    pub access_token: String,
}

// Validation implementations
impl RegisterPayload {
    pub fn validate(&self) -> Result<(), crate::errors::AppError> {
        if self.name.len() < 2 || self.name.len() > 150 {
            return Err(crate::errors::AppError::BadRequest("Name must be between 2 and 150 characters".to_string()));
        }
        if !self.email.contains('@') {
            return Err(crate::errors::AppError::BadRequest("Invalid email address".to_string()));
        }
        if self.password.len() < 8 {
            return Err(crate::errors::AppError::BadRequest("Password must be at least 8 characters long".to_string()));
        }
        let role = self.role.as_ref().unwrap_or(&UserRole::Student);
        if *role == UserRole::Staff {
            if self.sub_role.is_none() {
                return Err(crate::errors::AppError::BadRequest("Sub-role is required for staff members".to_string()));
            }
        } else if self.sub_role.is_some() {
            return Err(crate::errors::AppError::BadRequest("Sub-role is only allowed for staff members".to_string()));
        }
        Ok(())
    }
}

impl LoginPayload {
    pub fn validate(&self) -> Result<(), crate::errors::AppError> {
        if !self.email.contains('@') {
            return Err(crate::errors::AppError::BadRequest("Invalid email address".to_string()));
        }
        if self.password.is_empty() {
            return Err(crate::errors::AppError::BadRequest("Password is required".to_string()));
        }
        Ok(())
    }
}

impl ChangePasswordPayload {
    pub fn validate(&self) -> Result<(), crate::errors::AppError> {
        if let Some(ref current) = self.current_password {
            if current.is_empty() {
                return Err(crate::errors::AppError::BadRequest("Current password is required".to_string()));
            }
        }
        if self.new_password.len() < 8 {
            return Err(crate::errors::AppError::BadRequest("New password must be at least 8 characters long".to_string()));
        }
        Ok(())
    }
}
