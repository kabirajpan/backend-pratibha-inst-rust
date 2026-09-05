use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
    pub issue_duration_days: i32,
    pub fine_per_day: f64,
    pub max_books_per_member: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

// ─── PAYLOADS ─────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct AddBookPayload {
    pub acc_no: Option<String>,
    pub title: String,
    pub author: String,
    pub subject: Option<String>,
    #[serde(default)]
    pub price: Option<f64>,
    #[serde(default)]
    pub quantity: Option<i32>,
    pub added_date: Option<String>,
    pub sl_no: Option<String>,
    #[serde(rename = "type", default)]
    pub r#type: Option<String>,
    pub volume: Option<String>,
    pub number_val: Option<String>,
    pub month: Option<String>,
    pub year: Option<String>,
    pub publisher: Option<String>,
}

impl AddBookPayload {
    pub fn validate(&self) -> Result<(), crate::errors::AppError> {
        if self.title.trim().is_empty() || self.title.len() > 255 {
            return Err(crate::errors::AppError::BadRequest("Title is required and must not exceed 255 characters".to_string()));
        }
        if self.author.trim().is_empty() || self.author.len() > 255 {
            return Err(crate::errors::AppError::BadRequest("Author is required and must not exceed 255 characters".to_string()));
        }
        if let Some(ref acc_no) = self.acc_no {
            if acc_no.len() > 20 {
                return Err(crate::errors::AppError::BadRequest("Accession number must not exceed 20 characters".to_string()));
            }
        }
        if let Some(price) = self.price {
            if price < 0.0 {
                return Err(crate::errors::AppError::BadRequest("Price must be at least 0".to_string()));
            }
        }
        if let Some(quantity) = self.quantity {
            if quantity < 1 {
                return Err(crate::errors::AppError::BadRequest("Quantity must be at least 1".to_string()));
            }
        }
        if let Some(ref r#type) = self.r#type {
            let t = r#type.to_lowercase();
            if t != "book" && t != "international_journal" && t != "national_journal" && t != "magazine" {
                return Err(crate::errors::AppError::BadRequest("Invalid book type".to_string()));
            }
        }
        if let Some(ref added_date) = self.added_date {
            if !added_date.is_empty() {
                chrono::NaiveDate::parse_from_str(added_date, "%Y-%m-%d")
                    .map_err(|_| crate::errors::AppError::BadRequest("added_date must be in YYYY-MM-DD format".to_string()))?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateBookPayload {
    pub acc_no: Option<String>,
    pub title: Option<String>,
    pub author: Option<String>,
    pub subject: Option<String>,
    pub price: Option<f64>,
    pub quantity: Option<i32>,
    pub added_date: Option<String>,
    pub sl_no: Option<String>,
    #[serde(rename = "type")]
    pub r#type: Option<String>,
    pub volume: Option<String>,
    pub number_val: Option<String>,
    pub month: Option<String>,
    pub year: Option<String>,
    pub publisher: Option<String>,
}

impl UpdateBookPayload {
    pub fn validate(&self) -> Result<(), crate::errors::AppError> {
        if let Some(ref title) = self.title {
            if title.trim().is_empty() || title.len() > 255 {
                return Err(crate::errors::AppError::BadRequest("Title must not exceed 255 characters".to_string()));
            }
        }
        if let Some(ref author) = self.author {
            if author.trim().is_empty() || author.len() > 255 {
                return Err(crate::errors::AppError::BadRequest("Author must not exceed 255 characters".to_string()));
            }
        }
        if let Some(price) = self.price {
            if price < 0.0 {
                return Err(crate::errors::AppError::BadRequest("Price must be at least 0".to_string()));
            }
        }
        if let Some(quantity) = self.quantity {
            if quantity < 0 {
                return Err(crate::errors::AppError::BadRequest("Quantity must be at least 0".to_string()));
            }
        }
        if let Some(ref r#type) = self.r#type {
            let t = r#type.to_lowercase();
            if t != "book" && t != "international_journal" && t != "national_journal" && t != "magazine" {
                return Err(crate::errors::AppError::BadRequest("Invalid book type".to_string()));
            }
        }
        if let Some(ref added_date) = self.added_date {
            if !added_date.is_empty() {
                chrono::NaiveDate::parse_from_str(added_date, "%Y-%m-%d")
                    .map_err(|_| crate::errors::AppError::BadRequest("added_date must be in YYYY-MM-DD format".to_string()))?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct AddMemberPayload {
    #[serde(rename = "student_id")]
    pub student_id: String,
    pub name: String,
    pub class: Option<String>,
    pub course: Option<String>,
    pub phone: Option<String>,
    #[serde(rename = "user_id")]
    pub user_id: Option<Uuid>,
}

impl AddMemberPayload {
    pub fn validate(&self) -> Result<(), crate::errors::AppError> {
        if self.student_id.trim().is_empty() || self.student_id.len() > 20 {
            return Err(crate::errors::AppError::BadRequest("student_id is required and must not exceed 20 characters".to_string()));
        }
        if self.name.trim().is_empty() || self.name.len() > 150 {
            return Err(crate::errors::AppError::BadRequest("name is required and must not exceed 150 characters".to_string()));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateMemberPayload {
    pub name: Option<String>,
    pub class: Option<String>,
    pub course: Option<String>,
    pub phone: Option<String>,
    pub status: Option<String>,
}

impl UpdateMemberPayload {
    pub fn validate(&self) -> Result<(), crate::errors::AppError> {
        if let Some(ref status) = self.status {
            if status != "active" && status != "inactive" {
                return Err(crate::errors::AppError::BadRequest("Status must be active or inactive".to_string()));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct IssueBookPayload {
    #[serde(rename = "member_id")]
    pub member_id: String,
    #[serde(rename = "book_id")]
    pub book_id: Uuid,
    #[serde(rename = "issue_date")]
    pub issue_date: Option<String>,
    #[serde(rename = "due_date")]
    pub due_date: Option<String>,
}

impl IssueBookPayload {
    pub fn validate(&self) -> Result<(), crate::errors::AppError> {
        if self.member_id.trim().is_empty() {
            return Err(crate::errors::AppError::BadRequest("member_id is required".to_string()));
        }
        if let Some(ref issue_date) = self.issue_date {
            chrono::NaiveDate::parse_from_str(issue_date, "%Y-%m-%d")
                .map_err(|_| crate::errors::AppError::BadRequest("issue_date must be YYYY-MM-DD".to_string()))?;
        }
        if let Some(ref due_date) = self.due_date {
            chrono::NaiveDate::parse_from_str(due_date, "%Y-%m-%d")
                .map_err(|_| crate::errors::AppError::BadRequest("due_date must be YYYY-MM-DD".to_string()))?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ReturnBookPayload {
    #[serde(rename = "issue_id")]
    pub issue_id: Uuid,
    #[serde(rename = "fine_paid", default)]
    pub fine_paid: bool,
    pub remarks: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateIssuePayload {
    pub issue_date: Option<String>,
    pub due_date: Option<String>,
    pub return_date: Option<String>,
    pub status: Option<String>,
    pub fine_amount: Option<f64>,
    pub fine_paid: Option<bool>,
    pub payment_mode: Option<String>,
    pub utr_no: Option<String>,
    pub amount: Option<f64>,
    pub due_fees: Option<f64>,
    pub receipt_book_no: Option<String>,
    pub receipt_no: Option<String>,
    pub receipt_date: Option<String>,
    pub payment_date: Option<String>,
    pub remarks: Option<String>,
}

impl UpdateIssuePayload {
    pub fn validate(&self) -> Result<(), crate::errors::AppError> {
        if let Some(ref d) = self.issue_date {
            chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d")
                .map_err(|_| crate::errors::AppError::BadRequest("issue_date must be YYYY-MM-DD".to_string()))?;
        }
        if let Some(ref d) = self.due_date {
            chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d")
                .map_err(|_| crate::errors::AppError::BadRequest("due_date must be YYYY-MM-DD".to_string()))?;
        }
        if let Some(ref d) = self.return_date {
            if !d.is_empty() && d != "—" && d != "-" {
                chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d")
                    .map_err(|_| crate::errors::AppError::BadRequest("return_date must be YYYY-MM-DD".to_string()))?;
            }
        }
        if let Some(ref status) = self.status {
            let s = status.to_lowercase();
            if s != "issued" && s != "returned" && s != "overdue" {
                return Err(crate::errors::AppError::BadRequest("Invalid issue status".to_string()));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateSettingsPayload {
    #[serde(rename = "issue_duration_days")]
    pub issue_duration_days: i32,
    #[serde(rename = "fine_per_day")]
    pub fine_per_day: f64,
    #[serde(rename = "max_books_per_member")]
    pub max_books_per_member: i32,
}

impl UpdateSettingsPayload {
    pub fn validate(&self) -> Result<(), crate::errors::AppError> {
        if self.issue_duration_days < 1 {
            return Err(crate::errors::AppError::BadRequest("Issue duration must be at least 1 day".to_string()));
        }
        if self.fine_per_day < 0.0 {
            return Err(crate::errors::AppError::BadRequest("Fine per day must be at least 0".to_string()));
        }
        if self.max_books_per_member < 1 {
            return Err(crate::errors::AppError::BadRequest("Max books per member must be at least 1".to_string()));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateFinePayload {
    #[serde(rename = "fine_paid")]
    pub fine_paid: bool,
    pub remarks: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ImportedBook {
    pub acc_no: String,
    pub title: String,
    pub author: String,
    pub subject: Option<String>,
    #[serde(default)]
    pub price: Option<f64>,
    #[serde(default)]
    pub quantity: Option<i32>,
    pub added_date: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ImportBooksPayload {
    pub books: Vec<ImportedBook>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ImportedMember {
    #[serde(rename = "student_id")]
    pub student_id: String,
    pub name: String,
    pub class: Option<String>,
    pub course: Option<String>,
    pub phone: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ImportMembersPayload {
    pub members: Vec<ImportedMember>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ImportedReturn {
    #[serde(rename = "issue_no")]
    pub issue_no: String,
    #[serde(rename = "fine_paid", default)]
    pub fine_paid: bool,
    pub remarks: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ImportReturnsPayload {
    pub returns: Vec<ImportedReturn>,
}

// ─── QUERY STRUCTS ────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct GetBooksQuery {
    pub r#type: Option<String>,
    pub search: Option<String>,
    pub page: Option<i32>,
    pub limit: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetMembersQuery {
    pub search: Option<String>,
    pub page: Option<i32>,
    pub limit: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetIssuesQuery {
    pub status: Option<String>,
    #[serde(rename = "startDate")]
    pub start_date: Option<String>,
    #[serde(rename = "endDate")]
    pub end_date: Option<String>,
    pub page: Option<i32>,
    pub limit: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetStatsQuery {
    #[serde(rename = "startDate")]
    pub start_date: Option<String>,
    #[serde(rename = "endDate")]
    pub end_date: Option<String>,
}
