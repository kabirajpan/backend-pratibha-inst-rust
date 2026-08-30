// src/modules/finance/hostel.rs
use sqlx::{PgPool, Row};
use uuid::Uuid;
use crate::errors::AppError;
use super::models::{FeeRecord, AddFeeRecordPayload, UpdateFeeRecordPayload};

pub async fn create_record(
    db: &PgPool,
    payload: AddFeeRecordPayload,
) -> Result<FeeRecord, AppError> {
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

    let receipt_no = if payload.receipt_no.trim().is_empty() || payload.receipt_no == "—" {
        format!("HS-{}", chrono::Utc::now().timestamp_micros())
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

    let room = payload.room.as_deref().unwrap_or("—");
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
                fee_type = 'hostel',
                room = $2,
                receipt_book_no = $3,
                receipt_date = $4,
                payment_date = $5,
                amount = $6,
                utr_no = $7,
                payment_mode = $8,
                due_fees = $9,
                remarks = $10,
                discount = $11
            WHERE id = $12
            RETURNING id, student_id, fee_type, room, bus_route, bus_no, receipt_book_no, receipt_no, receipt_date, payment_date,
                      amount::float8 AS amount, utr_no, payment_mode, due_fees::float8 AS due_fees, remarks, discount::float8 AS discount, created_at
            "#
        )
        .bind(&payload.student_id)
        .bind(room)
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
            ) VALUES ($1, 'hostel', $2, '—', '—', $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING id, student_id, fee_type, room, bus_route, bus_no, receipt_book_no, receipt_no, receipt_date, payment_date,
                      amount::float8 AS amount, utr_no, payment_mode, due_fees::float8 AS due_fees, remarks, discount::float8 AS discount, created_at
            "#
        )
        .bind(&payload.student_id)
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
        .await?
    };

    Ok(record)
}

pub async fn update_record(
    db: &PgPool,
    id: Uuid,
    payload: UpdateFeeRecordPayload,
) -> Result<FeeRecord, AppError> {
    payload.validate()?;

    let existing = sqlx::query("SELECT id FROM fee_collections WHERE id = $1 AND fee_type = 'hostel'")
        .bind(id)
        .fetch_optional(db)
        .await?;

    if existing.is_none() {
        return Err(AppError::NotFound("Hostel fee record not found".to_string()));
    }

    let parsed_receipt_date = match &payload.receipt_date {
        Some(d) => Some(chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d")
            .map_err(|_| AppError::BadRequest("Invalid receipt date format".to_string()))?),
        None => None,
    };
    let parsed_payment_date = match &payload.payment_date {
        Some(d) => Some(chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d")
            .map_err(|_| AppError::BadRequest("Invalid payment date format".to_string()))?),
        None => None,
    };

    let record = sqlx::query_as::<_, FeeRecord>(
        r#"
        UPDATE fee_collections SET
            student_id = COALESCE($1, student_id),
            room = COALESCE($2, room),
            receipt_book_no = COALESCE($3, receipt_book_no),
            receipt_date = COALESCE($4, receipt_date),
            payment_date = COALESCE($5, payment_date),
            amount = COALESCE($6, amount),
            utr_no = COALESCE($7, utr_no),
            payment_mode = COALESCE($8, payment_mode),
            due_fees = COALESCE($9, due_fees),
            remarks = COALESCE($10, remarks),
            discount = COALESCE($11, discount)
        WHERE id = $12
        RETURNING id, student_id, fee_type, room, bus_route, bus_no, receipt_book_no, receipt_no, receipt_date, payment_date,
                  amount::float8 AS amount, utr_no, payment_mode, due_fees::float8 AS due_fees, remarks, discount::float8 AS discount, created_at
        "#
    )
    .bind(payload.student_id.as_deref())
    .bind(payload.room.as_deref())
    .bind(payload.receipt_book_no.as_deref())
    .bind(parsed_receipt_date)
    .bind(parsed_payment_date)
    .bind(payload.amount)
    .bind(payload.utr_no.as_deref())
    .bind(payload.payment_mode.as_deref())
    .bind(payload.due_fees)
    .bind(payload.remarks.as_deref())
    .bind(payload.discount)
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
        DELETE FROM fee_collections WHERE id = $1 AND fee_type = 'hostel'
        RETURNING id, student_id, fee_type, room, bus_route, bus_no, receipt_book_no, receipt_no, receipt_date, payment_date,
                  amount::float8 AS amount, utr_no, payment_mode, due_fees::float8 AS due_fees, remarks, discount::float8 AS discount, created_at
        "#
    )
    .bind(id)
    .fetch_optional(db)
    .await?;

    let record = deleted.ok_or_else(|| AppError::NotFound("Hostel fee record not found".to_string()))?;
    Ok(record)
}
