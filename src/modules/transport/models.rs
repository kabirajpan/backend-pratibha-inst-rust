// backend-rust/src/modules/transport/models.rs
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use super::dto::*;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Vehicle {
    pub id: Uuid,
    pub reg_no: String,
    #[serde(rename = "type")]
    #[sqlx(rename = "type")]
    pub type_val: String,
    pub capacity: i32,
    pub driver: String,
    pub route: String,
    pub status: String,
    pub remarks: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct VehicleWithCount {
    pub id: Uuid,
    pub reg_no: String,
    #[serde(rename = "type")]
    #[sqlx(rename = "type")]
    pub type_val: String,
    pub capacity: i32,
    pub driver: String,
    pub route: String,
    pub status: String,
    pub remarks: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub total_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TransportExpense {
    pub id: Uuid,
    pub date: chrono::NaiveDate,
    pub vehicle_no: String,
    #[serde(rename = "type")]
    #[sqlx(rename = "type")]
    pub type_val: String,
    pub vendor: String,
    pub liters: Option<f64>,
    pub rate: Option<f64>,
    pub amount: f64,
    pub payment_mode: String,
    pub remarks: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub utr_no: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TransportExpenseWithCount {
    pub id: Uuid,
    pub date: chrono::NaiveDate,
    pub vehicle_no: String,
    #[serde(rename = "type")]
    #[sqlx(rename = "type")]
    pub type_val: String,
    pub vendor: String,
    pub liters: Option<f64>,
    pub rate: Option<f64>,
    pub amount: f64,
    pub payment_mode: String,
    pub remarks: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub utr_no: String,
    pub total_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TransportStudent {
    pub id: Uuid,
    pub student_id: String,
    pub vehicle_no: Option<String>,
    pub route: Option<String>,
    pub pickup_point: Option<String>,
    pub fee_amount: f64,
    pub status: String,
    pub remarks: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TransportStudentWithDetails {
    pub id: Uuid,
    pub student_id: String,
    pub vehicle_no: Option<String>,
    pub route: Option<String>,
    pub pickup_point: Option<String>,
    pub fee_amount: f64,
    pub status: String,
    pub remarks: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub student_name: String,
    pub class_name: Option<String>,
    pub course_name: Option<String>,
    pub total_count: i32,
}
