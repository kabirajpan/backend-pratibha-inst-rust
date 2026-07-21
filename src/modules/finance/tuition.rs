// src/modules/finance/tuition.rs
use sqlx::PgPool;
use uuid::Uuid;
use crate::errors::AppError;
use super::models::{FeeRecord, AddFeeRecordPayload, UpdateFeeRecordPayload};

pub async fn create_record(
    db: &PgPool,
    fee_type: &str,
    payload: AddFeeRecordPayload,
) -> Result<FeeRecord, AppError> {
    let student = sqlx::query("SELECT id FROM students WHERE student_id = $1")
        .bind(&payload.student_id)
        .fetch_optional(db)
        .await?;

    if student.is_none() {
        return Err(AppError::NotFound(format!("Student with ID {} is not registered", payload.student_id)));
    }

    // 2. Verify receipt_no is unique (or generate a unique fallback if empty or '—')
    let receipt_no = if payload.receipt_no.trim().is_empty() || payload.receipt_no == "—" {
        format!("TU-{}", chrono::Utc::now().timestamp_micros())
    } else {
        let existing = sqlx::query("SELECT id FROM fee_collections WHERE receipt_no = $1")
            .bind(&payload.receipt_no)
            .fetch_optional(db)
            .await?;

        if existing.is_some() {
            return Err(AppError::Conflict(format!("Receipt No. {} has already been logged", payload.receipt_no)));
        }
        payload.receipt_no.clone()
    };

    let parsed_receipt_date = chrono::NaiveDate::parse_from_str(&payload.receipt_date, "%Y-%m-%d")
        .map_err(|_| AppError::BadRequest("Invalid receipt date format".to_string()))?;
    let parsed_payment_date = chrono::NaiveDate::parse_from_str(&payload.payment_date, "%Y-%m-%d")
        .map_err(|_| AppError::BadRequest("Invalid payment date format".to_string()))?;

    let room = payload.room.as_deref().unwrap_or("—");
    let receipt_book_no = payload.receipt_book_no.as_deref().unwrap_or("—");
    let utr_no = payload.utr_no.as_deref().unwrap_or("—");
    let payment_mode = payload.payment_mode.as_deref().unwrap_or("Online");
    let due_fees = payload.due_fees.unwrap_or(0.0);
    let remarks = payload.remarks.as_deref().unwrap_or("—");
    let discount = payload.discount.unwrap_or(0.0);

    let record = sqlx::query_as::<_, FeeRecord>(
        r#"
        INSERT INTO fee_collections (
            student_id, fee_type, room, bus_route, bus_no, 
            receipt_book_no, receipt_no, receipt_date, payment_date, 
            amount, utr_no, payment_mode, due_fees, remarks, discount
        ) VALUES ($1, $2, $3, '—', '—', $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
        RETURNING id, student_id, fee_type, room, bus_route, bus_no, receipt_book_no, receipt_no, receipt_date, payment_date,
                  amount::float8 AS amount, utr_no, payment_mode, due_fees::float8 AS due_fees, remarks, discount::float8 AS discount, created_at
        "#
    )
    .bind(&payload.student_id)
    .bind(fee_type)
    .bind(room)
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
    .await?;

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

    if let Some(fee_type) = payload.fee_type { existing.fee_type = fee_type; }
    if let Some(room) = payload.room { existing.room = room; }
    if let Some(receipt_book_no) = payload.receipt_book_no { existing.receipt_book_no = receipt_book_no; }
    if let Some(remarks) = payload.remarks { existing.remarks = Some(remarks); }
    if let Some(utr_no) = payload.utr_no { existing.utr_no = utr_no; }
    if let Some(payment_mode) = payload.payment_mode { existing.payment_mode = payment_mode; }
    if let Some(amount) = payload.amount { existing.amount = amount; }
    if let Some(due_fees) = payload.due_fees { existing.due_fees = due_fees; }
    if let Some(discount) = payload.discount { existing.discount = discount; }

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
        SET student_id = $1, fee_type = $2, room = $3, bus_route = '—', bus_no = '—', 
            receipt_book_no = $4, receipt_no = $5, receipt_date = $6, payment_date = $7, 
            amount = $8, utr_no = $9, payment_mode = $10, due_fees = $11, remarks = $12, discount = $13
        WHERE id = $14
        RETURNING id, student_id, fee_type, room, bus_route, bus_no, receipt_book_no, receipt_no, receipt_date, payment_date,
                  amount::float8 AS amount, utr_no, payment_mode, due_fees::float8 AS due_fees, remarks, discount::float8 AS discount, created_at
        "#
    )
    .bind(&existing.student_id)
    .bind(&existing.fee_type)
    .bind(&existing.room)
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
    .fetch_optional(db)
    .await?;

    let record = deleted.ok_or_else(|| AppError::NotFound("Fee record not found".to_string()))?;
    Ok(record)
}
