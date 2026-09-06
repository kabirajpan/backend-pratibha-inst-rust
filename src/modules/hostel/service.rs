// backend-rust/src/modules/hostel/service.rs
//! Hostel Business Logic & Domain Service Layer

use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;
use crate::errors::AppError;
use super::models::{
    AddHostelStudentPayload, CreateHostelRoomPayload, GetHostelRoomsQuery,
    GetHostelStudentsQuery, HostelRoom, HostelRoomWithOccupancy,
    HostelStudent, HostelStudentWithDetails, UpdateHostelRoomPayload,
    UpdateHostelStudentPayload,
};
use super::repository;

// ─── Rooms Service ────────────────────────────────────────────────────────────

pub async fn list_rooms(pool: &PgPool, q: &GetHostelRoomsQuery) -> Result<Vec<HostelRoomWithOccupancy>, AppError> {
    repository::find_rooms(pool, q)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to fetch hostel rooms: {}", e)))
}

pub async fn create_or_update_room(pool: &PgPool, payload: CreateHostelRoomPayload) -> Result<HostelRoom, AppError> {
    if payload.room_no.trim().is_empty() {
        return Err(AppError::BadRequest("Room number is required".to_string()));
    }
    if payload.block.trim().is_empty() {
        return Err(AppError::BadRequest("Block name is required".to_string()));
    }

    let room_no = payload.room_no.trim().to_uppercase();
    let block = payload.block.trim().to_string();
    let floor = payload.floor.as_deref().unwrap_or("Ground Floor");
    let capacity = payload.capacity.unwrap_or(4);
    let room_type = payload.room_type.as_deref().unwrap_or("Non-AC");
    let fee_per_term = payload.fee_per_term.unwrap_or(0.0);
    let status = payload.status.as_deref().unwrap_or("available");
    let remarks = payload.remarks.as_deref().unwrap_or("—");

    repository::upsert_room(
        pool,
        &room_no,
        &block,
        floor,
        capacity,
        room_type,
        fee_per_term,
        status,
        remarks,
    )
    .await
    .map_err(|e| AppError::Internal(format!("Failed to save hostel room: {}", e)))
}

pub async fn update_room(pool: &PgPool, id: Uuid, payload: UpdateHostelRoomPayload) -> Result<HostelRoom, AppError> {
    let existing = repository::find_room_by_id(pool, id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Hostel room not found".to_string()))?;

    let mut updated = existing;
    if let Some(rn) = payload.room_no { updated.room_no = rn.trim().to_uppercase(); }
    if let Some(b) = payload.block { updated.block = b.trim().to_string(); }
    if let Some(f) = payload.floor { updated.floor = f.trim().to_string(); }
    if let Some(c) = payload.capacity { updated.capacity = c; }
    if let Some(rt) = payload.room_type { updated.room_type = rt; }
    if let Some(fee) = payload.fee_per_term { updated.fee_per_term = fee; }
    if let Some(st) = payload.status { updated.status = st; }
    if let Some(rem) = payload.remarks { updated.remarks = Some(rem); }

    repository::update_room(pool, &updated)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to update hostel room: {}", e)))
}

pub async fn delete_room(pool: &PgPool, id: Uuid) -> Result<(), AppError> {
    let affected = repository::delete_room(pool, id)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to delete hostel room: {}", e)))?;

    if affected == 0 {
        return Err(AppError::NotFound("Hostel room not found".to_string()));
    }

    Ok(())
}

// ─── Students Resident Service ────────────────────────────────────────────────

pub async fn list_students(
    pool: &PgPool,
    q: &GetHostelStudentsQuery,
) -> Result<(Vec<HostelStudentWithDetails>, i32, i64, i64), AppError> {
    let page = q.page.unwrap_or(1).max(1);
    let limit = q.limit.unwrap_or(50).max(1);
    let offset = (page - 1) * limit;

    let list = repository::find_students(pool, q, limit, offset)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to fetch hostel students: {}", e)))?;

    let total = if list.is_empty() { 0 } else { list[0].total_count };

    Ok((list, total, page, limit))
}

pub async fn allocate_student(pool: &PgPool, payload: AddHostelStudentPayload) -> Result<HostelStudent, AppError> {
    if payload.student_id.trim().is_empty() {
        return Err(AppError::BadRequest("Student ID is required".to_string()));
    }
    if payload.room_no.trim().is_empty() {
        return Err(AppError::BadRequest("Room number is required".to_string()));
    }

    let master_sid = repository::find_master_student_id(pool, &payload.student_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let student_id = master_sid.unwrap_or_else(|| payload.student_id.trim().to_string());
    let room_no = payload.room_no.trim().to_uppercase();

    // Strict validation: room must exist in hostel_rooms
    let room_exists = repository::find_room_by_no(pool, &room_no)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    if room_exists.is_none() {
        return Err(AppError::BadRequest(format!(
            "Room '{}' is not registered in hostel room inventory. Please select or register a valid room first.",
            room_no
        )));
    }

    let bed_no = payload.bed_no.as_deref().unwrap_or("Bed 1");
    let check_in_date = payload
        .check_in_date
        .as_deref()
        .and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .unwrap_or_else(|| chrono::Utc::now().date_naive());

    let fee_amount = payload.fee_amount.unwrap_or(0.0);
    let status = payload.status.as_deref().unwrap_or("active");
    let emergency_contact = payload.emergency_contact.as_deref().unwrap_or("—");
    let remarks = payload.remarks.as_deref().unwrap_or("—");

    repository::upsert_student(
        pool,
        &student_id,
        &room_no,
        bed_no,
        check_in_date,
        fee_amount,
        status,
        emergency_contact,
        remarks,
    )
    .await
    .map_err(|e| AppError::Internal(format!("Failed to save student resident record: {}", e)))
}

pub async fn update_student(
    pool: &PgPool,
    id: Uuid,
    payload: UpdateHostelStudentPayload,
) -> Result<HostelStudent, AppError> {
    let existing = repository::find_student_by_id(pool, id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Hostel student resident record not found".to_string()))?;

    let mut updated = existing;
    if let Some(rn) = payload.room_no { updated.room_no = rn.trim().to_uppercase(); }
    if let Some(bn) = payload.bed_no { updated.bed_no = Some(bn); }
    if let Some(ref cd) = payload.check_in_date {
        if let Ok(parsed) = NaiveDate::parse_from_str(cd, "%Y-%m-%d") {
            updated.check_in_date = Some(parsed);
        }
    }
    if let Some(fee) = payload.fee_amount { updated.fee_amount = fee; }
    if let Some(st) = payload.status { updated.status = st; }
    if let Some(ec) = payload.emergency_contact { updated.emergency_contact = Some(ec); }
    if let Some(rem) = payload.remarks { updated.remarks = Some(rem); }

    repository::update_student(pool, &updated)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to update student resident record: {}", e)))
}

pub async fn delete_student(pool: &PgPool, id: Uuid) -> Result<(), AppError> {
    let affected = repository::delete_student(pool, id)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to delete student resident record: {}", e)))?;

    if affected == 0 {
        return Err(AppError::NotFound("Hostel student resident record not found".to_string()));
    }

    Ok(())
}
