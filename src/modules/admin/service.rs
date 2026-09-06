// backend-rust/src/modules/admin/service.rs
//! Admin & Students Business Logic & Domain Service Layer

use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;
use crate::errors::AppError;
use crate::utils::password::hash_password;
use super::models::{
    AdminUserItem, AuditLogItem, CreateAuditLogPayload, CreateStudentPayload,
    CreateStudentResponse, GetAuditLogsQuery, GetStudentsQuery, Student,
    StudentWithCount, UpdateStudentPayload,
};
use super::repository;

// ─── Students Services ────────────────────────────────────────────────────────

pub async fn list_students(
    pool: &PgPool,
    q: &GetStudentsQuery,
) -> Result<(Vec<StudentWithCount>, i32, i32, i32), AppError> {
    let page = q.page.unwrap_or(1).max(1);
    let limit = q.limit.unwrap_or(50).max(1);
    let offset = (page - 1) * limit;

    let list = repository::find_students(pool, q, limit, offset)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to fetch students: {}", e)))?;

    let total = if list.is_empty() { 0 } else { list[0].total_count };

    Ok((list, total, page, limit))
}

pub async fn get_student(pool: &PgPool, id: Uuid) -> Result<Student, AppError> {
    repository::find_student_by_id(pool, id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Student not found".to_string()))
}

pub async fn create_student(
    pool: &PgPool,
    payload: CreateStudentPayload,
) -> Result<CreateStudentResponse, AppError> {
    payload.validate()?;

    let existing = repository::find_student_by_student_id(pool, &payload.student_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    if existing.is_some() {
        return Err(AppError::Conflict(format!(
            "Student with ID {} is already registered",
            payload.student_id
        )));
    }

    let dob = NaiveDate::parse_from_str(&payload.dob, "%Y-%m-%d")
        .map_err(|_| AppError::BadRequest("Invalid dob format".to_string()))?;

    let admission_date = match &payload.admission_date {
        Some(d) if !d.is_empty() => Some(
            NaiveDate::parse_from_str(d, "%Y-%m-%d")
                .map_err(|_| AppError::BadRequest("Invalid admission_date format".to_string()))?,
        ),
        _ => None,
    };

    let student = repository::insert_student(pool, &payload, dob, admission_date)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to create student: {}", e)))?;

    // Auto-create user account if email is provided
    let mut default_password = None;
    if let Some(ref email) = student.email {
        if !email.is_empty() {
            let first_name: String = student
                .name
                .trim()
                .split_whitespace()
                .next()
                .unwrap_or("XX")
                .chars()
                .filter(|c| c.is_alphabetic())
                .collect();
            let initials = format!("{:X>2}", &first_name.to_uppercase()[..first_name.len().min(2)]);
            let dob_part = format!("{}{}{}", dob.format("%d"), dob.format("%m"), dob.format("%Y"));
            let raw_password = format!("{}{}", initials, dob_part);
            if let Ok(password_hash) = hash_password(&raw_password) {
                let _ = repository::upsert_user_account(pool, &student.name, email, &password_hash).await;
                default_password = Some(raw_password);
            }
        }
    }

    // Allocate hostel room if specified
    if let Some(ref room_no) = payload.hostel_room {
        let rn = room_no.trim().to_uppercase();
        if !rn.is_empty() {
            let _ = sqlx::query(
                r#"
                INSERT INTO hostel_students (student_id, room_no, check_in_date, status)
                VALUES ($1, $2, $3, 'active')
                ON CONFLICT (student_id) DO UPDATE SET room_no = EXCLUDED.room_no, status = 'active'
                "#
            )
            .bind(&student.student_id)
            .bind(&rn)
            .bind(chrono::Utc::now().date_naive())
            .execute(pool)
            .await;
        }
    }

    // Allocate transport vehicle if specified
    if let Some(ref vehicle_no) = payload.transport_vehicle {
        let vn = vehicle_no.trim().to_uppercase();
        if !vn.is_empty() {
            let _ = sqlx::query(
                r#"
                INSERT INTO transport_students (student_id, vehicle_no, status)
                VALUES ($1, $2, 'active')
                ON CONFLICT (student_id) DO UPDATE SET vehicle_no = EXCLUDED.vehicle_no, status = 'active'
                "#
            )
            .bind(&student.student_id)
            .bind(&vn)
            .execute(pool)
            .await;
        }
    }

    crate::utils::activity::log_audit(
        pool,
        "Admin",
        "Admin",
        "STUDENT_CREATED",
        "Students",
        &format!("Created student {} ({})", student.name, student.student_id),
    )
    .await?;

    Ok(CreateStudentResponse {
        student,
        default_password,
    })
}

pub async fn update_student(
    pool: &PgPool,
    id: Uuid,
    payload: UpdateStudentPayload,
) -> Result<Student, AppError> {
    payload.validate()?;

    let existing = repository::find_student_by_id(pool, id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Student not found".to_string()))?;

    let dob = match &payload.dob {
        Some(d) if !d.is_empty() => Some(
            NaiveDate::parse_from_str(d, "%Y-%m-%d")
                .map_err(|_| AppError::BadRequest("Invalid dob format".to_string()))?,
        ),
        _ => None,
    };

    let admission_date = match &payload.admission_date {
        Some(d) if !d.is_empty() => Some(
            NaiveDate::parse_from_str(d, "%Y-%m-%d")
                .map_err(|_| AppError::BadRequest("Invalid admission_date format".to_string()))?,
        ),
        _ => None,
    };

    let updated = repository::update_student(pool, id, &payload, dob, admission_date, &existing)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to update student: {}", e)))?
        .ok_or_else(|| AppError::NotFound("Student not found".to_string()))?;

    // Handle hostel room assignment update
    if let Some(ref room_no) = payload.hostel_room {
        let rn = room_no.trim().to_uppercase();
        if rn.is_empty() {
            let _ = sqlx::query("DELETE FROM hostel_students WHERE student_id = $1")
                .bind(&updated.student_id)
                .execute(pool)
                .await;
        } else {
            let _ = sqlx::query(
                r#"
                INSERT INTO hostel_students (student_id, room_no, check_in_date, status)
                VALUES ($1, $2, $3, 'active')
                ON CONFLICT (student_id) DO UPDATE SET room_no = EXCLUDED.room_no, status = 'active'
                "#
            )
            .bind(&updated.student_id)
            .bind(&rn)
            .bind(chrono::Utc::now().date_naive())
            .execute(pool)
            .await;
        }
    }

    // Handle transport vehicle assignment update
    if let Some(ref vehicle_no) = payload.transport_vehicle {
        let vn = vehicle_no.trim().to_uppercase();
        if vn.is_empty() {
            let _ = sqlx::query("DELETE FROM transport_students WHERE student_id = $1")
                .bind(&updated.student_id)
                .execute(pool)
                .await;
        } else {
            let _ = sqlx::query(
                r#"
                INSERT INTO transport_students (student_id, vehicle_no, status)
                VALUES ($1, $2, 'active')
                ON CONFLICT (student_id) DO UPDATE SET vehicle_no = EXCLUDED.vehicle_no, status = 'active'
                "#
            )
            .bind(&updated.student_id)
            .bind(&vn)
            .execute(pool)
            .await;
        }
    }

    crate::utils::activity::log_audit(
        pool,
        "Admin",
        "Admin",
        "STUDENT_UPDATED",
        "Students",
        &format!("Updated student {} ({})", updated.name, updated.student_id),
    )
    .await?;

    Ok(updated)
}

pub async fn delete_student(pool: &PgPool, id: Uuid) -> Result<Student, AppError> {
    let deleted = repository::delete_student(pool, id)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to delete student: {}", e)))?
        .ok_or_else(|| AppError::NotFound("Student not found".to_string()))?;

    let _ = sqlx::query("DELETE FROM hostel_students WHERE student_id = $1")
        .bind(&deleted.student_id)
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM transport_students WHERE student_id = $1")
        .bind(&deleted.student_id)
        .execute(pool)
        .await;

    crate::utils::activity::log_audit(
        pool,
        "Admin",
        "Admin",
        "STUDENT_DELETED",
        "Students",
        &format!("Deleted student {} ({})", deleted.name, deleted.student_id),
    )
    .await?;

    Ok(deleted)
}

pub async fn import_students(
    pool: &PgPool,
    students: Vec<CreateStudentPayload>,
) -> Result<Vec<Student>, AppError> {
    let mut results = Vec::new();

    for s in students {
        let _ = s.validate();

        let dob = match NaiveDate::parse_from_str(&s.dob, "%Y-%m-%d") {
            Ok(d) => d,
            Err(_) => NaiveDate::from_ymd_opt(2000, 1, 1).unwrap(),
        };

        let admission_date = s.admission_date.as_deref().and_then(|d| {
            NaiveDate::parse_from_str(d, "%Y-%m-%d").ok()
        });

        let student = repository::upsert_student_import(pool, &s, dob, admission_date)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to import student {}: {}", s.student_id, e)))?;

        // Auto-create user account if email provided
        if let Some(ref email) = student.email {
            if !email.is_empty() {
                if let Some(dob) = student.dob {
                    let first_name: String = student
                        .name
                        .trim()
                        .split_whitespace()
                        .next()
                        .unwrap_or("XX")
                        .chars()
                        .filter(|c| c.is_alphabetic())
                        .collect();
                    let initials = format!("{:X>2}", &first_name.to_uppercase()[..first_name.len().min(2)]);
                    let dob_part = format!("{}{}{}", dob.format("%d"), dob.format("%m"), dob.format("%Y"));
                    let raw_password = format!("{}{}", initials, dob_part);
                    if let Ok(password_hash) = hash_password(&raw_password) {
                        let _ = repository::upsert_user_account(pool, &student.name, email, &password_hash).await;
                    }
                }
            }
        }

        results.push(student);
    }

    crate::utils::activity::log_audit(
        pool,
        "Admin",
        "Admin",
        "STUDENTS_IMPORTED",
        "Students",
        &format!("Imported {} students", results.len()),
    )
    .await?;

    Ok(results)
}

// ─── User Management Services ────────────────────────────────────────────────

pub async fn list_all_users(pool: &PgPool) -> Result<Vec<AdminUserItem>, AppError> {
    repository::find_all_users(pool)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to fetch users: {}", e)))
}

pub async fn toggle_user_active(
    pool: &PgPool,
    actor_id: Uuid,
    target_id: Uuid,
) -> Result<bool, AppError> {
    if actor_id == target_id {
        return Err(AppError::BadRequest("Cannot deactivate your own account".to_string()));
    }

    repository::toggle_user_active(pool, target_id)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to toggle user status: {}", e)))?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))
}

pub async fn delete_user(
    pool: &PgPool,
    actor_id: Uuid,
    target_id: Uuid,
) -> Result<(), AppError> {
    if actor_id == target_id {
        return Err(AppError::BadRequest("Cannot delete your own account".to_string()));
    }

    let deleted = repository::delete_user(pool, target_id)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to delete user: {}", e)))?;

    if deleted.is_none() {
        return Err(AppError::NotFound("User not found".to_string()));
    }

    Ok(())
}

// ─── Audit Log Services ──────────────────────────────────────────────────────

pub async fn list_audit_logs(
    pool: &PgPool,
    q: &GetAuditLogsQuery,
) -> Result<Vec<AuditLogItem>, AppError> {
    let page = q.page.unwrap_or(1).max(1);
    let limit = q.limit.unwrap_or(100).max(1);
    let offset = (page - 1) * limit;

    repository::find_audit_logs(pool, q, limit, offset)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to fetch audit logs: {}", e)))
}

pub async fn record_audit_log(
    pool: &PgPool,
    user_role_str: &str,
    payload: CreateAuditLogPayload,
) -> Result<(), AppError> {
    crate::utils::activity::log_audit(
        pool,
        "Staff User",
        user_role_str,
        &payload.action,
        &payload.module,
        &payload.details,
    )
    .await
}
