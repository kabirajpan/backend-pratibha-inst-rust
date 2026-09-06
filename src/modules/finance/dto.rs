// backend-rust/src/modules/finance/dto.rs
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize)]
pub struct AddFeeRecordPayload {
    pub student_id: String,
    pub student_name: Option<String>,
    pub class_name: Option<String>,
    pub course_name: Option<String>,
    pub room: Option<String>,
    pub bus_route: Option<String>,
    pub bus_no: Option<String>,
    pub receipt_book_no: Option<String>,
    pub receipt_no: String,
    pub receipt_date: String,
    pub payment_date: String,
    pub amount: f64,
    pub utr_no: Option<String>,
    pub payment_mode: Option<String>,
    pub due_fees: Option<f64>,
    pub remarks: Option<String>,
    pub discount: Option<f64>,
}

impl AddFeeRecordPayload {
    pub fn validate(&self) -> Result<(), crate::errors::AppError> {
        if self.student_id.trim().is_empty() || self.student_id.len() > 50 {
            return Err(crate::errors::AppError::BadRequest("student_id must be between 1 and 50 characters".to_string()));
        }
        if self.receipt_no.trim().is_empty() || self.receipt_no.len() > 50 {
            return Err(crate::errors::AppError::BadRequest("receipt_no must be between 1 and 50 characters".to_string()));
        }
        chrono::NaiveDate::parse_from_str(&self.receipt_date, "%Y-%m-%d")
            .map_err(|_| crate::errors::AppError::BadRequest("receipt_date must be in YYYY-MM-DD format".to_string()))?;
        chrono::NaiveDate::parse_from_str(&self.payment_date, "%Y-%m-%d")
            .map_err(|_| crate::errors::AppError::BadRequest("payment_date must be in YYYY-MM-DD format".to_string()))?;
        if self.amount < 0.0 {
            return Err(crate::errors::AppError::BadRequest("amount must be positive or zero".to_string()));
        }
        if let Some(due) = self.due_fees {
            if due < 0.0 {
                return Err(crate::errors::AppError::BadRequest("due_fees must be positive or zero".to_string()));
            }
        }
        if let Some(disc) = self.discount {
            if disc < 0.0 {
                return Err(crate::errors::AppError::BadRequest("discount must be positive or zero".to_string()));
            }
        }
        if let Some(ref mode) = self.payment_mode {
            if mode != "Online" && mode != "Cash" && mode != "Bank" && mode != "Cheque" && mode != "Waived" && mode != "UPI" && mode != "Card" {
                return Err(crate::errors::AppError::BadRequest("payment_mode must be Online, Cash, Bank, Cheque, UPI, Card, or Waived".to_string()));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateFeeRecordPayload {
    pub id: Option<Uuid>,
    pub student_id: Option<String>,
    pub student_name: Option<String>,
    pub class_name: Option<String>,
    pub course_name: Option<String>,
    pub fee_type: Option<String>,
    pub room: Option<String>,
    pub bus_route: Option<String>,
    pub bus_no: Option<String>,
    pub receipt_book_no: Option<String>,
    pub receipt_no: Option<String>,
    pub receipt_date: Option<String>,
    pub payment_date: Option<String>,
    pub amount: Option<f64>,
    pub utr_no: Option<String>,
    pub payment_mode: Option<String>,
    pub due_fees: Option<f64>,
    pub remarks: Option<String>,
    pub discount: Option<f64>,
}

impl UpdateFeeRecordPayload {
    pub fn validate(&self) -> Result<(), crate::errors::AppError> {
        if let Some(ref student_id) = self.student_id {
            if student_id.trim().is_empty() || student_id.len() > 50 {
                return Err(crate::errors::AppError::BadRequest("student_id must be between 1 and 50 characters".to_string()));
            }
        }
        if let Some(ref fee_type) = self.fee_type {
            if fee_type != "tuition" && fee_type != "hostel" && fee_type != "transport" && fee_type != "library" {
                return Err(crate::errors::AppError::BadRequest("fee_type must be tuition, hostel, transport, or library".to_string()));
            }
        }
        if let Some(ref r_no) = self.receipt_no {
            if r_no.trim().is_empty() || r_no.len() > 50 {
                return Err(crate::errors::AppError::BadRequest("receipt_no must be between 1 and 50 characters".to_string()));
            }
        }
        if let Some(ref r_date) = self.receipt_date {
            chrono::NaiveDate::parse_from_str(r_date, "%Y-%m-%d")
                .map_err(|_| crate::errors::AppError::BadRequest("receipt_date must be in YYYY-MM-DD format".to_string()))?;
        }
        if let Some(ref p_date) = self.payment_date {
            chrono::NaiveDate::parse_from_str(p_date, "%Y-%m-%d")
                .map_err(|_| crate::errors::AppError::BadRequest("payment_date must be in YYYY-MM-DD format".to_string()))?;
        }
        if let Some(amount) = self.amount {
            if amount < 0.0 {
                return Err(crate::errors::AppError::BadRequest("amount must be positive or zero".to_string()));
            }
        }
        if let Some(due) = self.due_fees {
            if due < 0.0 {
                return Err(crate::errors::AppError::BadRequest("due_fees must be positive or zero".to_string()));
            }
        }
        if let Some(disc) = self.discount {
            if disc < 0.0 {
                return Err(crate::errors::AppError::BadRequest("discount must be positive or zero".to_string()));
            }
        }
        if let Some(ref mode) = self.payment_mode {
            if mode != "Online" && mode != "Cash" && mode != "Bank" && mode != "Cheque" && mode != "Waived" && mode != "UPI" && mode != "Card" {
                return Err(crate::errors::AppError::BadRequest("payment_mode must be Online, Cash, Bank, Cheque, UPI, Card, or Waived".to_string()));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct AddExpensePayload {
    pub description: String,
    pub amount: f64,
    pub category: Option<String>,
    pub date: Option<String>,
    pub payment_mode: Option<String>,
    pub remarks: Option<String>,
    pub utr: Option<String>,
    pub receipt: Option<String>,
    pub party_name: Option<String>,
    pub spent_by: Option<String>,
    pub voucher_no: Option<String>,
}

impl AddExpensePayload {
    pub fn validate(&self) -> Result<(), crate::errors::AppError> {
        if self.description.trim().is_empty() || self.description.len() > 255 {
            return Err(crate::errors::AppError::BadRequest("description is required and must not exceed 255 characters".to_string()));
        }
        if self.amount <= 0.0 {
            return Err(crate::errors::AppError::BadRequest("amount must be greater than 0".to_string()));
        }
        if let Some(ref d) = self.date {
            if !d.is_empty() {
                chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d")
                    .map_err(|_| crate::errors::AppError::BadRequest("date must be in YYYY-MM-DD format".to_string()))?;
            }
        }
        if let Some(ref mode) = self.payment_mode {
            if mode != "Online" && mode != "Cash" && mode != "Bank" && mode != "Cheque" {
                return Err(crate::errors::AppError::BadRequest("payment_mode must be Online, Cash, Bank, or Cheque".to_string()));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateExpensePayload {
    pub description: Option<String>,
    pub amount: Option<f64>,
    pub category: Option<String>,
    pub date: Option<String>,
    pub payment_mode: Option<String>,
    pub remarks: Option<String>,
    pub utr: Option<String>,
    pub receipt: Option<String>,
    pub party_name: Option<String>,
    pub spent_by: Option<String>,
    pub voucher_no: Option<String>,
}

impl UpdateExpensePayload {
    pub fn validate(&self) -> Result<(), crate::errors::AppError> {
        if let Some(ref desc) = self.description {
            if desc.trim().is_empty() || desc.len() > 255 {
                return Err(crate::errors::AppError::BadRequest("description must not exceed 255 characters".to_string()));
            }
        }
        if let Some(amount) = self.amount {
            if amount <= 0.0 {
                return Err(crate::errors::AppError::BadRequest("amount must be greater than 0".to_string()));
            }
        }
        if let Some(ref cat) = self.category {
            if cat.trim().is_empty() || cat.len() > 100 {
                return Err(crate::errors::AppError::BadRequest("category must not exceed 100 characters".to_string()));
            }
        }
        if let Some(ref date_str) = self.date {
            if !date_str.is_empty() {
                chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
                    .map_err(|_| crate::errors::AppError::BadRequest("date must be in YYYY-MM-DD format".to_string()))?;
            }
        }
        if let Some(ref mode) = self.payment_mode {
            if mode != "Online" && mode != "Cash" && mode != "Bank" && mode != "Cheque" {
                return Err(crate::errors::AppError::BadRequest("payment_mode must be Online, Cash, Bank, or Cheque".to_string()));
            }
        }
        Ok(())
    }
}

// ─── QUERY STRUCTS ────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct GetFeesQuery {
    pub search: Option<String>,
    pub class_name: Option<String>,
    pub course_name: Option<String>,
    #[serde(rename = "fromDate")]
    pub from_date: Option<String>,
    #[serde(rename = "toDate")]
    pub to_date: Option<String>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetExpensesQuery {
    pub search: Option<String>,
    pub category: Option<String>,
    #[serde(rename = "fromDate", default)]
    pub from_date: Option<String>,
    #[serde(rename = "toDate", default)]
    pub to_date: Option<String>,
    #[serde(default)]
    pub start_date: Option<String>,
    #[serde(default)]
    pub end_date: Option<String>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
}
