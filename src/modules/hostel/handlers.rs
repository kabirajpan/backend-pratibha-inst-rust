use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::NaiveDate;
use serde_json::json;
use uuid::Uuid;

use crate::AppState;
use crate::errors::AppError;
use crate::middleware::AuthUser;
use crate::modules::auth::models::{ApiResponse, UserSubRole};
use super::models::*;

// ─── HOSTEL ROOMS ──────────────────────────────────────────────────────────

pub async fn get_hostel_rooms(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Query(q): Query<GetHostelRoomsQuery>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize_sub_role(&[UserSubRole::HostelManager, UserSubRole::FinanceManager, UserSubRole::TransportManager])?;

    let mut sql = r#"
        SELECT hr.id, hr.room_no, hr.block, hr.floor, hr.capacity, hr.room_type,
               hr.fee_per_term::float8 AS fee_per_term, hr.status, hr.remarks,
               hr.created_at, hr.updated_at,
               COALESCE((SELECT COUNT(*) FROM hostel_students hs WHERE hs.room_no = hr.room_no AND hs.status = 'active'), 0)::bigint AS occupied_beds
        FROM hostel_rooms hr
        WHERE 1=1
    "#
    .to_string();

    let mut binders = Vec::new();
    let mut idx = 1usize;

    if let Some(ref search) = q.search {
        let term = format!("%{}%", search.trim());
        sql.push_str(&format!(" AND (hr.room_no ILIKE ${idx} OR hr.block ILIKE ${idx} OR hr.floor ILIKE ${idx})"));
        binders.push(term);
        idx += 1;
    }

    if let Some(ref block) = q.block {
        if block != "All Blocks" {
            sql.push_str(&format!(" AND hr.block = ${idx}"));
            binders.push(block.clone());
            idx += 1;
        }
    }

    if let Some(ref rtype) = q.room_type {
        if rtype != "All Types" {
            sql.push_str(&format!(" AND hr.room_type = ${idx}"));
            binders.push(rtype.clone());
            idx += 1;
        }
    }

    if let Some(ref status) = q.status {
        if status != "All Statuses" {
            sql.push_str(&format!(" AND hr.status = ${idx}"));
            binders.push(status.clone());
            idx += 1;
        }
    }

    sql.push_str(" ORDER BY hr.block ASC, hr.room_no ASC");

    let mut db_query = sqlx::query_as::<_, HostelRoomWithOccupancy>(&sql);
    for val in binders {
        db_query = db_query.bind(val);
    }

    let rooms = db_query.fetch_all(&state.db).await?;
    Ok(Json(ApiResponse { success: true, data: rooms }))
}

pub async fn create_hostel_room(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(payload): Json<CreateHostelRoomPayload>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize_sub_role(&[UserSubRole::HostelManager, UserSubRole::FinanceManager, UserSubRole::TransportManager])?;

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

    let record = sqlx::query_as::<_, HostelRoom>(
        r#"
        INSERT INTO hostel_rooms (room_no, block, floor, capacity, room_type, fee_per_term, status, remarks)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        ON CONFLICT (room_no) DO UPDATE
        SET block = EXCLUDED.block,
            floor = EXCLUDED.floor,
            capacity = EXCLUDED.capacity,
            room_type = EXCLUDED.room_type,
            fee_per_term = EXCLUDED.fee_per_term,
            status = EXCLUDED.status,
            remarks = EXCLUDED.remarks,
            updated_at = now()
        RETURNING id, room_no, block, floor, capacity, room_type, fee_per_term::float8 AS fee_per_term, status, remarks, created_at, updated_at
        "#
    )
    .bind(room_no)
    .bind(block)
    .bind(floor)
    .bind(capacity)
    .bind(room_type)
    .bind(fee_per_term)
    .bind(status)
    .bind(remarks)
    .fetch_one(&state.db)
    .await?;

    Ok((StatusCode::CREATED, Json(ApiResponse { success: true, data: record })))
}

