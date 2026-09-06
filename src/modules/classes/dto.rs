// backend-rust/src/modules/classes/dto.rs
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct CreateClassPayload {
    pub name: String,
}

impl CreateClassPayload {
    pub fn validate(&self) -> Result<(), crate::errors::AppError> {
        let name = self.name.trim();
        if name.is_empty() {
            return Err(crate::errors::AppError::BadRequest("Class name cannot be empty".to_string()));
        }
        if name.len() > 50 {
            return Err(crate::errors::AppError::BadRequest("Class name cannot exceed 50 characters".to_string()));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateClassPayload {
    pub name: String,
}

impl UpdateClassPayload {
    pub fn validate(&self) -> Result<(), crate::errors::AppError> {
        let name = self.name.trim();
        if name.is_empty() {
            return Err(crate::errors::AppError::BadRequest("Class name cannot be empty".to_string()));
        }
        if name.len() > 50 {
            return Err(crate::errors::AppError::BadRequest("Class name cannot exceed 50 characters".to_string()));
        }
        Ok(())
    }
}
