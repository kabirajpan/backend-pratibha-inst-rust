use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Todo {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub module: String,
    pub text: String,
    pub completed: bool,
    pub priority: String,
    pub due_date: Option<chrono::NaiveDate>,
    pub category: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateTodoPayload {
    pub text: String,
    #[serde(default)]
    pub priority: Option<String>,
    #[serde(rename = "dueDate", default)]
    pub due_date: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
}

impl CreateTodoPayload {
    pub fn validate(&self) -> Result<(), crate::errors::AppError> {
        if self.text.trim().is_empty() {
            return Err(crate::errors::AppError::BadRequest("Todo text cannot be empty".to_string()));
        }
        if let Some(ref priority) = self.priority {
            let p = priority.to_lowercase();
            if p != "low" && p != "medium" && p != "high" {
                return Err(crate::errors::AppError::BadRequest("Priority must be low, medium, or high".to_string()));
            }
        }
        if let Some(ref due_date) = self.due_date {
            if !due_date.is_empty() {
                chrono::NaiveDate::parse_from_str(due_date, "%Y-%m-%d")
                    .map_err(|_| crate::errors::AppError::BadRequest("dueDate must be in YYYY-MM-DD format".to_string()))?;
            }
        }
        if let Some(ref category) = self.category {
            if category.len() > 100 {
                return Err(crate::errors::AppError::BadRequest("Category cannot exceed 100 characters".to_string()));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateTodoPayload {
    pub text: Option<String>,
    pub completed: Option<bool>,
    pub priority: Option<String>,
    #[serde(rename = "dueDate", default)]
    pub due_date: Option<String>,
    pub category: Option<String>,
}

impl UpdateTodoPayload {
    pub fn validate(&self) -> Result<(), crate::errors::AppError> {
        if let Some(ref text) = self.text {
            if text.trim().is_empty() {
                return Err(crate::errors::AppError::BadRequest("Todo text cannot be empty".to_string()));
            }
        }
        if let Some(ref priority) = self.priority {
            let p = priority.to_lowercase();
            if p != "low" && p != "medium" && p != "high" {
                return Err(crate::errors::AppError::BadRequest("Priority must be low, medium, or high".to_string()));
            }
        }
        if let Some(ref due_date) = self.due_date {
            if !due_date.is_empty() {
                chrono::NaiveDate::parse_from_str(due_date, "%Y-%m-%d")
                    .map_err(|_| crate::errors::AppError::BadRequest("dueDate must be in YYYY-MM-DD format".to_string()))?;
            }
        }
        if let Some(ref category) = self.category {
            if category.len() > 100 {
                return Err(crate::errors::AppError::BadRequest("Category cannot exceed 100 characters".to_string()));
            }
        }
        Ok(())
    }
}
