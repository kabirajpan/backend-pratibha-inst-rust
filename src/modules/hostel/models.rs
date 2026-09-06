// backend-rust/src/modules/hostel/models.rs
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use super::dto::*;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct HostelRoom {
    pub id: Uuid,
    pub room_no: String,
    pub block: String,
    pub floor: String,
    pub capacity: i32,
    pub room_type: String,
    pub fee_per_term: f64,
    pub status: String,
    pub remarks: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct HostelRoomWithOccupancy {
    pub id: Uuid,
    pub room_no: String,
    pub block: String,
    pub floor: String,
    pub capacity: i32,
    pub room_type: String,
    pub fee_per_term: f64,
    pub status: String,
    pub remarks: Option<String>,
    pub occupied_beds: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct HostelStudent {
    pub id: Uuid,
    pub student_id: String,
    pub room_no: String,
    pub bed_no: Option<String>,
    pub check_in_date: Option<NaiveDate>,
    pub fee_amount: f64,
    pub status: String,
    pub emergency_contact: Option<String>,
    pub remarks: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct HostelStudentWithDetails {
    pub id: Uuid,
    pub student_id: String,
    pub room_no: String,
    pub bed_no: Option<String>,
    pub check_in_date: Option<NaiveDate>,
    pub fee_amount: f64,
    pub status: String,
    pub emergency_contact: Option<String>,
    pub remarks: Option<String>,
    pub student_name: String,
    pub class_name: Option<String>,
    pub course_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub total_count: i32,
}
