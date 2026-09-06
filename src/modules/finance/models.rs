// backend-rust/src/modules/finance/models.rs
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use super::dto::*;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct FeeRecord {
    pub id: Uuid,
    pub student_id: String,
    pub fee_type: String,
    pub room: String,
    pub bus_route: String,
    pub bus_no: String,
    pub receipt_book_no: String,
    pub receipt_no: String,
    pub receipt_date: chrono::NaiveDate,
    pub payment_date: chrono::NaiveDate,
    pub amount: f64,
    pub utr_no: String,
    pub payment_mode: String,
    pub due_fees: f64,
    pub remarks: Option<String>,
    pub discount: f64,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct FeeRecordWithDetails {
    pub id: Uuid,
    pub student_id: String,
    pub fee_type: String,
    pub room: String,
    pub bus_route: String,
    pub bus_no: String,
    pub receipt_book_no: String,
    pub receipt_no: String,
    pub receipt_date: chrono::NaiveDate,
    pub payment_date: chrono::NaiveDate,
    pub amount: f64,
    pub utr_no: String,
    pub payment_mode: String,
    pub due_fees: f64,
    pub remarks: Option<String>,
    pub discount: f64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub student_name: String,
    pub class_name: Option<String>,
    pub course_name: Option<String>,
    pub total_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct LibraryUnionRecord {
    pub id: Uuid,
    pub student_id: String,
    pub student_name: String,
    pub class_name: Option<String>,
    pub course_name: Option<String>,
    pub resource_category: Option<String>,
    pub accession_no: Option<String>,
    pub room: String,
    pub overdue_days: i32,
    pub amount: f64,
    pub due_fees: f64,
    pub payment_mode: String,
    pub remarks: Option<String>,
    pub payment_date: Option<chrono::NaiveDate>,
    pub receipt_date: Option<chrono::NaiveDate>,
    pub receipt_no: String,
    pub receipt_book_no: String,
    pub utr_no: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub total_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct GeneralExpense {
    pub id: Uuid,
    pub ref_no: String,
    pub description: String,
    pub amount: f64,
    pub category: String,
    pub date: chrono::NaiveDate,
    pub payment_mode: String,
    pub remarks: Option<String>,
    pub utr: String,
    pub receipt: String,
    pub party_name: String,
    pub spent_by: String,
    pub voucher_no: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct GeneralExpenseWithCount {
    pub id: Uuid,
    pub ref_no: String,
    pub description: String,
    pub amount: f64,
    pub category: String,
    pub date: chrono::NaiveDate,
    pub payment_mode: String,
    pub remarks: Option<String>,
    pub utr: String,
    pub receipt: String,
    pub party_name: String,
    pub spent_by: String,
    pub voucher_no: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub total_count: i32,
}

pub type Expense = GeneralExpense;
pub type ExpenseWithCount = GeneralExpenseWithCount;
