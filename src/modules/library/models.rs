// backend-rust/src/modules/library/models.rs
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use super::dto::*;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Book {
    pub id: Uuid,
    pub acc_no: Option<String>,
    pub title: String,
    pub author: String,
    pub subject: Option<String>,
    pub price: f64,
    pub quantity: i32,
    pub added_date: chrono::NaiveDate,
    pub sl_no: Option<String>,
    #[serde(rename = "type")]
    #[sqlx(rename = "type")]
    pub r#type: String,
    pub volume: Option<String>,
    pub number_val: Option<String>,
    pub month: Option<String>,
    pub year: Option<String>,
    pub publisher: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct BookWithStatus {
    pub id: Uuid,
    pub acc_no: Option<String>,
    pub title: String,
    pub author: String,
    pub subject: Option<String>,
    pub price: f64,
    pub quantity: i32,
    pub added_date: chrono::NaiveDate,
    pub sl_no: Option<String>,
    #[serde(rename = "type")]
    #[sqlx(rename = "type")]
    pub r#type: String,
    pub volume: Option<String>,
    pub number_val: Option<String>,
    pub month: Option<String>,
    pub year: Option<String>,
    pub publisher: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub available_quantity: i32,
    pub status: String,
    pub total_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct LibraryMember {
    pub id: Uuid,
    pub student_id: String,
    pub user_id: Option<Uuid>,
    pub name: String,
    pub class: Option<String>,
    pub course: Option<String>,
    pub phone: Option<String>,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct LibraryMemberWithStats {
    pub id: Uuid,
    pub student_id: String,
    pub user_id: Option<Uuid>,
    pub name: String,
    pub class: Option<String>,
    pub course: Option<String>,
    pub phone: Option<String>,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub currently_issued: i64,
    pub total_issued: i64,
    pub total_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct BookIssue {
    pub id: Uuid,
    pub issue_no: String,
    pub member_id: Uuid,
    pub book_id: Uuid,
    pub issued_by: Option<Uuid>,
    pub issue_date: chrono::NaiveDate,
    pub due_date: chrono::NaiveDate,
    pub return_date: Option<chrono::NaiveDate>,
    pub fine_amount: f64,
    pub fine_paid: bool,
    pub status: String,
    pub remarks: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct BookIssueWithDetails {
    pub id: Uuid,
    pub issue_no: String,
    pub member_id: Uuid,
    pub book_id: Uuid,
    pub issued_by: Option<Uuid>,
    pub issue_date: chrono::NaiveDate,
    pub due_date: chrono::NaiveDate,
    pub return_date: Option<chrono::NaiveDate>,
    pub fine_amount: f64,
    pub fine_paid: bool,
    pub status: String,
    pub remarks: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub member_name: String,
    pub student_id: String,
    pub class: Option<String>,
    pub course: Option<String>,
    pub book_title: String,
    pub acc_no: Option<String>,
    pub book_type: String,
    pub book_sl_no: Option<String>,
    pub receipt_book_no: Option<String>,
    pub receipt_no: Option<String>,
    pub receipt_date: Option<chrono::NaiveDate>,
    pub payment_date: Option<chrono::NaiveDate>,
    pub payment_mode: Option<String>,
    pub utr_no: Option<String>,
    pub total_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct LibrarySettings {
    pub id: Uuid,
    #[sqlx(default)]
    pub issue_duration_days: i32,
    #[sqlx(default)]
    pub max_books_per_member: i32,
    #[sqlx(default)]
    pub issue_limit: i32,
    #[sqlx(default)]
    pub return_days: i32,
    pub fine_per_day: f64,
    #[sqlx(default)]
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct LibraryActivityLog {
    pub id: Uuid,
    pub issue_no: String,
    pub action: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub member_name: String,
    pub student_id: String,
    pub book_title: String,
    pub actor_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryStats {
    pub total_books: i64,
    pub unique_titles: i64,
    pub available_books: i64,
    pub issued_books: i64,
    pub overdue_books: i64,
    pub total_members: i64,
    pub active_members: i64,
    pub total_fines: f64,
    pub collected_fines: f64,
    pub pending_fines: f64,
}

pub type CreateBookPayload = AddBookPayload;
pub type CreateMemberPayload = AddMemberPayload;
pub type EditBookIssuePayload = UpdateIssuePayload;
pub type ImportBookRow = ImportedBook;
pub type ImportMemberRow = ImportedMember;
pub type ImportReturnRow = ImportedReturn;