pub async fn edit_hostel_room(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateHostelRoomPayload>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize_sub_role(&[UserSubRole::HostelManager, UserSubRole::FinanceManager, UserSubRole::TransportManager])?;

    let existing = sqlx::query_as::<_, HostelRoom>(
        "SELECT id, room_no, block, floor, capacity, room_type, fee_per_term::float8 AS fee_per_term, status, remarks, created_at, updated_at FROM hostel_rooms WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?;

    let mut existing = existing.ok_or_else(|| AppError::NotFound("Hostel room not found".to_string()))?;

    if let Some(rn) = payload.room_no { existing.room_no = rn.trim().to_uppercase(); }
    if let Some(b) = payload.block { existing.block = b.trim().to_string(); }
    if let Some(f) = payload.floor { existing.floor = f.trim().to_string(); }
    if let Some(c) = payload.capacity { existing.capacity = c; }
    if let Some(rt) = payload.room_type { existing.room_type = rt; }
    if let Some(fee) = payload.fee_per_term { existing.fee_per_term = fee; }
    if let Some(st) = payload.status { existing.status = st; }
    if let Some(rem) = payload.remarks { existing.remarks = Some(rem); }

    let record = sqlx::query_as::<_, HostelRoom>(
        r#"
        UPDATE hostel_rooms
        SET room_no = $1, block = $2, floor = $3, capacity = $4, room_type = $5, fee_per_term = $6, status = $7, remarks = $8, updated_at = now()
        WHERE id = $9
        RETURNING id, room_no, block, floor, capacity, room_type, fee_per_term::float8 AS fee_per_term, status, remarks, created_at, updated_at
        "#
    )
    .bind(existing.room_no)
    .bind(existing.block)
    .bind(existing.floor)
    .bind(existing.capacity)
    .bind(existing.room_type)
    .bind(existing.fee_per_term)
    .bind(existing.status)
    .bind(existing.remarks)
    .bind(id)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(ApiResponse { success: true, data: record }))
}

