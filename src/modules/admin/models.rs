// backend-rust/src/modules/admin/models.rs
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use super::dto::*;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Student {
    pub id: Uuid,
    pub student_id: String,
    pub name: String,
    pub class_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub dob: Option<chrono::NaiveDate>,
    pub status: String,
    pub gender: Option<String>,
    pub blood_group: Option<String>,
    pub father_name: Option<String>,
    pub mother_name: Option<String>,
    pub parent_phone: Option<String>,
    pub current_address: Option<String>,
    pub permanent_address: Option<String>,
    pub aadhar_no: Option<String>,
    pub bank_name: Option<String>,
    pub account_no: Option<String>,
    pub ifsc_code: Option<String>,
    pub admission_no: Option<String>,
    pub admission_date: Option<chrono::NaiveDate>,
    pub session: Option<String>,
    pub course_name: Option<String>,
    pub year: Option<String>,
    pub photo_url: Option<String>,
    pub signature_url: Option<String>,
    pub tuition_fee: Option<f64>,
    pub tuition_duration: Option<i32>,
    pub tuition_start_date: Option<chrono::NaiveDate>,
    pub tuition_end_date: Option<chrono::NaiveDate>,
    pub transport_fee: Option<f64>,
    pub transport_duration: Option<i32>,
    pub transport_start_date: Option<chrono::NaiveDate>,
    pub transport_end_date: Option<chrono::NaiveDate>,
    pub hostel_fee: Option<f64>,
    pub hostel_duration: Option<i32>,
    pub hostel_start_date: Option<chrono::NaiveDate>,
    pub hostel_end_date: Option<chrono::NaiveDate>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct StudentWithCount {
    pub id: Uuid,
    pub student_id: String,
    pub name: String,
    pub class_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub dob: Option<chrono::NaiveDate>,
    pub status: String,
    pub gender: Option<String>,
    pub blood_group: Option<String>,
    pub father_name: Option<String>,
    pub mother_name: Option<String>,
    pub parent_phone: Option<String>,
    pub current_address: Option<String>,
    pub permanent_address: Option<String>,
    pub aadhar_no: Option<String>,
    pub bank_name: Option<String>,
    pub account_no: Option<String>,
    pub ifsc_code: Option<String>,
    pub admission_no: Option<String>,
    pub admission_date: Option<chrono::NaiveDate>,
    pub session: Option<String>,
    pub course_name: Option<String>,
    pub year: Option<String>,
    pub photo_url: Option<String>,
    pub signature_url: Option<String>,
    pub tuition_fee: Option<f64>,
    pub tuition_duration: Option<i32>,
    pub tuition_start_date: Option<chrono::NaiveDate>,
    pub tuition_end_date: Option<chrono::NaiveDate>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub hostel_room: Option<String>,
    pub hostel_bed: Option<String>,
    pub hostel_fee: Option<f64>,
    pub hostel_duration: Option<i32>,
    pub hostel_start_date: Option<chrono::NaiveDate>,
    pub hostel_end_date: Option<chrono::NaiveDate>,
    pub transport_vehicle: Option<String>,
    pub transport_route: Option<String>,
    pub transport_fee: Option<f64>,
    pub transport_duration: Option<i32>,
    pub transport_start_date: Option<chrono::NaiveDate>,
    pub transport_end_date: Option<chrono::NaiveDate>,
    pub total_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AuditLogItem {
    pub id: Uuid,
    pub user_name: String,
    pub role: String,
    pub action: String,
    pub module: String,
    pub details: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SystemSettings {
    pub id: String,
    pub email_service_enabled: bool,
    pub student_welcome_email_enabled: bool,
    pub fee_receipt_email_enabled: bool,
    pub staff_welcome_email_enabled: bool,
    pub announcement_email_enabled: bool,
    pub sms_service_enabled: bool,
    pub fee_receipt_sms_enabled: bool,
    pub whatsapp_service_enabled: bool,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct StudentCredentialRecord {
    pub student_id: String,
    pub name: String,
    pub email: Option<String>,
    pub dob: Option<chrono::NaiveDate>,
    pub class_name: Option<String>,
}

