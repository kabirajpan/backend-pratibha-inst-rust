use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Vehicle {
    pub id: Uuid,
    pub reg_no: String,
    pub type_val: String, // mapped to 'type' column
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
    pub class: Option<String>,
    pub total_count: i32,
}

// ─── PAYLOADS ─────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct AddVehiclePayload {
    pub reg_no: String,
    #[serde(rename = "type")]
    pub type_val: String,
    #[serde(default)]
    pub capacity: Option<i32>,
    pub driver: Option<String>,
    pub route: Option<String>,
    pub status: Option<String>,
    pub remarks: Option<String>,
}

impl AddVehiclePayload {
    pub fn validate(&self) -> Result<(), crate::errors::AppError> {
        if self.reg_no.trim().len() < 2 || self.reg_no.len() > 50 {
            return Err(crate::errors::AppError::BadRequest("reg_no must be between 2 and 50 characters".to_string()));
        }
        let t = self.type_val.trim();
        if t != "Bus" && t != "Mini Bus" && t != "Van" && t != "Other" {
            return Err(crate::errors::AppError::BadRequest("type must be Bus, Mini Bus, Van, or Other".to_string()));
        }
        if let Some(capacity) = self.capacity {
            if capacity < 1 {
                return Err(crate::errors::AppError::BadRequest("capacity must be at least 1".to_string()));
            }
        }
        if let Some(ref status) = self.status {
            if status != "active" && status != "maintenance" {
                return Err(crate::errors::AppError::BadRequest("status must be active or maintenance".to_string()));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateVehiclePayload {
    pub reg_no: Option<String>,
    #[serde(rename = "type")]
    pub type_val: Option<String>,
    pub capacity: Option<i32>,
    pub driver: Option<String>,
    pub route: Option<String>,
    pub status: Option<String>,
    pub remarks: Option<String>,
}

impl UpdateVehiclePayload {
    pub fn validate(&self) -> Result<(), crate::errors::AppError> {
        if let Some(ref reg_no) = self.reg_no {
            if reg_no.trim().len() < 2 || reg_no.len() > 50 {
                return Err(crate::errors::AppError::BadRequest("reg_no must be between 2 and 50 characters".to_string()));
            }
        }
        if let Some(ref type_val) = self.type_val {
            let t = type_val.trim();
            if t != "Bus" && t != "Mini Bus" && t != "Van" && t != "Other" {
                return Err(crate::errors::AppError::BadRequest("type must be Bus, Mini Bus, Van, or Other".to_string()));
            }
        }
        if let Some(capacity) = self.capacity {
            if capacity < 1 {
                return Err(crate::errors::AppError::BadRequest("capacity must be at least 1".to_string()));
            }
        }
        if let Some(ref status) = self.status {
            if status != "active" && status != "maintenance" {
                return Err(crate::errors::AppError::BadRequest("status must be active or maintenance".to_string()));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct AddExpensePayload {
    pub date: String,
    pub vehicle_no: String,
    #[serde(rename = "type")]
    pub type_val: String,
    pub vendor: String,
    pub liters: Option<f64>,
    pub rate: Option<f64>,
    pub amount: f64,
    pub payment_mode: Option<String>,
    pub remarks: Option<String>,
    pub utr_no: Option<String>,
}

impl AddExpensePayload {
    pub fn validate(&self) -> Result<(), crate::errors::AppError> {
        chrono::NaiveDate::parse_from_str(&self.date, "%Y-%m-%d")
            .map_err(|_| crate::errors::AppError::BadRequest("date must be in YYYY-MM-DD format".to_string()))?;
        if self.vehicle_no.trim().is_empty() || self.vehicle_no.len() > 50 {
            return Err(crate::errors::AppError::BadRequest("vehicle_no is required and must not exceed 50 characters".to_string()));
        }
        let t = self.type_val.trim();
        if t != "Fuel / Petrol" && t != "Maintenance & Repairs" && t != "Insurance & Tax" && t != "Miscellaneous" {
            return Err(crate::errors::AppError::BadRequest("type must be Fuel / Petrol, Maintenance & Repairs, Insurance & Tax, or Miscellaneous".to_string()));
        }
        if self.vendor.trim().len() < 2 || self.vendor.len() > 150 {
            return Err(crate::errors::AppError::BadRequest("vendor must be between 2 and 150 characters".to_string()));
        }
        if self.amount <= 0.0 {
            return Err(crate::errors::AppError::BadRequest("amount must be positive".to_string()));
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
    pub date: Option<String>,
    pub vehicle_no: Option<String>,
    #[serde(rename = "type")]
    pub type_val: Option<String>,
    pub vendor: Option<String>,
    pub liters: Option<f64>,
    pub rate: Option<f64>,
    pub amount: Option<f64>,
    pub payment_mode: Option<String>,
    pub remarks: Option<String>,
    pub utr_no: Option<String>,
}

impl UpdateExpensePayload {
    pub fn validate(&self) -> Result<(), crate::errors::AppError> {
        if let Some(ref date) = self.date {
            chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d")
                .map_err(|_| crate::errors::AppError::BadRequest("date must be in YYYY-MM-DD format".to_string()))?;
        }
        if let Some(ref vehicle_no) = self.vehicle_no {
            if vehicle_no.trim().is_empty() || vehicle_no.len() > 50 {
                return Err(crate::errors::AppError::BadRequest("vehicle_no must not exceed 50 characters".to_string()));
            }
        }
        if let Some(ref type_val) = self.type_val {
            let t = type_val.trim();
            if t != "Fuel / Petrol" && t != "Maintenance & Repairs" && t != "Insurance & Tax" && t != "Miscellaneous" {
                return Err(crate::errors::AppError::BadRequest("type must be Fuel / Petrol, Maintenance & Repairs, Insurance & Tax, or Miscellaneous".to_string()));
            }
        }
        if let Some(ref vendor) = self.vendor {
            if vendor.trim().len() < 2 || vendor.len() > 150 {
                return Err(crate::errors::AppError::BadRequest("vendor must be between 2 and 150 characters".to_string()));
            }
        }
        if let Some(amount) = self.amount {
            if amount <= 0.0 {
                return Err(crate::errors::AppError::BadRequest("amount must be positive".to_string()));
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
pub struct AddTransportStudentPayload {
    pub student_id: String,
    pub vehicle_no: Option<String>,
    pub route: Option<String>,
    pub pickup_point: Option<String>,
    pub fee_amount: Option<f64>,
    pub status: Option<String>,
    pub remarks: Option<String>,
}

impl AddTransportStudentPayload {
    pub fn validate(&self) -> Result<(), crate::errors::AppError> {
        if self.student_id.trim().is_empty() || self.student_id.len() > 50 {
            return Err(crate::errors::AppError::BadRequest("student_id is required and must not exceed 50 characters".to_string()));
        }
        if let Some(fee_amount) = self.fee_amount {
            if fee_amount < 0.0 {
                return Err(crate::errors::AppError::BadRequest("fee_amount must be at least 0".to_string()));
            }
        }
        if let Some(ref status) = self.status {
            if status != "active" && status != "inactive" {
                return Err(crate::errors::AppError::BadRequest("status must be active or inactive".to_string()));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateTransportStudentPayload {
    pub student_id: Option<String>,
    pub vehicle_no: Option<String>,
    pub route: Option<String>,
    pub pickup_point: Option<String>,
    pub fee_amount: Option<f64>,
    pub status: Option<String>,
    pub remarks: Option<String>,
}

impl UpdateTransportStudentPayload {
    pub fn validate(&self) -> Result<(), crate::errors::AppError> {
        if let Some(ref student_id) = self.student_id {
            if student_id.trim().is_empty() || student_id.len() > 50 {
                return Err(crate::errors::AppError::BadRequest("student_id must not exceed 50 characters".to_string()));
            }
        }
        if let Some(fee_amount) = self.fee_amount {
            if fee_amount < 0.0 {
                return Err(crate::errors::AppError::BadRequest("fee_amount must be at least 0".to_string()));
            }
        }
        if let Some(ref status) = self.status {
            if status != "active" && status != "inactive" {
                return Err(crate::errors::AppError::BadRequest("status must be active or inactive".to_string()));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ImportTransportStudentRow {
    pub student_id: String,
    pub vehicle_no: Option<String>,
    pub route: Option<String>,
    pub pickup_point: Option<String>,
    pub fee_amount: Option<f64>,
    pub status: Option<String>,
    pub remarks: Option<String>,
}

// ─── QUERY STRUCTS ────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct GetVehiclesQuery {
    pub search: Option<String>,
    pub r#type: Option<String>,
    pub status: Option<String>,
    pub page: Option<i32>,
    pub limit: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetExpensesQuery {
    pub search: Option<String>,
    pub r#type: Option<String>,
    pub vehicle_no: Option<String>,
    #[serde(rename = "fromDate")]
    pub from_date: Option<String>,
    #[serde(rename = "toDate")]
    pub to_date: Option<String>,
    pub page: Option<i32>,
    pub limit: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetTransportStudentsQuery {
    pub search: Option<String>,
    pub route: Option<String>,
    pub status: Option<String>,
    #[serde(rename = "class_name")]
    pub class_name: Option<String>,
    #[serde(rename = "vehicle_no")]
    pub vehicle_no: Option<String>,
    pub page: Option<i32>,
    pub limit: Option<i32>,
}