pub async fn remove_hostel_room(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize_sub_role(&[UserSubRole::HostelManager, UserSubRole::FinanceManager, UserSubRole::TransportManager])?;

    let res = sqlx::query("DELETE FROM hostel_rooms WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await?;

    if res.rows_affected() == 0 {
        return Err(AppError::NotFound("Hostel room not found".to_string()));
    }

    Ok(Json(json!({ "success": true, "message": "Hostel room deleted successfully" })))
}

// ─── HOSTEL STUDENTS / RESIDENTS ──────────────────────────────────────────

pub async fn get_hostel_students(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Query(q): Query<GetHostelStudentsQuery>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize_sub_role(&[UserSubRole::HostelManager, UserSubRole::FinanceManager, UserSubRole::TransportManager])?;

    let page = q.page.unwrap_or(1).max(1);
    let limit = q.limit.unwrap_or(50).max(1);
    let offset = (page - 1) * limit;

    let mut sql = r#"
        SELECT hs.id, hs.student_id, hs.room_no, hs.bed_no, hs.check_in_date,
               hs.fee_amount::float8 AS fee_amount, hs.status, hs.emergency_contact, hs.remarks,
               hs.created_at, hs.updated_at,
               COALESCE(s.name, hs.student_id) AS student_name, COALESCE(s.class_name, '—') AS class,
               COUNT(*) OVER()::int AS total_count
        FROM hostel_students hs
        LEFT JOIN students s ON LOWER(TRIM(s.student_id)) = LOWER(TRIM(hs.student_id))
        WHERE 1=1
    "#
    .to_string();

    let mut binders = Vec::new();
    let mut idx = 1usize;

    if let Some(ref search) = q.search {
        let term = format!("%{}%", search.trim());
        sql.push_str(&format!(" AND (COALESCE(s.name, '') ILIKE ${idx} OR hs.student_id ILIKE ${idx} OR hs.room_no ILIKE ${idx} OR COALESCE(hs.bed_no, '') ILIKE ${idx})"));
        binders.push(term);
        idx += 1;
    }

    if let Some(ref room_no) = q.room_no {
        if room_no != "All Rooms" {
            sql.push_str(&format!(" AND hs.room_no = ${idx}"));
            binders.push(room_no.clone());
            idx += 1;
        }
    }

    if let Some(ref status) = q.status {
        if status != "All Statuses" {
            sql.push_str(&format!(" AND hs.status = ${idx}"));
            binders.push(status.clone());
            idx += 1;
        }
    }

    if let Some(ref class_name) = q.class_name {
        if class_name != "All Classes" {
            sql.push_str(&format!(" AND COALESCE(s.class_name, '') = ${idx}"));
            binders.push(class_name.clone());
            idx += 1;
        }
    }

    sql.push_str(" ORDER BY COALESCE(s.name, hs.student_id) ASC");
    sql.push_str(&format!(" LIMIT ${idx} OFFSET ${}", idx + 1));

    let mut db_query = sqlx::query_as::<_, HostelStudentWithDetails>(&sql);
    for val in binders {
        db_query = db_query.bind(val);
    }
    db_query = db_query.bind(limit).bind(offset);

    let list = db_query.fetch_all(&state.db).await?;
    let total = if list.is_empty() { 0 } else { list[0].total_count };

    Ok(Json(json!({
        "success": true,
        "data": list,
        "pagination": {
            "total": total,
            "page": page,
            "limit": limit,
            "pages": (total as f64 / limit as f64).ceil() as i32
        }
    })))
}

pub async fn create_hostel_student(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(payload): Json<AddHostelStudentPayload>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize_sub_role(&[UserSubRole::HostelManager, UserSubRole::FinanceManager, UserSubRole::TransportManager])?;

    if payload.student_id.trim().is_empty() {
        return Err(AppError::BadRequest("Student ID is required".to_string()));
    }
    if payload.room_no.trim().is_empty() {
        return Err(AppError::BadRequest("Room number is required".to_string()));
    }

    let student = sqlx::query_as::<_, (String,)>("SELECT student_id FROM students WHERE LOWER(TRIM(student_id)) = LOWER(TRIM($1))")
        .bind(&payload.student_id)
        .fetch_optional(&state.db)
        .await?;

    let student_id = match student {
        Some((sid,)) => sid,
        None => payload.student_id.trim().to_string(),
    };

    let room_no = payload.room_no.trim().to_uppercase();

    // Ensure room exists in hostel_rooms
    let _ = sqlx::query(
        "INSERT INTO hostel_rooms (room_no, block, floor, capacity, room_type, fee_per_term, status, remarks)
         VALUES ($1, 'Main Hostel Block', '1st Floor', 4, 'Non-AC', 5000.00, 'available', 'Auto-created during resident registration')
         ON CONFLICT (room_no) DO NOTHING"
    )
    .bind(&room_no)
    .execute(&state.db)
    .await;

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

    let record = sqlx::query_as::<_, HostelStudent>(
        r#"
        INSERT INTO hostel_students (student_id, room_no, bed_no, check_in_date, fee_amount, status, emergency_contact, remarks)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        ON CONFLICT (student_id) DO UPDATE
        SET room_no = EXCLUDED.room_no,
            bed_no = EXCLUDED.bed_no,
            check_in_date = EXCLUDED.check_in_date,
            fee_amount = EXCLUDED.fee_amount,
            status = EXCLUDED.status,
            emergency_contact = EXCLUDED.emergency_contact,
            remarks = EXCLUDED.remarks,
            updated_at = now()
        RETURNING id, student_id, room_no, bed_no, check_in_date, fee_amount::float8 AS fee_amount, status, emergency_contact, remarks, created_at, updated_at
        "#
    )
    .bind(&student_id)
    .bind(room_no)
    .bind(bed_no)
    .bind(check_in_date)
    .bind(fee_amount)
    .bind(status)
    .bind(emergency_contact)
    .bind(remarks)
    .fetch_one(&state.db)
    .await?;

    Ok((StatusCode::CREATED, Json(ApiResponse { success: true, data: record })))
}

pub async fn edit_hostel_student(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateHostelStudentPayload>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize_sub_role(&[UserSubRole::HostelManager, UserSubRole::FinanceManager, UserSubRole::TransportManager])?;

    let existing = sqlx::query_as::<_, HostelStudent>(
        "SELECT id, student_id, room_no, bed_no, check_in_date, fee_amount::float8 AS fee_amount, status, emergency_contact, remarks, created_at, updated_at FROM hostel_students WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?;

    let mut existing = existing.ok_or_else(|| AppError::NotFound("Hostel student resident record not found".to_string()))?;

    if let Some(rn) = payload.room_no { existing.room_no = rn.trim().to_uppercase(); }
    if let Some(bn) = payload.bed_no { existing.bed_no = Some(bn); }
    if let Some(ref cd) = payload.check_in_date {
        if let Ok(parsed) = NaiveDate::parse_from_str(cd, "%Y-%m-%d") {
            existing.check_in_date = Some(parsed);
        }
    }
    if let Some(fee) = payload.fee_amount { existing.fee_amount = fee; }
    if let Some(st) = payload.status { existing.status = st; }
    if let Some(ec) = payload.emergency_contact { existing.emergency_contact = Some(ec); }
    if let Some(rem) = payload.remarks { existing.remarks = Some(rem); }

    let record = sqlx::query_as::<_, HostelStudent>(
        r#"
        UPDATE hostel_students
        SET room_no = $1, bed_no = $2, check_in_date = $3, fee_amount = $4, status = $5, emergency_contact = $6, remarks = $7, updated_at = now()
        WHERE id = $8
        RETURNING id, student_id, room_no, bed_no, check_in_date, fee_amount::float8 AS fee_amount, status, emergency_contact, remarks, created_at, updated_at
        "#
    )
    .bind(existing.room_no)
    .bind(existing.bed_no)
    .bind(existing.check_in_date)
    .bind(existing.fee_amount)
    .bind(existing.status)
    .bind(existing.emergency_contact)
    .bind(existing.remarks)
    .bind(id)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(ApiResponse { success: true, data: record }))
}

pub async fn remove_hostel_student(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize_sub_role(&[UserSubRole::HostelManager, UserSubRole::FinanceManager, UserSubRole::TransportManager])?;

    let res = sqlx::query("DELETE FROM hostel_students WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await?;

    if res.rows_affected() == 0 {
        return Err(AppError::NotFound("Hostel student resident record not found".to_string()));
    }

    Ok(Json(json!({ "success": true, "message": "Resident record deleted successfully" })))
}
