// src/modules/finance/transport.rs
use sqlx::{PgPool, Row};
use uuid::Uuid;
use crate::errors::AppError;
use crate::utils::activity::log_audit;
use super::models::{FeeRecord, AddFeeRecordPayload, UpdateFeeRecordPayload};

pub async fn create_record(
    db: &PgPool,
    payload: AddFeeRecordPayload,
) -> Result<FeeRecord, AppError> {
    // 1. Verify student exists (auto-create student if missing for bulk imports)
    let student = sqlx::query("SELECT id FROM students WHERE student_id = $1")
        .bind(&payload.student_id)
        .fetch_optional(db)
        .await?;

    if student.is_none() {
        let default_class = sqlx::query_scalar::<_, String>("SELECT name FROM classes ORDER BY name ASC LIMIT 1")
            .fetch_optional(db)
            .await
            .unwrap_or(None)
            .unwrap_or_else(|| "CLASS 8".to_string());
        let default_dob = chrono::NaiveDate::from_ymd_opt(2000, 1, 1).unwrap();
        let _ = sqlx::query(
            r#"
            INSERT INTO students (
                student_id, name, class_name, dob, status
            ) VALUES ($1, $2, $3, $4, 'active')
            ON CONFLICT (student_id) DO NOTHING
            "#
        )
        .bind(&payload.student_id)
        .bind(payload.student_name.as_deref().unwrap_or(&format!("Student {}", payload.student_id)))
        .bind(payload.class_name.as_deref().unwrap_or(&default_class))
        .bind(default_dob)
        .execute(db)
        .await;
    }

    let p_name = payload.student_name.as_ref();
    let p_class = payload.class_name.as_ref();
    let p_course = payload.course_name.as_ref();

    if p_class.is_some() || p_course.is_some() || p_name.is_some() {
        let name_val = p_name.map(|s| s.as_str()).filter(|s| !s.trim().is_empty() && *s != "—" && !s.contains("Select"));

        let class_val = if let Some(ref c) = p_class {
            let clean = c.trim();
            if !clean.is_empty() && clean != "—" && !clean.to_lowercase().contains("select") {
                let existing_cls: Option<(String,)> = sqlx::query_as("SELECT name FROM classes WHERE LOWER(TRIM(name)) = LOWER(TRIM($1)) LIMIT 1")
                    .bind(clean)
                    .fetch_optional(db)
                    .await?;
                if let Some((official_name,)) = existing_cls {
                    Some(official_name)
                } else {
                    return Err(AppError::BadRequest(format!(
                        "Invalid Class Name '{}'. It does not exist in master classes. Please select a valid class option.", clean
                    )));
                }
            } else {
                None
            }
        } else {
            None
        };

        let course_val = if let Some(ref cr) = p_course {
            let clean = cr.trim();
            if !clean.is_empty() && clean != "—" && !clean.to_lowercase().contains("select") {
                let existing_crs: Option<(String,)> = sqlx::query_as("SELECT name FROM courses WHERE LOWER(TRIM(name)) = LOWER(TRIM($1)) LIMIT 1")
                    .bind(clean)
                    .fetch_optional(db)
                    .await?;
                if let Some((official_name,)) = existing_crs {
                    Some(official_name)
                } else {
                    return Err(AppError::BadRequest(format!(
                        "Invalid Course Name '{}'. It does not exist in master courses. Please select a valid course option.", clean
                    )));
                }
            } else {
                None
            }
        } else {
            None
        };

        let existing_student: Option<(Option<String>, Option<String>)> = sqlx::query_as(
            "SELECT class_name, course_name FROM students WHERE TRIM(student_id) = TRIM($1) OR student_id ILIKE $1 LIMIT 1"
        )
        .bind(&payload.student_id)
        .fetch_optional(db)
        .await?;

        let (final_class_update, final_course_update) = if let Some((cur_cls, cur_crs)) = existing_student {
            let update_cls = if cur_cls.as_deref().unwrap_or("").trim().is_empty() { class_val } else { None };
            let update_crs = if cur_crs.as_deref().unwrap_or("").trim().is_empty() { course_val } else { None };
            (update_cls, update_crs)
        } else {
            (class_val, course_val)
        };

        if final_class_update.is_some() || final_course_update.is_some() || name_val.is_some() {
            let _ = sqlx::query(
                r#"
                UPDATE students
                SET class_name  = CASE WHEN $1::text IS NOT NULL THEN $1::text ELSE class_name END,
                    course_name = CASE WHEN $2::text IS NOT NULL THEN $2::text ELSE course_name END,
                    name        = CASE WHEN $3::text IS NOT NULL THEN $3::text ELSE name END,
                    updated_at  = now()
                WHERE TRIM(student_id) = TRIM($4) OR student_id ILIKE $4
                "#
            )
            .bind(final_class_update.as_deref())
            .bind(final_course_update.as_deref())
            .bind(name_val)
            .bind(&payload.student_id)
            .execute(db)
            .await;
        }
    }

    let receipt_no = if payload.receipt_no.trim().is_empty() || payload.receipt_no == "—" {
        format!("TR-{}", chrono::Utc::now().timestamp_micros())
    } else {
        payload.receipt_no.clone()
    };

    let existing_fee = sqlx::query("SELECT id FROM fee_collections WHERE receipt_no = $1")
        .bind(&receipt_no)
        .fetch_optional(db)
        .await?;

    if existing_fee.is_some() {
        return Err(AppError::BadRequest(format!(
            "Receipt No. '{}' already exists in fee records. Please use a unique receipt number, or use the Edit button on the table row to modify existing records.",
            receipt_no
        )));
    }

    let parsed_receipt_date = chrono::NaiveDate::parse_from_str(&payload.receipt_date, "%Y-%m-%d")
        .map_err(|_| AppError::BadRequest("Invalid receipt date format".to_string()))?;
    let parsed_payment_date = chrono::NaiveDate::parse_from_str(&payload.payment_date, "%Y-%m-%d")
        .map_err(|_| AppError::BadRequest("Invalid payment date format".to_string()))?;

    let bus_route = payload.bus_route.as_deref().unwrap_or("—");
    let bus_no = payload.bus_no.as_deref().unwrap_or("—");
    let receipt_book_no = payload.receipt_book_no.as_deref().unwrap_or("—");
    let utr_no = payload.utr_no.as_deref().unwrap_or("—");
    let payment_mode = payload.payment_mode.as_deref().unwrap_or("Online");
    let due_fees = payload.due_fees.unwrap_or(0.0);
    let remarks = payload.remarks.as_deref().unwrap_or("—");
    let discount = payload.discount.unwrap_or(0.0);
    let duration = payload.duration.unwrap_or(1);
    let parsed_start_date = payload.start_date.as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());
    let parsed_end_date = payload.end_date.as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());

    let record = sqlx::query_as::<_, FeeRecord>(
        r#"
        INSERT INTO fee_collections (
            student_id, fee_type, room, bus_route, bus_no, 
            receipt_book_no, receipt_no, receipt_date, payment_date, 
            amount, utr_no, payment_mode, due_fees, remarks, discount,
            duration, start_date, end_date
        ) VALUES ($1, 'transport', '—', $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
        RETURNING id, student_id, fee_type, room, bus_route, bus_no, receipt_book_no, receipt_no, receipt_date, payment_date,
                  amount::float8 AS amount, utr_no, payment_mode, due_fees::float8 AS due_fees, remarks, discount::float8 AS discount,
                  duration, start_date, end_date, created_at
        "#
    )
    .bind(&payload.student_id)
    .bind(bus_route)
    .bind(bus_no)
    .bind(receipt_book_no)
    .bind(&receipt_no)
    .bind(parsed_receipt_date)
    .bind(parsed_payment_date)
    .bind(payload.amount)
    .bind(utr_no)
    .bind(payment_mode)
    .bind(due_fees)
    .bind(remarks)
    .bind(discount)
    .bind(duration)
    .bind(parsed_start_date)
    .bind(parsed_end_date)
    .fetch_one(db)
    .await?;

    let rec_no = &record.receipt_no;
    let _ = log_audit(
        db,
        "Staff User",
        "TransportManager",
        "FEE_COLLECTED",
        "transport",
        &format!("Transport Fee Receipt {} for student {} amount ₹{}", rec_no, record.student_id, record.amount)
    ).await;

    Ok(record)
}

