// backend-rust/src/modules/todos/models.rs
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use super::dto::*;

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
