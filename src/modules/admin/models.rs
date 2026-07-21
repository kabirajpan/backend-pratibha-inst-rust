use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
    pub photo_url: Option<String>,
    pub signature_url: Option<String>,
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
    pub photo_url: Option<String>,
    pub signature_url: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub total_count: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetStudentsQuery {
    pub search: Option<String>,
    pub class_name: Option<String>,
    pub session: Option<String>,
    pub page: Option<i32>,
    pub limit: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateStudentPayload {
    pub student_id: String,
    pub name: String,
    pub class_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub dob: String,               // Required — used to generate default password
    pub status: Option<String>,
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
    pub admission_date: Option<String>,
    pub session: Option<String>,
    pub course_name: Option<String>,
    pub photo_url: Option<String>,
    pub signature_url: Option<String>,
}

impl CreateStudentPayload {
    pub fn validate(&self) -> Result<(), crate::errors::AppError> {
        if self.student_id.trim().len() < 2 || self.student_id.len() > 50 {
            return Err(crate::errors::AppError::BadRequest("student_id must be 2–50 characters".to_string()));
        }
        if self.name.trim().len() < 2 || self.name.len() > 150 {
            return Err(crate::errors::AppError::BadRequest("name must be 2–150 characters".to_string()));
        }
        if self.dob.trim().is_empty() {
            return Err(crate::errors::AppError::BadRequest("dob is required".to_string()));
        }
        chrono::NaiveDate::parse_from_str(&self.dob, "%Y-%m-%d")
            .map_err(|_| crate::errors::AppError::BadRequest("dob must be in YYYY-MM-DD format".to_string()))?;
        if let Some(ref email) = self.email {
            if !email.is_empty() && !email.contains('@') {
                return Err(crate::errors::AppError::BadRequest("Invalid email address".to_string()));
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
pub struct UpdateStudentPayload {
    pub student_id: Option<String>,
    pub name: Option<String>,
    pub class_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub dob: Option<String>,
    pub status: Option<String>,
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
    pub admission_date: Option<String>,
    pub session: Option<String>,
    pub course_name: Option<String>,
    pub photo_url: Option<String>,
    pub signature_url: Option<String>,
}

impl UpdateStudentPayload {
    pub fn validate(&self) -> Result<(), crate::errors::AppError> {
        if let Some(ref student_id) = self.student_id {
            if student_id.trim().len() < 2 || student_id.len() > 50 {
                return Err(crate::errors::AppError::BadRequest("student_id must be 2–50 characters".to_string()));
            }
        }
        if let Some(ref name) = self.name {
            if name.trim().len() < 2 || name.len() > 150 {
                return Err(crate::errors::AppError::BadRequest("name must be 2–150 characters".to_string()));
            }
        }
        if let Some(ref dob) = self.dob {
            if !dob.is_empty() {
                chrono::NaiveDate::parse_from_str(dob, "%Y-%m-%d")
                    .map_err(|_| crate::errors::AppError::BadRequest("dob must be in YYYY-MM-DD format".to_string()))?;
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
pub struct ImportStudentsPayload {
    pub students: Vec<CreateStudentPayload>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateStudentResponse {
    #[serde(flatten)]
    pub student: Student,
    #[serde(rename = "defaultPassword", skip_serializing_if = "Option::is_none")]
    pub default_password: Option<String>,
}