pub async fn update_record(
    db: &PgPool,
    id: Uuid,
    payload: UpdateFeeRecordPayload,
) -> Result<FeeRecord, AppError> {
    let existing = sqlx::query_as::<_, FeeRecord>(
        r#"
        SELECT id, student_id, fee_type, room, bus_route, bus_no, receipt_book_no, receipt_no, receipt_date, payment_date,
               amount::float8 AS amount, utr_no, payment_mode, due_fees::float8 AS due_fees, remarks, discount::float8 AS discount,
               duration, start_date, end_date, created_at
        FROM fee_collections WHERE id = $1
        "#
    )
    .bind(id)
    .fetch_optional(db)
    .await?;

    let mut existing = existing.ok_or_else(|| AppError::NotFound("Fee record not found".to_string()))?;

    if let Some(ref student_id) = payload.student_id {
        let student = sqlx::query("SELECT id FROM students WHERE student_id = $1")
            .bind(student_id)
            .fetch_optional(db)
            .await?;
        if student.is_none() {
            return Err(AppError::NotFound(format!("Student with ID {} is not registered", student_id)));
        }
        existing.student_id = student_id.clone();
    }

    if let Some(ref r_no) = payload.receipt_no {
        if r_no != "—" {
            let duplicate = sqlx::query("SELECT id FROM fee_collections WHERE receipt_no = $1 AND id != $2")
                .bind(r_no)
                .bind(id)
                .fetch_optional(db)
                .await?;
            if duplicate.is_some() {
                return Err(AppError::Conflict("Receipt No. is already assigned to another record".to_string()));
            }
        }
        existing.receipt_no = r_no.clone();
    }

    if let Some(bus_route) = payload.bus_route { existing.bus_route = bus_route; }
    if let Some(bus_no) = payload.bus_no { existing.bus_no = bus_no; }
    if let Some(receipt_book_no) = payload.receipt_book_no { existing.receipt_book_no = receipt_book_no; }
    if let Some(remarks) = payload.remarks { existing.remarks = Some(remarks); }
    if let Some(utr_no) = payload.utr_no { existing.utr_no = utr_no; }
    if let Some(payment_mode) = payload.payment_mode { existing.payment_mode = payment_mode; }
    if let Some(amount) = payload.amount { existing.amount = amount; }
    if let Some(due_fees) = payload.due_fees { existing.due_fees = due_fees; }
    if let Some(discount) = payload.discount { existing.discount = discount; }
    if let Some(dur) = payload.duration { existing.duration = Some(dur); }
    if let Some(ref s_date) = payload.start_date {
        existing.start_date = if s_date.trim().is_empty() { None } else {
            Some(chrono::NaiveDate::parse_from_str(s_date, "%Y-%m-%d")
                .map_err(|_| AppError::BadRequest("Invalid start_date format".to_string()))?)
        };
    }
    if let Some(ref e_date) = payload.end_date {
        existing.end_date = if e_date.trim().is_empty() { None } else {
            Some(chrono::NaiveDate::parse_from_str(e_date, "%Y-%m-%d")
                .map_err(|_| AppError::BadRequest("Invalid end_date format".to_string()))?)
        };
    }

    if let Some(ref sname) = payload.student_name {
        if !sname.trim().is_empty() && !sname.starts_with("Student STU-") {
            let _ = sqlx::query("UPDATE students SET name = COALESCE(NULLIF($2, ''), name) WHERE student_id = $1")
                .bind(&existing.student_id)
                .bind(sname)
                .execute(db)
                .await;
        }
    }
    if let Some(ref cname) = payload.class_name {
        let clean = cname.trim();
        if !clean.is_empty() && clean != "—" && !clean.to_lowercase().contains("select") {
            let existing_cls: Option<(String,)> = sqlx::query_as("SELECT name FROM classes WHERE LOWER(TRIM(name)) = LOWER(TRIM($1)) LIMIT 1")
                .bind(clean)
                .fetch_optional(db)
                .await?;
            if let Some((official_name,)) = existing_cls {
                let cur_cls: Option<(Option<String>,)> = sqlx::query_as("SELECT class_name FROM students WHERE student_id = $1")
                    .bind(&existing.student_id)
                    .fetch_optional(db)
                    .await?;
                let cur = cur_cls.and_then(|(c,)| c).unwrap_or_default();
                if cur.trim().is_empty() {
                    let _ = sqlx::query("UPDATE students SET class_name = $2 WHERE student_id = $1")
                        .bind(&existing.student_id)
                        .bind(official_name)
                        .execute(db)
                        .await;
                }
            } else {
                return Err(AppError::BadRequest(format!(
                    "Invalid Class Name '{}'. It does not exist in master classes. Please select a valid class option.", clean
                )));
            }
        }
    }
    if let Some(ref crsname) = payload.course_name {
        let clean = crsname.trim();
        if !clean.is_empty() && clean != "—" && !clean.to_lowercase().contains("select") {
            let existing_crs: Option<(String,)> = sqlx::query_as("SELECT name FROM courses WHERE LOWER(TRIM(name)) = LOWER(TRIM($1)) LIMIT 1")
                .bind(clean)
                .fetch_optional(db)
                .await?;
            if let Some((official_name,)) = existing_crs {
                let cur_crs: Option<(Option<String>,)> = sqlx::query_as("SELECT course_name FROM students WHERE student_id = $1")
                    .bind(&existing.student_id)
                    .fetch_optional(db)
                    .await?;
                let cur = cur_crs.and_then(|(c,)| c).unwrap_or_default();
                if cur.trim().is_empty() {
                    let _ = sqlx::query("UPDATE students SET course_name = $2 WHERE student_id = $1")
                        .bind(&existing.student_id)
                        .bind(official_name)
                        .execute(db)
                        .await;
                }
            } else {
                return Err(AppError::BadRequest(format!(
                    "Invalid Course Name '{}'. It does not exist in master courses. Please select a valid course option.", clean
                )));
            }
        }
    }

    if let Some(ref r_date) = payload.receipt_date {
        existing.receipt_date = chrono::NaiveDate::parse_from_str(r_date, "%Y-%m-%d")
            .map_err(|_| AppError::BadRequest("Invalid receipt date format".to_string()))?;
    }
    if let Some(ref p_date) = payload.payment_date {
        existing.payment_date = chrono::NaiveDate::parse_from_str(p_date, "%Y-%m-%d")
            .map_err(|_| AppError::BadRequest("Invalid payment date format".to_string()))?;
    }

    let record = sqlx::query_as::<_, FeeRecord>(
        r#"
        UPDATE fee_collections
        SET student_id = $1, fee_type = 'transport', room = '—', bus_route = $2, bus_no = $3, 
            receipt_book_no = $4, receipt_no = $5, receipt_date = $6, payment_date = $7, 
            amount = $8, utr_no = $9, payment_mode = $10, due_fees = $11, remarks = $12, discount = $13,
            duration = $14, start_date = $15, end_date = $16
        WHERE id = $17
        RETURNING id, student_id, fee_type, room, bus_route, bus_no, receipt_book_no, receipt_no, receipt_date, payment_date,
                  amount::float8 AS amount, utr_no, payment_mode, due_fees::float8 AS due_fees, remarks, discount::float8 AS discount,
                  duration, start_date, end_date, created_at
        "#
    )
    .bind(&existing.student_id)
    .bind(&existing.bus_route)
    .bind(&existing.bus_no)
    .bind(&existing.receipt_book_no)
    .bind(&existing.receipt_no)
    .bind(existing.receipt_date)
    .bind(existing.payment_date)
    .bind(existing.amount)
    .bind(&existing.utr_no)
    .bind(&existing.payment_mode)
    .bind(existing.due_fees)
    .bind(existing.remarks)
    .bind(existing.discount)
    .bind(existing.duration)
    .bind(existing.start_date)
    .bind(existing.end_date)
    .bind(id)
    .fetch_one(db)
    .await?;

    Ok(record)
}

pub async fn delete_record(
    db: &PgPool,
    id: Uuid,
) -> Result<FeeRecord, AppError> {
    let deleted = sqlx::query_as::<_, FeeRecord>(
        r#"
        DELETE FROM fee_collections WHERE id = $1
        RETURNING id, student_id, fee_type, room, bus_route, bus_no, receipt_book_no, receipt_no, receipt_date, payment_date,
                  amount::float8 AS amount, utr_no, payment_mode, due_fees::float8 AS due_fees, remarks, discount::float8 AS discount,
                  duration, start_date, end_date, created_at
        "#
    )
    .bind(id)
    .fetch_one(db)
    .await?;

    Ok(deleted)
}
