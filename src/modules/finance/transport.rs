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
        let default_class = "B.Sc. Nursing 1st Year";
        let _ = sqlx::query("INSERT INTO classes (name) VALUES ($1) ON CONFLICT (name) DO NOTHING")
            .bind(default_class)
            .execute(db)
            .await;

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
        .bind(format!("Student {}", payload.student_id))
        .bind(default_class)
        .bind(default_dob)
        .execute(db)
        .await;
    }

    if payload.class_name.is_some() || payload.course_name.is_some() || payload.student_name.is_some() {
        let name_val = payload.student_name.as_deref().filter(|s| !s.trim().is_empty() && *s != "—" && !s.contains("Select"));

        let class_val = if let Some(ref c) = payload.class_name {
            let clean = c.trim();
            if !clean.is_empty() && clean != "—" && !clean.to_lowercase().contains("select") {
                let _ = sqlx::query("INSERT INTO classes (name) VALUES ($1) ON CONFLICT (name) DO NOTHING")
                    .bind(clean)
                    .execute(db)
                    .await;
                Some(clean.to_string())
            } else {
                None
            }
        } else {
            None
        };

        let course_val = if let Some(ref cr) = payload.course_name {
            let clean = cr.trim();
            if !clean.is_empty() && clean != "—" && !clean.to_lowercase().contains("select") {
                let _ = sqlx::query("INSERT INTO courses (name) VALUES ($1) ON CONFLICT (name) DO NOTHING")
                    .bind(clean)
                    .execute(db)
                    .await;
                Some(clean.to_string())
            } else {
                None
            }
        } else {
            None
        };

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
        .bind(class_val.as_deref())
        .bind(course_val.as_deref())
        .bind(name_val)
        .bind(&payload.student_id)
        .execute(db)
        .await;
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

    let record = if let Some(row) = existing_fee {
        let existing_id: Uuid = row.get("id");
        sqlx::query_as::<_, FeeRecord>(
            r#"
            UPDATE fee_collections SET
                student_id = $1,
                fee_type = 'transport',
                room = '—',
                bus_route = $2,
                bus_no = $3,
                receipt_book_no = $4,
                receipt_date = $5,
                payment_date = $6,
                amount = $7,
                utr_no = $8,
                payment_mode = $9,
                due_fees = $10,
                remarks = $11,
                discount = $12
            WHERE id = $13
            RETURNING id, student_id, fee_type, room, bus_route, bus_no, receipt_book_no, receipt_no, receipt_date, payment_date,
                      amount::float8 AS amount, utr_no, payment_mode, due_fees::float8 AS due_fees, remarks, discount::float8 AS discount, created_at
            "#
        )
        .bind(&payload.student_id)
        .bind(bus_route)
        .bind(bus_no)
        .bind(receipt_book_no)
        .bind(parsed_receipt_date)
        .bind(parsed_payment_date)
        .bind(payload.amount)
        .bind(utr_no)
        .bind(payment_mode)
        .bind(due_fees)
        .bind(remarks)
        .bind(discount)
        .bind(existing_id)
        .fetch_one(db)
        .await?
    } else {
        sqlx::query_as::<_, FeeRecord>(
            r#"
            INSERT INTO fee_collections (
                student_id, fee_type, room, bus_route, bus_no, 
                receipt_book_no, receipt_no, receipt_date, payment_date, 
                amount, utr_no, payment_mode, due_fees, remarks, discount
            ) VALUES ($1, 'transport', '—', $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            RETURNING id, student_id, fee_type, room, bus_route, bus_no, receipt_book_no, receipt_no, receipt_date, payment_date,
                      amount::float8 AS amount, utr_no, payment_mode, due_fees::float8 AS due_fees, remarks, discount::float8 AS discount, created_at
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
        .fetch_one(db)
        .await?
    };

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
               amount::float8 AS amount, utr_no, payment_mode, due_fees::float8 AS due_fees, remarks, discount::float8 AS discount, created_at
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
        if !cname.trim().is_empty() {
            let _ = sqlx::query("UPDATE students SET class_name = COALESCE(NULLIF($2, ''), class_name) WHERE student_id = $1")
                .bind(&existing.student_id)
                .bind(cname)
                .execute(db)
                .await;
        }
    }
    if let Some(ref crsname) = payload.course_name {
        if !crsname.trim().is_empty() {
            let _ = sqlx::query("UPDATE students SET course_name = COALESCE(NULLIF($2, ''), course_name) WHERE student_id = $1")
                .bind(&existing.student_id)
                .bind(crsname)
                .execute(db)
                .await;
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
            amount = $8, utr_no = $9, payment_mode = $10, due_fees = $11, remarks = $12, discount = $13
        WHERE id = $14
        RETURNING id, student_id, fee_type, room, bus_route, bus_no, receipt_book_no, receipt_no, receipt_date, payment_date,
                  amount::float8 AS amount, utr_no, payment_mode, due_fees::float8 AS due_fees, remarks, discount::float8 AS discount, created_at
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
                  amount::float8 AS amount, utr_no, payment_mode, due_fees::float8 AS due_fees, remarks, discount::float8 AS discount, created_at
        "#
    )
    .bind(id)
    .fetch_one(db)
    .await?;

    Ok(deleted)
}
