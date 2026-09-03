use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use sqlx::Row;
use uuid::Uuid;
use crate::AppState;
use crate::errors::AppError;
use crate::middleware::AuthUser;
use crate::modules::auth::models::{ApiResponse, UserSubRole};
use crate::utils::activity::log_audit;
use super::models::*;
use super::{transport, hostel, tuition};

// ─── FEE COLLECTIONS ──────────────────────────────────────

pub async fn get_fee_records(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(fee_type): Path<String>,
    Query(q): Query<GetFeesQuery>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize_sub_role(&[UserSubRole::FinanceManager, UserSubRole::TransportManager, UserSubRole::HostelManager, UserSubRole::LibraryManager])?;

    let page = q.page.unwrap_or(1).max(1);
    let limit = q.limit.unwrap_or(50).max(1);
    let offset = (page - 1) * limit;

    if fee_type == "library" {
        // Run library union query
        let mut sql = r#"
            WITH library_union AS (
                -- 1. Unpaid fines from book_issues
                SELECT 
                    bi.id,
                    m.student_id,
                    m.name AS student_name,
                    s.class_name AS class,
                    b.title AS room,
                    (CURRENT_DATE - bi.due_date)::int AS overdue_days,
                    0.00::float8 AS amount,
                    bi.fine_amount::float8 AS due_fees,
                    '—' AS payment_mode,
                    bi.remarks,
                    NULL::date AS payment_date,
                    NULL::date AS receipt_date,
                    '—' AS receipt_no,
                    '—' AS receipt_book_no,
                    '—' AS utr_no,
                    bi.created_at
                FROM book_issues bi
                JOIN library_members m ON m.id = bi.member_id
                JOIN books b ON b.id = bi.book_id
                LEFT JOIN students s ON s.student_id = m.student_id
                WHERE bi.fine_amount > 0 AND bi.fine_paid = false

                UNION ALL

                -- 2. Paid fines from fee_collections
                SELECT 
                    f.id,
                    f.student_id,
                    s.name AS student_name,
                    s.class_name AS class,
                    f.room,
                    0::int AS overdue_days,
                    f.amount::float8 AS amount,
                    f.due_fees::float8 AS due_fees,
                    f.payment_mode,
                    f.remarks,
                    f.payment_date,
                    f.receipt_date,
                    f.receipt_no,
                    f.receipt_book_no,
                    f.utr_no,
                    f.created_at
                FROM fee_collections f
                JOIN students s ON s.student_id = f.student_id
                WHERE f.fee_type = 'library'

                UNION ALL

                -- 3. Legacy paid fines from book_issues
                SELECT 
                    bi.id,
                    m.student_id,
                    m.name AS student_name,
                    s.class_name AS class,
                    b.title AS room,
                    0::int AS overdue_days,
                    bi.fine_amount::float8 AS amount,
                    0.00::float8 AS due_fees,
                    'Cash' AS payment_mode,
                    bi.remarks,
                    bi.return_date AS payment_date,
                    bi.return_date AS receipt_date,
                    '—' AS receipt_no,
                    '—' AS receipt_book_no,
                    '—' AS utr_no,
                    bi.created_at
                FROM book_issues bi
                JOIN library_members m ON m.id = bi.member_id
                JOIN books b ON b.id = bi.book_id
                LEFT JOIN students s ON s.student_id = m.student_id
                WHERE bi.fine_amount > 0 
                  AND bi.fine_paid = true
                  AND NOT EXISTS (
                      SELECT 1 FROM fee_collections f 
                      WHERE f.student_id = m.student_id 
                        AND f.fee_type = 'library' 
                        AND f.room = b.title
                  )
            )
            SELECT *, COUNT(*) OVER()::int AS total_count 
            FROM library_union
            WHERE 1=1
        "#.to_string();

        let mut binders = Vec::new();
        let mut idx = 1usize;

        if let Some(ref search) = q.search {
            let term = format!("%{}%", search);
            sql.push_str(&format!(" AND (student_name ILIKE ${idx} OR student_id ILIKE ${idx} OR room ILIKE ${idx})"));
            binders.push(term);
            idx += 1;
        }

        if let Some(ref class_name) = q.class_name {
            if class_name != "All Classes" {
                sql.push_str(&format!(" AND class = ${idx}"));
                binders.push(class_name.clone());
                idx += 1;
            }
        }

        if let Some(ref from_date) = q.from_date {
            if !from_date.is_empty() {
                let parsed = chrono::NaiveDate::parse_from_str(from_date, "%Y-%m-%d")
                    .map_err(|_| AppError::BadRequest("Invalid fromDate".to_string()))?;
                sql.push_str(&format!(" AND (payment_date >= ${idx}::date OR (payment_date IS NULL AND created_at::date >= ${idx}::date))"));
                binders.push(parsed.to_string());
                idx += 1;
            }
        }

        if let Some(ref to_date) = q.to_date {
            if !to_date.is_empty() {
                let parsed = chrono::NaiveDate::parse_from_str(to_date, "%Y-%m-%d")
                    .map_err(|_| AppError::BadRequest("Invalid toDate".to_string()))?;
                sql.push_str(&format!(" AND (payment_date <= ${idx}::date OR (payment_date IS NULL AND created_at::date <= ${idx}::date))"));
                binders.push(parsed.to_string());
                idx += 1;
            }
        }

        sql.push_str(" ORDER BY payment_date DESC NULLS LAST, created_at DESC");
        sql.push_str(&format!(" LIMIT ${idx} OFFSET ${}", idx + 1));

        let mut db_query = sqlx::query_as::<_, LibraryUnionRecord>(&sql);
        for val in binders {
            db_query = db_query.bind(val);
        }
        db_query = db_query.bind(limit).bind(offset);

        let list = db_query.fetch_all(&state.db).await?;
        let total = if list.is_empty() { 0 } else { list[0].total_count };

        return Ok(Json(json!({
            "success": true,
            "data": list,
            "pagination": {
                "total": total,
                "page": page,
                "limit": limit,
                "pages": (total as f64 / limit as f64).ceil() as i32
            }
        })));
    }



    // Default fee collections query (tuition, hostel, transport)
    let mut sql = r#"
        SELECT f.id, f.student_id, f.fee_type, f.room, f.bus_route, f.bus_no, f.receipt_book_no, f.receipt_no, f.receipt_date, f.payment_date,
               f.amount::float8 AS amount, f.utr_no, f.payment_mode, f.due_fees::float8 AS due_fees, f.remarks, f.discount::float8 AS discount, f.created_at,
               COALESCE(NULLIF(s.name, ''), f.student_id) AS student_name, s.class_name AS class_name, s.course_name AS course_name,
               COUNT(*) OVER()::int AS total_count
        FROM fee_collections f
        LEFT JOIN students s ON (TRIM(s.student_id) = TRIM(f.student_id) OR s.student_id ILIKE f.student_id)
        WHERE f.fee_type = $1
    "#
    .to_string();

    let mut binders = vec![fee_type];
    let mut idx = 2usize;

    if let Some(ref search) = q.search {
        let term = format!("%{}%", search);
        sql.push_str(&format!(" AND (s.name ILIKE ${idx} OR f.student_id ILIKE ${idx} OR f.receipt_no ILIKE ${idx})"));
        binders.push(term);
        idx += 1;
    }

    if let Some(ref class_name) = q.class_name {
        if class_name != "All Classes" {
            sql.push_str(&format!(" AND s.class_name = ${idx}"));
            binders.push(class_name.clone());
            idx += 1;
        }
    }

    if let Some(ref from_date) = q.from_date {
        if !from_date.is_empty() {
            let parsed = chrono::NaiveDate::parse_from_str(from_date, "%Y-%m-%d")
                .map_err(|_| AppError::BadRequest("Invalid fromDate".to_string()))?;
            sql.push_str(&format!(" AND f.payment_date >= ${idx}::date"));
            binders.push(parsed.to_string());
            idx += 1;
        }
    }

    if let Some(ref to_date) = q.to_date {
        if !to_date.is_empty() {
            let parsed = chrono::NaiveDate::parse_from_str(to_date, "%Y-%m-%d")
                .map_err(|_| AppError::BadRequest("Invalid toDate".to_string()))?;
            sql.push_str(&format!(" AND f.payment_date <= ${idx}::date"));
            binders.push(parsed.to_string());
            idx += 1;
        }
    }

    sql.push_str(" ORDER BY f.payment_date DESC, f.created_at DESC");
    sql.push_str(&format!(" LIMIT ${idx} OFFSET ${}", idx + 1));

    let mut db_query = sqlx::query_as::<_, FeeRecordWithDetails>(&sql);
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

pub async fn get_fee_record(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize_sub_role(&[UserSubRole::FinanceManager, UserSubRole::TransportManager, UserSubRole::HostelManager, UserSubRole::LibraryManager])?;

    // Check if ID is in book_issues (library fines unpaid/legacy)
    let lib_issue = sqlx::query_as::<_, LibraryUnionRecord>(
        r#"
        SELECT bi.id, m.student_id, m.name AS student_name, s.class_name AS class, b.title AS room,
               0::int AS overdue_days, 0.00::float8 AS amount, bi.fine_amount::float8 AS due_fees,
               '—' AS payment_mode, bi.remarks, NULL::date AS payment_date, NULL::date AS receipt_date,
               '—' AS receipt_no, '—' AS receipt_book_no, '—' AS utr_no, bi.created_at, 1::int AS total_count
        FROM book_issues bi
        JOIN library_members m ON m.id = bi.member_id
        JOIN books b ON b.id = bi.book_id
        LEFT JOIN students s ON s.student_id = m.student_id
        WHERE bi.id = $1
        "#
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?;

    if let Some(record) = lib_issue {
        return Ok(Json(json!({ "success": true, "data": record })));
    }

    // Default fee collections query
    let record = sqlx::query_as::<_, FeeRecordWithDetails>(
        r#"
        SELECT f.id, f.student_id, f.fee_type, f.room, f.bus_route, f.bus_no, f.receipt_book_no, f.receipt_no, f.receipt_date, f.payment_date,
               f.amount::float8 AS amount, f.utr_no, f.payment_mode, f.due_fees::float8 AS due_fees, f.remarks, f.discount::float8 AS discount, f.created_at,
               s.name AS student_name, s.class_name AS class, 1::int AS total_count
        FROM fee_collections f
        JOIN students s ON s.student_id = f.student_id
        WHERE f.id = $1
        "#
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?;

    let record = record.ok_or_else(|| AppError::NotFound("Fee record not found".to_string()))?;
    Ok(Json(json!({ "success": true, "data": record })))
}

pub async fn create_fee_record(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(fee_type): Path<String>,
    Json(payload): Json<AddFeeRecordPayload>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize_sub_role(&[UserSubRole::FinanceManager, UserSubRole::TransportManager, UserSubRole::HostelManager, UserSubRole::LibraryManager])?;
    payload.validate()?;

    if fee_type == "library" {
        // Library custom logic with tx
        let mut tx = state.db.begin().await?;

        // 1. Verify library member/student exists (auto-create baseline student if missing for bulk imports)
        let student_id = &payload.student_id;
        let default_class = "B.Sc. Nursing 1st Year";
        let _ = sqlx::query("INSERT INTO classes (name) VALUES ($1) ON CONFLICT (name) DO NOTHING")
            .bind(default_class)
            .execute(&mut *tx)
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
        .bind(student_id)
        .bind(format!("Student {}", student_id))
        .bind(default_class)
        .bind(default_dob)
        .execute(&mut *tx)
        .await;

        let parsed_receipt_date = chrono::NaiveDate::parse_from_str(&payload.receipt_date, "%Y-%m-%d")
            .map_err(|_| AppError::BadRequest("Invalid receipt date format".to_string()))?;
        let parsed_payment_date = chrono::NaiveDate::parse_from_str(&payload.payment_date, "%Y-%m-%d")
            .map_err(|_| AppError::BadRequest("Invalid payment date format".to_string()))?;

        let room = payload.room.as_deref().unwrap_or("Library Fine");
        let receipt_book_no = payload.receipt_book_no.as_deref().unwrap_or("—");
        let receipt_no = if payload.receipt_no.trim().is_empty() || payload.receipt_no == "—" {
            format!("LIB-{}", chrono::Utc::now().timestamp_micros())
        } else {
            payload.receipt_no.clone()
        };
        let amount = payload.amount;
        let utr_no = payload.utr_no.as_deref().unwrap_or("—");
        let payment_mode = payload.payment_mode.as_deref().unwrap_or("Online");
        let due_fees = payload.due_fees.unwrap_or(0.0);
        let remarks = payload.remarks.as_deref().unwrap_or("—");

        let existing_fee = sqlx::query("SELECT id FROM fee_collections WHERE receipt_no = $1")
            .bind(&receipt_no)
            .fetch_optional(&mut *tx)
            .await?;

        let new_fee_record = if let Some(row) = existing_fee {
            let existing_id: Uuid = row.get("id");
            sqlx::query_as::<_, FeeRecord>(
                r#"
                UPDATE fee_collections SET
                    student_id = $1,
                    fee_type = 'library',
                    room = $2,
                    receipt_book_no = $3,
                    receipt_date = $4,
                    payment_date = $5,
                    amount = $6,
                    utr_no = $7,
                    payment_mode = $8,
                    due_fees = $9,
                    remarks = $10,
                    discount = 0.00
                WHERE id = $11
                RETURNING id, student_id, fee_type, room, bus_route, bus_no, receipt_book_no, receipt_no, receipt_date, payment_date,
                          amount::float8 AS amount, utr_no, payment_mode, due_fees::float8 AS due_fees, remarks, discount::float8 AS discount, created_at
                "#
            )
            .bind(student_id)
            .bind(room)
            .bind(receipt_book_no)
            .bind(parsed_receipt_date)
            .bind(parsed_payment_date)
            .bind(amount)
            .bind(utr_no)
            .bind(payment_mode)
            .bind(due_fees)
            .bind(remarks)
            .bind(existing_id)
            .fetch_one(&mut *tx)
            .await?
        } else {
            sqlx::query_as::<_, FeeRecord>(
                r#"
                INSERT INTO fee_collections (
                    student_id, fee_type, room, bus_route, bus_no, 
                    receipt_book_no, receipt_no, receipt_date, payment_date, 
                    amount, utr_no, payment_mode, due_fees, remarks, discount
                ) VALUES ($1, 'library', $2, '—', '—', $3, $4, $5, $6, $7, $8, $9, $10, $11, 0.00) 
                RETURNING id, student_id, fee_type, room, bus_route, bus_no, receipt_book_no, receipt_no, receipt_date, payment_date,
                          amount::float8 AS amount, utr_no, payment_mode, due_fees::float8 AS due_fees, remarks, discount::float8 AS discount, created_at
                "#
            )
            .bind(student_id)
            .bind(room)
            .bind(receipt_book_no)
            .bind(&receipt_no)
            .bind(parsed_receipt_date)
            .bind(parsed_payment_date)
            .bind(amount)
            .bind(utr_no)
            .bind(payment_mode)
            .bind(due_fees)
            .bind(remarks)
            .fetch_one(&mut *tx)
            .await?
        };

        tx.commit().await?;

        crate::modules::email::service::trigger_fee_receipt_email(
            &state.config,
            &state.db,
            &new_fee_record.student_id,
            &new_fee_record.receipt_no,
            "library",
            new_fee_record.amount,
            new_fee_record.due_fees,
            &new_fee_record.payment_mode,
            &new_fee_record.payment_date.to_string()
        ).await;

        crate::modules::sms::service::trigger_fee_receipt_sms(
            &state.config,
            &state.db,
            &new_fee_record.student_id,
            &new_fee_record.receipt_no,
            "library",
            new_fee_record.amount,
            new_fee_record.due_fees
        ).await;

        crate::modules::whatsapp::service::trigger_fee_receipt_whatsapp(
            &state.config,
            &state.db,
            &new_fee_record.student_id,
            &new_fee_record.receipt_no,
            "library",
            new_fee_record.amount,
            new_fee_record.due_fees
        ).await;

        Ok((StatusCode::CREATED, Json(ApiResponse { success: true, data: new_fee_record })))
    } else if fee_type == "transport" {
        let record = transport::create_record(&state.db, payload).await?;
        crate::modules::email::service::trigger_fee_receipt_email(
            &state.config,
            &state.db,
            &record.student_id,
            &record.receipt_no,
            "transport",
            record.amount,
            record.due_fees,
            &record.payment_mode,
            &record.payment_date.to_string()
        ).await;

        crate::modules::sms::service::trigger_fee_receipt_sms(
            &state.config,
            &state.db,
            &record.student_id,
            &record.receipt_no,
            "transport",
            record.amount,
            record.due_fees
        ).await;

        crate::modules::whatsapp::service::trigger_fee_receipt_whatsapp(
            &state.config,
            &state.db,
            &record.student_id,
            &record.receipt_no,
            "transport",
            record.amount,
            record.due_fees
        ).await;

        Ok((StatusCode::CREATED, Json(ApiResponse { success: true, data: record })))
    } else if fee_type == "hostel" {
        let record = hostel::create_record(&state.db, payload).await?;
        crate::modules::email::service::trigger_fee_receipt_email(
            &state.config,
            &state.db,
            &record.student_id,
            &record.receipt_no,
            "hostel",
            record.amount,
            record.due_fees,
            &record.payment_mode,
            &record.payment_date.to_string()
        ).await;

        crate::modules::sms::service::trigger_fee_receipt_sms(
            &state.config,
            &state.db,
            &record.student_id,
            &record.receipt_no,
            "hostel",
            record.amount,
            record.due_fees
        ).await;

        crate::modules::whatsapp::service::trigger_fee_receipt_whatsapp(
            &state.config,
            &state.db,
            &record.student_id,
            &record.receipt_no,
            "hostel",
            record.amount,
            record.due_fees
        ).await;

        Ok((StatusCode::CREATED, Json(ApiResponse { success: true, data: record })))
    } else {
        let record = tuition::create_record(&state.db, &fee_type, payload).await?;
        crate::modules::email::service::trigger_fee_receipt_email(
            &state.config,
            &state.db,
            &record.student_id,
            &record.receipt_no,
            &fee_type,
            record.amount,
            record.due_fees,
            &record.payment_mode,
            &record.payment_date.to_string()
        ).await;

        crate::modules::sms::service::trigger_fee_receipt_sms(
            &state.config,
            &state.db,
            &record.student_id,
            &record.receipt_no,
            &fee_type,
            record.amount,
            record.due_fees
        ).await;

        crate::modules::whatsapp::service::trigger_fee_receipt_whatsapp(
            &state.config,
            &state.db,
            &record.student_id,
            &record.receipt_no,
            &fee_type,
            record.amount,
            record.due_fees
        ).await;
        Ok((StatusCode::CREATED, Json(ApiResponse { success: true, data: record })))
    }
}

pub async fn edit_fee_record(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateFeeRecordPayload>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize_sub_role(&[UserSubRole::FinanceManager, UserSubRole::TransportManager, UserSubRole::HostelManager, UserSubRole::LibraryManager])?;
    payload.validate()?;

    // Check if updating library fine issue
    let book_issue = sqlx::query("SELECT id FROM book_issues WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.db)
        .await?;

    if book_issue.is_some() {
        // Library fine payment scenario
        let payment_mode = payload.payment_mode.as_deref().unwrap_or("Cash");
        let fine_paid = payload.due_fees.unwrap_or(0.0) == 0.0 || payment_mode == "Waived";
        let remarks = if payment_mode == "Waived" { "Waived" } else { payload.remarks.as_deref().unwrap_or("Paid") };

        let mut tx = state.db.begin().await?;

        sqlx::query("UPDATE book_issues SET fine_paid = $2, remarks = $3, updated_at = now() WHERE id = $1")
            .bind(id)
            .bind(fine_paid)
            .bind(remarks)
            .execute(&mut *tx)
            .await?;

        // Find student ID
        let student_id = payload.student_id.as_deref().unwrap_or("");
        let row = sqlx::query("SELECT student_id FROM students WHERE student_id = $1")
            .bind(student_id)
            .fetch_optional(&mut *tx)
            .await?;

        let actual_student_id = match row {
            Some(r) => r.try_get::<String, _>("student_id").unwrap_or_else(|_| student_id.to_string()),
            None => student_id.to_string(),
        };

        let now_str = chrono::Utc::now().naive_utc().date();
        let parsed_receipt_date = payload.receipt_date.as_ref()
            .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
            .unwrap_or(now_str);
        let parsed_payment_date = payload.payment_date.as_ref()
            .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
            .unwrap_or(now_str);

        let room = payload.room.as_deref().unwrap_or("Library Fine");
        let receipt_book_no = payload.receipt_book_no.as_deref().unwrap_or("—");
        let receipt_no = payload.receipt_no.clone().unwrap_or_else(|| format!("LIB-{}", chrono::Utc::now().timestamp()));
        let amount = payload.amount.unwrap_or(0.0);
        let utr_no = payload.utr_no.as_deref().unwrap_or("—");
        let due_fees = payload.due_fees.unwrap_or(0.0);
        let remarks = payload.remarks.as_deref().unwrap_or("—");

        let new_fee_record = sqlx::query_as::<_, FeeRecord>(
            r#"
            INSERT INTO fee_collections (
                student_id, fee_type, room, bus_route, bus_no, 
                receipt_book_no, receipt_no, receipt_date, payment_date, 
                amount, utr_no, payment_mode, due_fees, remarks, discount
            ) VALUES ($1, 'library', $2, '—', '—', $3, $4, $5, $6, $7, $8, $9, $10, $11, 0.00) 
            RETURNING id, student_id, fee_type, room, bus_route, bus_no, receipt_book_no, receipt_no, receipt_date, payment_date,
                      amount::float8 AS amount, utr_no, payment_mode, due_fees::float8 AS due_fees, remarks, discount::float8 AS discount, created_at
            "#
        )
        .bind(actual_student_id)
        .bind(room)
        .bind(receipt_book_no)
        .bind(receipt_no)
        .bind(parsed_receipt_date)
        .bind(parsed_payment_date)
        .bind(amount)
        .bind(utr_no)
        .bind(payment_mode)
        .bind(due_fees)
        .bind(remarks)
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;

        let user_role_str = format!("{:?}", auth_user.role);
        let rec_no = &new_fee_record.receipt_no;
        let _ = log_audit(
            &state.db,
            "Staff User",
            &user_role_str,
            "FEE_COLLECTED",
            "finance",
            &format!("Receipt {} for student {} amount ₹{}", rec_no, new_fee_record.student_id, new_fee_record.amount)
        ).await;

        return Ok(Json(ApiResponse { success: true, data: new_fee_record }));
    }

    // Otherwise, fetch fee_type to delegate update
    let fee_type_row = sqlx::query("SELECT fee_type FROM fee_collections WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.db)
        .await?;

    let existing_fee_type = match fee_type_row {
        Some(r) => r.try_get::<String, _>("fee_type").unwrap_or_else(|_| "tuition".to_string()),
        None => return Err(AppError::NotFound("Fee record not found".to_string())),
    };

    let record = if existing_fee_type == "transport" {
        transport::update_record(&state.db, id, payload).await?
    } else if existing_fee_type == "hostel" {
        hostel::update_record(&state.db, id, payload).await?
    } else {
        tuition::update_record(&state.db, id, payload).await?
    };

    Ok(Json(ApiResponse { success: true, data: record }))
}

pub async fn remove_fee_record(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize_sub_role(&[UserSubRole::FinanceManager, UserSubRole::TransportManager, UserSubRole::HostelManager, UserSubRole::LibraryManager])?;

    // Check if library fine book issue
    let book_issue = sqlx::query("SELECT id FROM book_issues WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.db)
        .await?;

    if book_issue.is_some() {
        let updated = sqlx::query(
            "UPDATE book_issues SET fine_amount = 0, fine_paid = false, updated_at = now() WHERE id = $1 RETURNING id"
        )
        .bind(id)
        .fetch_one(&state.db)
        .await?;
        return Ok(Json(json!({ "success": true, "data": updated.get::<Uuid, _>("id") })));
    }

    // Fetch existing fee record to determine type
    let fee_type_row = sqlx::query("SELECT fee_type FROM fee_collections WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.db)
        .await?;

    let existing_fee_type = match fee_type_row {
        Some(r) => r.try_get::<String, _>("fee_type").unwrap_or_else(|_| "tuition".to_string()),
        None => return Err(AppError::NotFound("Fee record not found".to_string())),
    };

    let mut tx = state.db.begin().await?;

    let record = if existing_fee_type == "transport" {
        transport::delete_record(&state.db, id).await?
    } else if existing_fee_type == "hostel" {
        hostel::delete_record(&state.db, id).await?
    } else {
        tuition::delete_record(&state.db, id).await?
    };

    // Revert fine paid status on book_issues if this was a library fine payment
    if record.fee_type == "library" {
        let member = sqlx::query("SELECT id FROM library_members WHERE student_id = $1")
            .bind(&record.student_id)
            .fetch_optional(&mut *tx)
            .await?;

        if let Some(row) = member {
            let member_id = row.get::<Uuid, _>("id");
            sqlx::query(
                r#"
                UPDATE book_issues 
                SET fine_paid = false, updated_at = now() 
                WHERE member_id = $1 AND fine_paid = true AND book_id IN (
                    SELECT id FROM books WHERE title = $2
                )
                "#
            )
            .bind(member_id)
            .bind(&record.room)
            .execute(&mut *tx)
            .await?;
        }
    }

    tx.commit().await?;
    Ok(Json(json!({ "success": true, "data": record })))
}

// ─── GENERAL EXPENSES ─────────────────────────────────────

pub async fn get_expenses(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Query(q): Query<GetExpensesQuery>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize_sub_role(&[UserSubRole::FinanceManager, UserSubRole::TransportManager, UserSubRole::HostelManager, UserSubRole::LibraryManager])?;

    let page = q.page.unwrap_or(1).max(1);
    let limit = q.limit.unwrap_or(50).max(1);
    let offset = (page - 1) * limit;

    let mut sql = "SELECT id, ref_no, description, amount::float8 AS amount, category, date, payment_mode, remarks, utr, receipt, party_name, spent_by, voucher_no, created_at, COUNT(*) OVER()::int AS total_count FROM expenses WHERE 1=1".to_string();
    let mut binders = Vec::new();
    let mut idx = 1usize;

    if let Some(ref search) = q.search {
        let term = format!("%{}%", search);
        sql.push_str(&format!(" AND (description ILIKE ${idx} OR ref_no ILIKE ${idx} OR utr ILIKE ${idx} OR party_name ILIKE ${idx} OR voucher_no ILIKE ${idx})"));
        binders.push(term);
        idx += 1;
    }

    if let Some(ref category) = q.category {
        if category != "All Categories" {
            sql.push_str(&format!(" AND category = ${idx}"));
            binders.push(category.clone());
            idx += 1;
        }
    }

    if let Some(ref from_date) = q.from_date {
        if !from_date.is_empty() {
            let parsed = chrono::NaiveDate::parse_from_str(from_date, "%Y-%m-%d")
                .map_err(|_| AppError::BadRequest("Invalid fromDate".to_string()))?;
            sql.push_str(&format!(" AND date >= ${idx}::date"));
            binders.push(parsed.to_string());
            idx += 1;
        }
    }

    if let Some(ref to_date) = q.to_date {
        if !to_date.is_empty() {
            let parsed = chrono::NaiveDate::parse_from_str(to_date, "%Y-%m-%d")
                .map_err(|_| AppError::BadRequest("Invalid toDate".to_string()))?;
            sql.push_str(&format!(" AND date <= ${idx}::date"));
            binders.push(parsed.to_string());
            idx += 1;
        }
    }

    sql.push_str(" ORDER BY date DESC, created_at DESC");
    sql.push_str(&format!(" LIMIT ${idx} OFFSET ${}", idx + 1));

    let mut db_query = sqlx::query_as::<_, GeneralExpenseWithCount>(&sql);
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

pub async fn get_expense(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize_sub_role(&[UserSubRole::FinanceManager, UserSubRole::TransportManager, UserSubRole::HostelManager, UserSubRole::LibraryManager])?;

    let expense = sqlx::query_as::<_, GeneralExpense>("SELECT id, ref_no, description, amount::float8 AS amount, category, date, payment_mode, remarks, utr, receipt, party_name, spent_by, voucher_no, created_at FROM expenses WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.db)
        .await?;

    let expense = expense.ok_or_else(|| AppError::NotFound("Expense record not found".to_string()))?;
    Ok(Json(ApiResponse { success: true, data: expense }))
}

pub async fn create_expense(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(payload): Json<AddExpensePayload>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize_sub_role(&[UserSubRole::FinanceManager, UserSubRole::TransportManager, UserSubRole::HostelManager, UserSubRole::LibraryManager])?;
    payload.validate()?;

    let now_str = chrono::Utc::now().naive_utc().date();
    let parsed_date = payload.date.as_ref()
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .unwrap_or(now_str);

    let default_voucher = format!("EXP-{}", rand::random_range(1000..10000));
    let voucher_no = payload.voucher_no.as_deref().unwrap_or(&default_voucher);
    let ref_no = voucher_no.to_string();

    let payment_mode = payload.payment_mode.as_deref().unwrap_or("Online");
    let remarks = payload.remarks.as_deref().unwrap_or("—");
    let utr = payload.utr.as_deref().unwrap_or("—");
    let receipt = payload.receipt.as_deref().unwrap_or("—");
    let party_name = payload.party_name.as_deref().unwrap_or("—");
    let spent_by = payload.spent_by.as_deref().unwrap_or("—");

    let existing_expense = sqlx::query("SELECT id FROM expenses WHERE voucher_no = $1 OR ref_no = $1")
        .bind(voucher_no)
        .fetch_optional(&state.db)
        .await?;

    let record = if let Some(row) = existing_expense {
        let existing_id: Uuid = row.get("id");
        sqlx::query_as::<_, GeneralExpense>(
            r#"
            UPDATE expenses SET
                ref_no = $1,
                description = $2,
                amount = $3,
                category = $4,
                date = $5,
                payment_mode = $6,
                remarks = $7,
                utr = $8,
                receipt = $9,
                party_name = $10,
                spent_by = $11,
                voucher_no = $12
            WHERE id = $13
            RETURNING id, ref_no, description, amount::float8 AS amount, category, date, payment_mode, remarks, utr, receipt, party_name, spent_by, voucher_no, created_at
            "#
        )
        .bind(ref_no)
        .bind(&payload.description)
        .bind(payload.amount)
        .bind(&payload.category)
        .bind(parsed_date)
        .bind(payment_mode)
        .bind(remarks)
        .bind(utr)
        .bind(receipt)
        .bind(party_name)
        .bind(spent_by)
        .bind(voucher_no)
        .bind(existing_id)
        .fetch_one(&state.db)
        .await?
    } else {
        sqlx::query_as::<_, GeneralExpense>(
            r#"
            INSERT INTO expenses (ref_no, description, amount, category, date, payment_mode, remarks, utr, receipt, party_name, spent_by, voucher_no)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING id, ref_no, description, amount::float8 AS amount, category, date, payment_mode, remarks, utr, receipt, party_name, spent_by, voucher_no, created_at
            "#
        )
        .bind(ref_no)
        .bind(&payload.description)
        .bind(payload.amount)
        .bind(&payload.category)
        .bind(parsed_date)
        .bind(payment_mode)
        .bind(remarks)
        .bind(utr)
        .bind(receipt)
        .bind(party_name)
        .bind(spent_by)
        .bind(voucher_no)
        .fetch_one(&state.db)
        .await?
    };

    let user_role_str = format!("{:?}", auth_user.role);
    let v_no = &record.voucher_no;
    let _ = log_audit(
        &state.db,
        "Staff User",
        &user_role_str,
        "EXPENSE_RECORDED",
        "finance",
        &format!("Voucher {} - {} amount ₹{}", v_no, record.description, record.amount)
    ).await;

    Ok((StatusCode::CREATED, Json(ApiResponse { success: true, data: record })))
}

pub async fn edit_expense(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateExpensePayload>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize_sub_role(&[UserSubRole::FinanceManager, UserSubRole::TransportManager, UserSubRole::HostelManager, UserSubRole::LibraryManager])?;
    payload.validate()?;

    // Fetch existing
    let existing = sqlx::query_as::<_, GeneralExpense>("SELECT id, ref_no, description, amount::float8 AS amount, category, date, payment_mode, remarks, utr, receipt, party_name, spent_by, voucher_no, created_at FROM expenses WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.db)
        .await?;

    let mut existing = existing.ok_or_else(|| AppError::NotFound("Expense record not found".to_string()))?;

    if let Some(description) = payload.description { existing.description = description; }
    if let Some(amount) = payload.amount { existing.amount = amount; }
    if let Some(category) = payload.category { existing.category = category; }
    if let Some(payment_mode) = payload.payment_mode { existing.payment_mode = payment_mode; }
    if let Some(remarks) = payload.remarks { existing.remarks = Some(remarks); }
    if let Some(utr) = payload.utr { existing.utr = utr; }
    if let Some(receipt) = payload.receipt { existing.receipt = receipt; }
    if let Some(party_name) = payload.party_name { existing.party_name = party_name; }
    if let Some(spent_by) = payload.spent_by { existing.spent_by = spent_by; }
    if let Some(voucher_no) = payload.voucher_no { existing.voucher_no = voucher_no; }

    if let Some(ref date_str) = payload.date {
        existing.date = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d").unwrap();
    }

    let record = sqlx::query_as::<_, GeneralExpense>(
        r#"
        UPDATE expenses
        SET ref_no = $1, description = $2, amount = $3, category = $4, date = $5, payment_mode = $6, remarks = $7, utr = $8, receipt = $9, party_name = $10, spent_by = $11, voucher_no = $12
        WHERE id = $13
        RETURNING id, ref_no, description, amount::float8 AS amount, category, date, payment_mode, remarks, utr, receipt, party_name, spent_by, voucher_no, created_at
        "#
    )
    .bind(&existing.ref_no)
    .bind(&existing.description)
    .bind(existing.amount)
    .bind(&existing.category)
    .bind(existing.date)
    .bind(&existing.payment_mode)
    .bind(existing.remarks)
    .bind(&existing.utr)
    .bind(&existing.receipt)
    .bind(&existing.party_name)
    .bind(&existing.spent_by)
    .bind(&existing.voucher_no)
    .bind(id)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(ApiResponse { success: true, data: record }))
}

pub async fn remove_expense(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize_sub_role(&[UserSubRole::FinanceManager, UserSubRole::TransportManager, UserSubRole::HostelManager, UserSubRole::LibraryManager])?;

    let deleted = sqlx::query_as::<_, GeneralExpense>("DELETE FROM expenses WHERE id = $1 RETURNING id, ref_no, description, amount::float8 AS amount, category, date, payment_mode, remarks, utr, receipt, party_name, spent_by, voucher_no, created_at")
        .bind(id)
        .fetch_optional(&state.db)
        .await?;

    let expense = deleted.ok_or_else(|| AppError::NotFound("Expense record not found".to_string()))?;
    Ok(Json(ApiResponse { success: true, data: expense }))
}
