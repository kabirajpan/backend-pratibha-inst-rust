// backend-rust/src/modules/hostel/dto.rs
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CreateHostelRoomPayload {
    pub room_no: String,
    pub block: String,
    pub floor: Option<String>,
    pub capacity: Option<i32>,
    pub room_type: Option<String>,
    pub fee_per_term: Option<f64>,
    pub status: Option<String>,
    pub remarks: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateHostelRoomPayload {
    pub room_no: Option<String>,
    pub block: Option<String>,
    pub floor: Option<String>,
    pub capacity: Option<i32>,
    pub room_type: Option<String>,
    pub fee_per_term: Option<f64>,
    pub status: Option<String>,
    pub remarks: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AddHostelStudentPayload {
    pub student_id: String,
    pub room_no: String,
    pub bed_no: Option<String>,
    pub check_in_date: Option<String>,
    pub fee_amount: Option<f64>,
    pub status: Option<String>,
    pub emergency_contact: Option<String>,
    pub remarks: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateHostelStudentPayload {
    pub room_no: Option<String>,
    pub bed_no: Option<String>,
    pub check_in_date: Option<String>,
    pub fee_amount: Option<f64>,
    pub status: Option<String>,
    pub emergency_contact: Option<String>,
    pub remarks: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GetHostelStudentsQuery {
    pub search: Option<String>,
    pub room_no: Option<String>,
    pub block: Option<String>,
    pub status: Option<String>,
    pub class_name: Option<String>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct GetHostelRoomsQuery {
    pub search: Option<String>,
    pub block: Option<String>,
    pub room_type: Option<String>,
    pub status: Option<String>,
}
