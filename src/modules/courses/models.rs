use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Course {
    pub id: Uuid,
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateCoursePayload {
    pub name: String,
}

impl CreateCoursePayload {
    pub fn validate(&self) -> Result<(), crate::errors::AppError> {
        let name = self.name.trim();
        if name.is_empty() {
            return Err(crate::errors::AppError::BadRequest("Course name cannot be empty".to_string()));
        }
        if name.len() > 100 {
            return Err(crate::errors::AppError::BadRequest("Course name cannot exceed 100 characters".to_string()));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateCoursePayload {
    pub name: String,
}

impl UpdateCoursePayload {
    pub fn validate(&self) -> Result<(), crate::errors::AppError> {
        let name = self.name.trim();
        if name.is_empty() {
            return Err(crate::errors::AppError::BadRequest("Course name cannot be empty".to_string()));
        }
        if name.len() > 100 {
            return Err(crate::errors::AppError::BadRequest("Course name cannot exceed 100 characters".to_string()));
        }
        Ok(())
    }
}
