use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::{Duration, Utc};
use serde_json::json;
use sqlx::Row;
use uuid::Uuid;
use crate::AppState;
use crate::errors::AppError;
use crate::middleware::AuthUser;
use crate::modules::auth::models::{ApiResponse, ApiMessageResponse};
use crate::utils::activity::log_activity;
use super::models::*;

// Helper to update overdue status of all active issues
async fn run_update_overdue_status(db: &sqlx::PgPool) -> Result<(), AppError> {
    sqlx::query(
        r#"
        UPDATE book_issues bi
        SET status = 'overdue',
            fine_amount = (CURRENT_DATE - bi.due_date) * s.fine_per_day
        FROM library_settings s
        WHERE bi.status IN ('issued', 'overdue') AND bi.due_date < CURRENT_DATE
        "#
    )
    .execute(db)
    .await?;
    Ok(())
}

// ─── BOOKS ───────────────────────────────────────────────

pub async fn get_books(
    State(state): State<AppState>,
    Query(query_params): Query<GetBooksQuery>,
) -> Result<impl IntoResponse, AppError> {
    let page = query_params.page.unwrap_or(1);
    let limit = query_params.limit.unwrap_or(50);
    let offset = (page - 1) * limit;

    let mut sql = r#"
        SELECT b.id, b.acc_no, b.title, b.author, b.subject, b.price::float8 AS price, b.quantity, b.added_date, b.sl_no, b.type, b.volume, b.number_val, b.month, b.year, b.publisher, b.created_at, b.updated_at,
            (b.quantity - COALESCE(active.count, 0))::int AS available_quantity,
            CASE
                WHEN b.quantity = 0 THEN 'unavailable'
                WHEN (b.quantity - COALESCE(active.count, 0)) <= 0 THEN 'issued'
                ELSE 'available'
            END AS status,
            COUNT(*) OVER()::int AS total_count
        FROM books b
        LEFT JOIN (
            SELECT book_id, COUNT(*) AS count
            FROM book_issues
            WHERE status = 'issued' OR status = 'overdue'
            GROUP BY book_id
        ) active ON active.book_id = b.id
        WHERE 1=1
    "#
    .to_string();

    let mut count_idx = 1;
    let mut binders = Vec::new();

    if let Some(ref r#type) = query_params.r#type {
        sql.push_str(&format!(" AND b.type = ${}", count_idx));
        binders.push(r#type.clone());
        count_idx += 1;
    }

    if let Some(ref search) = query_params.search {
        let search_term = format!("%{}%", search);
        sql.push_str(&format!(
            " AND (b.title ILIKE ${0} OR b.author ILIKE ${0} OR b.acc_no ILIKE ${0} OR b.sl_no ${0})",
            count_idx
        ));
        binders.push(search_term);
        count_idx += 1;
    }

    sql.push_str(" ORDER BY b.created_at DESC");
    sql.push_str(&format!(" LIMIT ${} OFFSET ${}", count_idx, count_idx + 1));

    let mut db_query = sqlx::query_as::<_, BookWithStatus>(&sql);
    for bind_val in binders {
        db_query = db_query.bind(bind_val);
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

pub async fn get_book(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let book = sqlx::query_as::<_, Book>(
        r#"
        SELECT id, acc_no, title, author, subject, price::float8 AS price, quantity, added_date, sl_no, type, volume, number_val, month, year, publisher, created_at, updated_at
        FROM books WHERE id = $1
        "#
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?;

    let book = book.ok_or_else(|| AppError::NotFound("Resource not found".to_string()))?;

    Ok(Json(ApiResponse {
        success: true,
        data: book,
    }))
}

pub async fn create_book(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(payload): Json<AddBookPayload>,
) -> Result<impl IntoResponse, AppError> {
    payload.validate()?;

    if let Some(ref acc_no) = payload.acc_no {
        let existing = sqlx::query("SELECT id FROM books WHERE acc_no = $1")
            .bind(acc_no)
            .fetch_optional(&state.db)
            .await?;
        if existing.is_some() {
            return Err(AppError::Conflict("Accession number already exists".to_string()));
        }
    }

    let added_date = match &payload.added_date {
        Some(d) if !d.is_empty() => chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").unwrap(),
        _ => Utc::now().date_naive(),
    };

    let price = payload.price.unwrap_or(0.0);
    let quantity = payload.quantity.unwrap_or(1);
    let r#type = payload.r#type.unwrap_or_else(|| "book".to_string());

    let book = sqlx::query_as::<_, Book>(
        r#"
        INSERT INTO books (
            acc_no, title, author, subject, price, quantity, added_date,
            sl_no, type, volume, number_val, month, year, publisher
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
        RETURNING id, acc_no, title, author, subject, price::float8 AS price, quantity, added_date, sl_no, type, volume, number_val, month, year, publisher, created_at, updated_at
        "#
    )
    .bind(&payload.acc_no)
    .bind(&payload.title)
    .bind(&payload.author)
    .bind(&payload.subject)
    .bind(price)
    .bind(quantity)
    .bind(added_date)
    .bind(&payload.sl_no)
    .bind(&r#type)
    .bind(&payload.volume)
    .bind(&payload.number_val)
    .bind(&payload.month)
    .bind(&payload.year)
    .bind(&payload.publisher)
    .fetch_one(&state.db)
    .await?;

    log_activity(
        &state.db,
        Some(auth_user.id),
        "BOOK_ADDED",
        "book",
        Some(book.id),
        Some(json!({ "title": book.title })),
    )
    .await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse {
            success: true,
            data: book,
        }),
    ))
}

pub async fn edit_book(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateBookPayload>,
) -> Result<impl IntoResponse, AppError> {
    payload.validate()?;

    let existing = sqlx::query_as::<_, Book>(
        r#"
        SELECT id, acc_no, title, author, subject, price::float8 AS price, quantity, added_date, sl_no, type, volume, number_val, month, year, publisher, created_at, updated_at
        FROM books WHERE id = $1
        "#
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?;

    let mut existing = existing.ok_or_else(|| AppError::NotFound("Book not found".to_string()))?;

    if let Some(ref acc_no) = payload.acc_no {
        let duplicate = sqlx::query("SELECT id FROM books WHERE acc_no = $1 AND id != $2")
            .bind(acc_no)
            .bind(id)
            .fetch_optional(&state.db)
            .await?;
        if duplicate.is_some() {
            return Err(AppError::Conflict("Accession number already exists".to_string()));
        }
        existing.acc_no = Some(acc_no.clone());
    }

    if let Some(title) = payload.title { existing.title = title; }
    if let Some(author) = payload.author { existing.author = author; }
    if let Some(subject) = payload.subject { existing.subject = Some(subject); }
    if let Some(price) = payload.price { existing.price = price; }
    if let Some(quantity) = payload.quantity { existing.quantity = quantity; }
    if let Some(sl_no) = payload.sl_no { existing.sl_no = Some(sl_no); }
    if let Some(r#type) = payload.r#type { existing.r#type = r#type; }
    if let Some(volume) = payload.volume { existing.volume = Some(volume); }
    if let Some(number_val) = payload.number_val { existing.number_val = Some(number_val); }
    if let Some(month) = payload.month { existing.month = Some(month); }
    if let Some(year) = payload.year { existing.year = Some(year); }
    if let Some(publisher) = payload.publisher { existing.publisher = Some(publisher); }

    if let Some(ref added_date_str) = payload.added_date {
        if !added_date_str.is_empty() {
            existing.added_date = chrono::NaiveDate::parse_from_str(added_date_str, "%Y-%m-%d").unwrap();
        }
    }

    let updated_book = sqlx::query_as::<_, Book>(
        r#"
        UPDATE books
        SET acc_no = $1, title = $2, author = $3, subject = $4, price = $5, quantity = $6, added_date = $7,
            sl_no = $8, type = $9, volume = $10, number_val = $11, month = $12, year = $13, publisher = $14, updated_at = now()
        WHERE id = $15
        RETURNING id, acc_no, title, author, subject, price::float8 AS price, quantity, added_date, sl_no, type, volume, number_val, month, year, publisher, created_at, updated_at
        "#
    )
    .bind(existing.acc_no)
    .bind(existing.title)
    .bind(existing.author)
    .bind(existing.subject)
    .bind(existing.price)
    .bind(existing.quantity)
    .bind(existing.added_date)
    .bind(existing.sl_no)
    .bind(existing.r#type)
    .bind(existing.volume)
    .bind(existing.number_val)
    .bind(existing.month)
    .bind(existing.year)
    .bind(existing.publisher)
    .bind(id)
    .fetch_one(&state.db)
    .await?;

    log_activity(
        &state.db,
        Some(auth_user.id),
        "BOOK_UPDATED",
        "book",
        Some(updated_book.id),
        Some(json!({ "title": updated_book.title })),
    )
    .await?;

    Ok(Json(ApiResponse {
        success: true,
        data: updated_book,
    }))
}

pub async fn remove_book(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    // Check if book has active issues
    let active_issues = sqlx::query("SELECT id FROM book_issues WHERE book_id = $1 AND status IN ('issued', 'overdue')")
        .bind(id)
        .fetch_optional(&state.db)
        .await?;

    if active_issues.is_some() {
        return Err(AppError::BadRequest("Cannot delete book with active issues".to_string()));
    }

    let deleted_book = sqlx::query_as::<_, Book>(
        r#"
        DELETE FROM books WHERE id = $1
        RETURNING id, acc_no, title, author, subject, price::float8 AS price, quantity, added_date, sl_no, type, volume, number_val, month, year, publisher, created_at, updated_at
        "#
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?;

    let deleted_book = deleted_book.ok_or_else(|| AppError::NotFound("Book not found".to_string()))?;

    log_activity(
        &state.db,
        Some(auth_user.id),
        "BOOK_DELETED",
        "book",
        Some(deleted_book.id),
        Some(json!({ "title": deleted_book.title })),
    )
    .await?;

    Ok(Json(ApiMessageResponse {
        success: true,
        message: "Book deleted".to_string(),
    }))
}

// ─── MEMBERS ─────────────────────────────────────────────

pub async fn get_members(
    State(state): State<AppState>,
    Query(query_params): Query<GetMembersQuery>,
) -> Result<impl IntoResponse, AppError> {
    let page = query_params.page.unwrap_or(1);
    let limit = query_params.limit.unwrap_or(50);
    let offset = (page - 1) * limit;

    let mut sql = r#"
        SELECT m.id, m.student_id, m.user_id, m.name, m.class, m.phone, m.status, m.created_at, m.updated_at,
            COUNT(CASE WHEN bi.status IN ('issued','overdue') THEN 1 END)::int8 AS currently_issued,
            COUNT(bi.id)::int8 AS total_issued,
            COUNT(*) OVER()::int AS total_count
        FROM library_members m
        LEFT JOIN book_issues bi ON bi.member_id = m.id
    "#
    .to_string();

    let mut count_idx = 1;
    let mut binders = Vec::new();

    if let Some(ref search) = query_params.search {
        let search_term = format!("%{}%", search);
        sql.push_str(&format!(
            " WHERE m.name ILIKE ${0} OR m.student_id ILIKE ${0} OR m.phone ILIKE ${0}",
            count_idx
        ));
        binders.push(search_term);
        count_idx += 1;
    }

    sql.push_str(" GROUP BY m.id ORDER BY m.created_at DESC");
    sql.push_str(&format!(" LIMIT ${} OFFSET ${}", count_idx, count_idx + 1));

    let mut db_query = sqlx::query_as::<_, LibraryMemberWithStats>(&sql);
    for bind_val in binders {
        db_query = db_query.bind(bind_val);
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

pub async fn get_member(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let member = sqlx::query_as::<_, LibraryMemberWithStats>(
        r#"
        SELECT m.id, m.student_id, m.user_id, m.name, m.class, m.phone, m.status, m.created_at, m.updated_at,
            COUNT(CASE WHEN bi.status IN ('issued','overdue') THEN 1 END)::int8 AS currently_issued,
            COUNT(bi.id)::int8 AS total_issued,
            1::int AS total_count
        FROM library_members m
        LEFT JOIN book_issues bi ON bi.member_id = m.id
        WHERE m.id = $1
        GROUP BY m.id
        "#
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?;

    let member = member.ok_or_else(|| AppError::NotFound("Member not found".to_string()))?;

    Ok(Json(ApiResponse {
        success: true,
        data: member,
    }))
}

pub async fn create_member(
    State(state): State<AppState>,
    _auth_user: AuthUser,
    Json(payload): Json<AddMemberPayload>,
) -> Result<impl IntoResponse, AppError> {
    payload.validate()?;

    let existing = sqlx::query("SELECT id FROM library_members WHERE student_id = $1")
        .bind(&payload.student_id)
        .fetch_optional(&state.db)
        .await?;

    if existing.is_some() {
        return Err(AppError::Conflict("Student ID already registered".to_string()));
    }

    let member = sqlx::query_as::<_, LibraryMember>(
        r#"
        INSERT INTO library_members (student_id, name, class, phone, user_id)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING *
        "#
    )
    .bind(&payload.student_id)
    .bind(&payload.name)
    .bind(&payload.class)
    .bind(&payload.phone)
    .bind(payload.user_id)
    .fetch_one(&state.db)
    .await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse {
            success: true,
            data: member,
        }),
    ))
}

pub async fn edit_member(
    State(state): State<AppState>,
    _auth_user: AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateMemberPayload>,
) -> Result<impl IntoResponse, AppError> {
    payload.validate()?;

    let existing = sqlx::query_as::<_, LibraryMember>(
        "SELECT * FROM library_members WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?;

    let mut existing = existing.ok_or_else(|| AppError::NotFound("Member not found".to_string()))?;

    if let Some(name) = payload.name { existing.name = name; }
    if let Some(class) = payload.class { existing.class = Some(class); }
    if let Some(phone) = payload.phone { existing.phone = Some(phone); }
    if let Some(status) = payload.status { existing.status = status; }

    let updated_member = sqlx::query_as::<_, LibraryMember>(
        r#"
        UPDATE library_members
        SET name = $1, class = $2, phone = $3, status = $4, updated_at = now()
        WHERE id = $5
        RETURNING *
        "#
    )
    .bind(existing.name)
    .bind(existing.class)
    .bind(existing.phone)
    .bind(existing.status)
    .bind(id)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(ApiResponse {
        success: true,
        data: updated_member,
    }))
}

// ─── ISSUES ──────────────────────────────────────────────

pub async fn get_issues(
    State(state): State<AppState>,
    Query(query_params): Query<GetIssuesQuery>,
) -> Result<impl IntoResponse, AppError> {
    run_update_overdue_status(&state.db).await?;

    let page = query_params.page.unwrap_or(1);
    let limit = query_params.limit.unwrap_or(50);
    let offset = (page - 1) * limit;

    let mut sql = r#"
        SELECT bi.id, bi.issue_no, bi.member_id, bi.book_id, bi.issued_by, bi.issue_date, bi.due_date, bi.return_date, bi.fine_amount::float8 AS fine_amount, bi.fine_paid, bi.status, bi.remarks, bi.created_at, bi.updated_at,
            m.name AS member_name, m.student_id, m.class,
            b.title AS book_title, b.acc_no, b.type AS book_type, b.sl_no AS book_sl_no,
            COUNT(*) OVER()::int AS total_count
        FROM book_issues bi
        JOIN library_members m ON m.id = bi.member_id
        JOIN books b ON b.id = bi.book_id
    "#
    .to_string();

    let mut conditions = Vec::new();
    let mut count_idx = 1;
    let mut binders = Vec::new();

    if let Some(ref status) = query_params.status {
        if status != "all" {
            conditions.push(format!("bi.status = ${}", count_idx));
            binders.push(status.clone());
            count_idx += 1;
        }
    }

    if let Some(ref start_date) = query_params.start_date {
        let parsed_start = chrono::NaiveDate::parse_from_str(start_date, "%Y-%m-%d")
            .map_err(|_| AppError::BadRequest("Invalid startDate format".to_string()))?;
        conditions.push(format!("bi.issue_date >= ${}", count_idx));
        binders.push(parsed_start.to_string());
        count_idx += 1;
    }

    if let Some(ref end_date) = query_params.end_date {
        let parsed_end = chrono::NaiveDate::parse_from_str(end_date, "%Y-%m-%d")
            .map_err(|_| AppError::BadRequest("Invalid endDate format".to_string()))?;
        conditions.push(format!("bi.issue_date <= ${}", count_idx));
        binders.push(parsed_end.to_string());
        count_idx += 1;
    }

    if !conditions.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(&conditions.join(" AND "));
    }

    sql.push_str(" ORDER BY bi.created_at DESC");
    sql.push_str(&format!(" LIMIT ${} OFFSET ${}", count_idx, count_idx + 1));

    let mut db_query = sqlx::query_as::<_, BookIssueWithDetails>(&sql);
    for bind_val in binders {
        db_query = db_query.bind(bind_val);
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

pub async fn issue_book(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(payload): Json<IssueBookPayload>,
) -> Result<impl IntoResponse, AppError> {
    payload.validate()?;

    // Read settings
    let settings = sqlx::query_as::<_, LibrarySettings>(
        "SELECT id, issue_duration_days, fine_per_day::float8 AS fine_per_day, max_books_per_member, created_at, updated_at FROM library_settings LIMIT 1"
    )
    .fetch_one(&state.db)
    .await?;

    let is_uuid = Uuid::parse_str(&payload.member_id).is_ok();

    // Check if library member exists
    let mut member = if is_uuid {
        let uid = Uuid::parse_str(&payload.member_id).unwrap();
        sqlx::query_as::<_, LibraryMember>("SELECT * FROM library_members WHERE id = $1")
            .bind(uid)
            .fetch_optional(&state.db)
            .await?
    } else {
        sqlx::query_as::<_, LibraryMember>("SELECT * FROM library_members WHERE student_id = $1")
            .bind(&payload.member_id)
            .fetch_optional(&state.db)
            .await?
    };

    // Auto-register student if missing but present in student directory
    if member.is_none() {
        let student = if is_uuid {
            let uid = Uuid::parse_str(&payload.member_id).unwrap();
            sqlx::query("SELECT student_id, name, class_name, phone FROM students WHERE id = $1")
                .bind(uid)
                .fetch_optional(&state.db)
                .await?
        } else {
            sqlx::query("SELECT student_id, name, class_name, phone FROM students WHERE student_id = $1")
                .bind(&payload.member_id)
                .fetch_optional(&state.db)
                .await?
        };

        let student_row = student.ok_or_else(|| {
            AppError::NotFound("Student or library member not found in registry".to_string())
        })?;

        let student_id: String = student_row.get("student_id");
        let name: String = student_row.get("name");
        let class_name: Option<String> = student_row.get("class_name");
        let phone: Option<String> = student_row.get("phone");

        let new_member = sqlx::query_as::<_, LibraryMember>(
            r#"
            INSERT INTO library_members (student_id, name, class, phone, status)
            VALUES ($1, $2, $3, $4, 'active')
            RETURNING *
            "#
        )
        .bind(student_id)
        .bind(name)
        .bind(class_name)
        .bind(phone)
        .fetch_one(&state.db)
        .await?;

        member = Some(new_member);
    }

    let member = member.unwrap();

    if member.status != "active" {
        return Err(AppError::BadRequest("Library member account is inactive".to_string()));
    }

    // Check maximum issued books constraint
    let active_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM book_issues WHERE member_id = $1 AND status IN ('issued', 'overdue')"
    )
    .bind(member.id)
    .fetch_one(&state.db)
    .await?;

    if active_count >= settings.max_books_per_member as i64 {
        return Err(AppError::BadRequest(format!(
            "Member already has {} books issued",
            settings.max_books_per_member
        )));
    }

    // Verify book availability
    let book = sqlx::query_as::<_, Book>(
        r#"
        SELECT id, acc_no, title, author, subject, price::float8 AS price, quantity, added_date, sl_no, type, volume, number_val, month, year, publisher, created_at, updated_at
        FROM books WHERE id = $1
        "#
    )
    .bind(payload.book_id)
    .fetch_optional(&state.db)
    .await?;

    let book = book.ok_or_else(|| AppError::NotFound("Book not found".to_string()))?;

    let issued_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM book_issues WHERE book_id = $1 AND status IN ('issued', 'overdue')"
    )
    .bind(payload.book_id)
    .fetch_one(&state.db)
    .await?;

    if issued_count >= book.quantity as i64 {
        return Err(AppError::BadRequest("No copies available".to_string()));
    }

    // Generate transaction issue number
    let total_issues_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM book_issues").fetch_one(&state.db).await?;
    let issue_no = format!("ISS-{:03}", total_issues_count + 1);

    let issue_date = match &payload.issue_date {
        Some(d) if !d.is_empty() => chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").unwrap(),
        _ => Utc::now().date_naive(),
    };

    let due_date = match &payload.due_date {
        Some(d) if !d.is_empty() => chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").unwrap(),
        _ => issue_date + Duration::days(settings.issue_duration_days as i64),
    };

    let issue = sqlx::query_as::<_, BookIssue>(
        r#"
        INSERT INTO book_issues (issue_no, member_id, book_id, issued_by, issue_date, due_date)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, issue_no, member_id, book_id, issued_by, issue_date, due_date, return_date, fine_amount::float8 AS fine_amount, fine_paid, status, remarks, created_at, updated_at
        "#
    )
    .bind(&issue_no)
    .bind(member.id)
    .bind(payload.book_id)
    .bind(Some(auth_user.id))
    .bind(issue_date)
    .bind(due_date)
    .fetch_one(&state.db)
    .await?;

    log_activity(
        &state.db,
        Some(auth_user.id),
        "BOOK_ISSUED",
        "book_issue",
        Some(issue.id),
        Some(json!({ "book": book.title, "member": member.name })),
    )
    .await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse {
            success: true,
            data: issue,
        }),
    ))
}

pub async fn return_book(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(payload): Json<ReturnBookPayload>,
) -> Result<impl IntoResponse, AppError> {
    let issue = sqlx::query(
        r#"
        SELECT bi.id, bi.due_date, bi.status, b.title AS book_title, m.name AS member_name
        FROM book_issues bi
        JOIN books b ON b.id = bi.book_id
        JOIN library_members m ON m.id = bi.member_id
        WHERE bi.id = $1
        "#
    )
    .bind(payload.issue_id)
    .fetch_optional(&state.db)
    .await?;

    let issue_row = issue.ok_or_else(|| AppError::NotFound("Issue record not found".to_string()))?;
    let status: String = issue_row.get("status");

    if status == "returned" {
        return Err(AppError::BadRequest("Book already returned".to_string()));
    }

    // Calculate fine
    let settings = sqlx::query_as::<_, LibrarySettings>(
        "SELECT id, issue_duration_days, fine_per_day::float8 AS fine_per_day, max_books_per_member, created_at, updated_at FROM library_settings LIMIT 1"
    )
    .fetch_one(&state.db)
    .await?;

    let due_date: chrono::NaiveDate = issue_row.get("due_date");
    let today = Utc::now().date_naive();
    let mut fine_amount = 0.0;

    if today > due_date {
        let days_overdue = today.signed_duration_since(due_date).num_days();
        fine_amount = (days_overdue as f64) * settings.fine_per_day;
    }

    let remarks = payload.remarks.unwrap_or_else(|| "—".to_string());

    let updated = sqlx::query_as::<_, BookIssue>(
        r#"
        UPDATE book_issues
        SET status = 'returned', return_date = CURRENT_DATE,
            fine_amount = $2, fine_paid = $3, remarks = $4, updated_at = now()
        WHERE id = $1
        RETURNING id, issue_no, member_id, book_id, issued_by, issue_date, due_date, return_date, fine_amount::float8 AS fine_amount, fine_paid, status, remarks, created_at, updated_at
        "#
    )
    .bind(payload.issue_id)
    .bind(fine_amount)
    .bind(payload.fine_paid)
    .bind(&remarks)
    .fetch_one(&state.db)
    .await?;

    let book_title: String = issue_row.get("book_title");
    let member_name: String = issue_row.get("member_name");

    log_activity(
        &state.db,
        Some(auth_user.id),
        "BOOK_RETURNED",
        "book_issue",
        Some(payload.issue_id),
        Some(json!({ "book": book_title, "member": member_name, "fine": fine_amount })),
    )
    .await?;

    Ok(Json(ApiResponse {
        success: true,
        data: updated,
    }))
}

pub async fn update_fine(
    State(state): State<AppState>,
    _auth_user: AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateFinePayload>,
) -> Result<impl IntoResponse, AppError> {
    let remarks = payload.remarks.unwrap_or_else(|| "—".to_string());

    let mut tx = state.db.begin().await?;

    let updated_issue = sqlx::query_as::<_, BookIssue>(
        r#"
        UPDATE book_issues
        SET fine_paid = $2, remarks = $3, updated_at = now()
        WHERE id = $1
        RETURNING id, issue_no, member_id, book_id, issued_by, issue_date, due_date, return_date, fine_amount::float8 AS fine_amount, fine_paid, status, remarks, created_at, updated_at
        "#
    )
    .bind(id)
    .bind(payload.fine_paid)
    .bind(&remarks)
    .fetch_optional(&mut *tx)
    .await?;

    let updated_issue = match updated_issue {
        Some(issue) => issue,
        None => return Err(AppError::NotFound("Issue record not found".to_string())),
    };

    // If fine is now marked paid and amount > 0, insert a transaction log in fee_collections
    if payload.fine_paid && updated_issue.fine_amount > 0.0 {
        let member = sqlx::query(
            r#"
            SELECT m.student_id, b.title AS book_title 
            FROM library_members m 
            JOIN book_issues bi ON bi.member_id = m.id
            JOIN books b ON b.id = bi.book_id
            WHERE bi.id = $1
            "#
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?;

        if let Some(member_row) = member {
            let student_id: String = member_row.get("student_id");
            let book_title: String = member_row.get("book_title");

            let is_waived = remarks.to_lowercase().contains("waived");
            let amt_paid = if is_waived { 0.0 } else { updated_issue.fine_amount };
            let due_fees = 0.0;
            let pay_mode = if is_waived { "Waived" } else { "Cash" };
            let remark_text = if is_waived { "Waived by Librarian".to_string() } else { format!("Paid to Library - {}", remarks) };
            let receipt_no = format!("LIB-COLL-{}", Utc::now().timestamp_millis());

            sqlx::query(
                r#"
                INSERT INTO fee_collections (
                    student_id, fee_type, room, bus_route, bus_no, 
                    receipt_book_no, receipt_no, receipt_date, payment_date, 
                    amount, utr_no, payment_mode, due_fees, remarks, discount
                ) VALUES ($1, 'library', $2, '—', '—', '—', $3, CURRENT_DATE, CURRENT_DATE, $4, '—', $5, $6, $7, 0.00)
                "#
            )
            .bind(&student_id)
            .bind(&book_title)
            .bind(&receipt_no)
            .bind(amt_paid)
            .bind(pay_mode)
            .bind(due_fees)
            .bind(&remark_text)
            .execute(&mut *tx)
            .await?;
        }
    }

    tx.commit().await?;

    Ok(Json(ApiResponse {
        success: true,
        data: updated_issue,
    }))
}

// ─── SETTINGS ────────────────────────────────────────────

pub async fn get_settings(
    State(state): State<AppState>,
    _auth_user: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    let settings = sqlx::query_as::<_, LibrarySettings>(
        "SELECT id, issue_duration_days, fine_per_day::float8 AS fine_per_day, max_books_per_member, created_at, updated_at FROM library_settings LIMIT 1"
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(ApiResponse {
        success: true,
        data: settings,
    }))
}

pub async fn edit_settings(
    State(state): State<AppState>,
    _auth_user: AuthUser,
    Json(payload): Json<UpdateSettingsPayload>,
) -> Result<impl IntoResponse, AppError> {
    payload.validate()?;

    let updated = sqlx::query_as::<_, LibrarySettings>(
        r#"
        UPDATE library_settings
        SET issue_duration_days = $1, fine_per_day = $2, max_books_per_member = $3, updated_at = now()
        WHERE id = (SELECT id FROM library_settings LIMIT 1)
        RETURNING id, issue_duration_days, fine_per_day::float8 AS fine_per_day, max_books_per_member, created_at, updated_at
        "#
    )
    .bind(payload.issue_duration_days)
    .bind(payload.fine_per_day)
    .bind(payload.max_books_per_member)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(ApiResponse {
        success: true,
        data: updated,
    }))
}

// ─── STATS & ACTIVITY ────────────────────────────────────

pub async fn get_stats(
    State(state): State<AppState>,
    Query(query_params): Query<GetStatsQuery>,
) -> Result<impl IntoResponse, AppError> {
    run_update_overdue_status(&state.db).await?;

    let mut books_filter = " WHERE 1=1".to_string();
    let mut members_filter = "".to_string();
    let mut issues_filter = "".to_string();
    let mut overdue_filter = "".to_string();
    let mut fines_filter = "".to_string();
    let mut binders = Vec::new();

    if let (Some(ref start_date), Some(ref end_date)) = (query_params.start_date, query_params.end_date) {
        let parsed_start = chrono::NaiveDate::parse_from_str(start_date, "%Y-%m-%d")
            .map_err(|_| AppError::BadRequest("Invalid startDate format".to_string()))?;
        let parsed_end = chrono::NaiveDate::parse_from_str(end_date, "%Y-%m-%d")
            .map_err(|_| AppError::BadRequest("Invalid endDate format".to_string()))?;

        binders.push(parsed_start.to_string());
        binders.push(parsed_end.to_string());

        books_filter = " WHERE added_date >= $1::date AND added_date <= $2::date".to_string();
        members_filter = " AND created_at >= $1::timestamp AND created_at <= $2::timestamp".to_string();
        issues_filter = " AND issue_date >= $1::date AND issue_date <= $2::date".to_string();
        overdue_filter = " AND issue_date >= $1::date AND issue_date <= $2::date".to_string();
        fines_filter = " AND issue_date >= $1::date AND issue_date <= $2::date".to_string();
    }

    let sql = format!(
        r#"
        SELECT
            (SELECT COUNT(*)::int8 FROM books{0} AND type = 'book') AS total_books,
            (SELECT COUNT(*)::int8 FROM books{0} AND type = 'international_journal') AS total_intl_journals,
            (SELECT COUNT(*)::int8 FROM books{0} AND type = 'national_journal') AS total_nat_journals,
            (SELECT COUNT(*)::int8 FROM books{0} AND type = 'magazine') AS total_magazines,
            (SELECT COUNT(*)::int8 FROM books{0}) AS total_all_catalogues,
            (SELECT COUNT(*)::int8 FROM library_members WHERE status = 'active'{1}) AS total_members,
            (SELECT COUNT(*)::int8 FROM book_issues WHERE status IN ('issued','overdue'){2}) AS books_issued,
            (SELECT COUNT(*)::int8 FROM book_issues WHERE status = 'overdue'{3}) AS overdue_books,
            (SELECT COALESCE(SUM(fine_amount), 0)::float8 FROM book_issues WHERE status = 'overdue' AND fine_paid = false{4}) AS pending_fines
        "#,
        books_filter, members_filter, issues_filter, overdue_filter, fines_filter
    );

    let mut db_query = sqlx::query(&sql);
    for val in binders {
        db_query = db_query.bind(val);
    }

    let row = db_query.fetch_one(&state.db).await?;

    let total_books: i64 = row.get("total_books");
    let total_intl_journals: i64 = row.get("total_intl_journals");
    let total_nat_journals: i64 = row.get("total_nat_journals");
    let total_magazines: i64 = row.get("total_magazines");
    let total_all_catalogues: i64 = row.get("total_all_catalogues");
    let total_members: i64 = row.get("total_members");
    let books_issued: i64 = row.get("books_issued");
    let overdue_books: i64 = row.get("overdue_books");
    let pending_fines: f64 = row.get("pending_fines");

    Ok(Json(ApiResponse {
        success: true,
        data: json!({
            "total_books": total_books,
            "total_intl_journals": total_intl_journals,
            "total_nat_journals": total_nat_journals,
            "total_magazines": total_magazines,
            "total_all_catalogues": total_all_catalogues,
            "total_members": total_members,
            "books_issued": books_issued,
            "overdue_books": overdue_books,
            "pending_fines": pending_fines
        }),
    }))
}

pub async fn get_activity(
    State(state): State<AppState>,
    _auth_user: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    let logs = sqlx::query(
        r#"
        SELECT al.id, al.user_id, al.action, al.entity_type, al.entity_id, al.meta, al.created_at, u.name AS actor_name
        FROM activity_log al
        LEFT JOIN users u ON u.id = al.user_id
        WHERE al.entity_type IN ('book', 'book_issue', 'library_member')
        ORDER BY al.created_at DESC
        LIMIT 10
        "#
    )
    .fetch_all(&state.db)
    .await?;

    let mut data = Vec::new();
    for row in logs {
        let id: Uuid = row.get("id");
        let user_id: Option<Uuid> = row.get("user_id");
        let action: String = row.get("action");
        let entity_type: Option<String> = row.get("entity_type");
        let entity_id: Option<Uuid> = row.get("entity_id");
        let meta_raw: Option<sqlx::types::Json<serde_json::Value>> = row.get("meta");
        let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
        let actor_name: Option<String> = row.get("actor_name");

        data.push(json!({
            "id": id,
            "user_id": user_id,
            "action": action,
            "entity_type": entity_type,
            "entity_id": entity_id,
            "meta": meta_raw.map(|j| j.0),
            "created_at": created_at,
            "actor_name": actor_name
        }));
    }

    Ok(Json(ApiResponse {
        success: true,
        data,
    }))
}

// ─── IMPORTS ─────────────────────────────────────────────

pub async fn import_books(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(payload): Json<ImportBooksPayload>,
) -> Result<impl IntoResponse, AppError> {
    let mut imported = Vec::new();

    for book in payload.books {
        let added_date = match &book.added_date {
            Some(d) if !d.is_empty() => chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").unwrap(),
            _ => Utc::now().date_naive(),
        };

        let price = book.price.unwrap_or(0.0);
        let quantity = book.quantity.unwrap_or(1);

        let upserted = sqlx::query_as::<_, Book>(
            r#"
            INSERT INTO books (acc_no, title, author, subject, price, quantity, added_date)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (acc_no) DO UPDATE
            SET title = EXCLUDED.title,
                author = EXCLUDED.author,
                subject = EXCLUDED.subject,
                price = EXCLUDED.price,
                quantity = EXCLUDED.quantity,
                added_date = COALESCE(EXCLUDED.added_date, books.added_date),
                updated_at = now()
            RETURNING id, acc_no, title, author, subject, price::float8 AS price, quantity, added_date, sl_no, type, volume, number_val, month, year, publisher, created_at, updated_at
            "#
        )
        .bind(&book.acc_no)
        .bind(&book.title)
        .bind(&book.author)
        .bind(&book.subject)
        .bind(price)
        .bind(quantity)
        .bind(added_date)
        .fetch_one(&state.db)
        .await?;

        imported.push(upserted);
    }

    if !imported.is_empty() {
        log_activity(
            &state.db,
            Some(auth_user.id),
            "BOOKS_IMPORTED",
            "book",
            Some(imported[0].id),
            Some(json!({ "count": imported.len() })),
        )
        .await?;
    }

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse {
            success: true,
            data: imported,
        }),
    ))
}

pub async fn import_members(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(payload): Json<ImportMembersPayload>,
) -> Result<impl IntoResponse, AppError> {
    let mut imported = Vec::new();

    for m in payload.members {
        if m.student_id.trim().is_empty() || m.name.trim().is_empty() {
            continue;
        }

        let existing = sqlx::query("SELECT id FROM library_members WHERE student_id = $1")
            .bind(&m.student_id)
            .fetch_optional(&state.db)
            .await?;

        let status = m.status.unwrap_or_else(|| "active".to_string());

        let member = if existing.is_some() {
            sqlx::query_as::<_, LibraryMember>(
                r#"
                UPDATE library_members
                SET name = $2, class = $3, phone = $4, status = $5, updated_at = now()
                WHERE student_id = $1
                RETURNING *
                "#
            )
            .bind(&m.student_id)
            .bind(&m.name)
            .bind(&m.class)
            .bind(&m.phone)
            .bind(&status)
            .fetch_one(&state.db)
            .await?
        } else {
            sqlx::query_as::<_, LibraryMember>(
                r#"
                INSERT INTO library_members (student_id, name, class, phone, status)
                VALUES ($1, $2, $3, $4, $5)
                RETURNING *
                "#
            )
            .bind(&m.student_id)
            .bind(&m.name)
            .bind(&m.class)
            .bind(&m.phone)
            .bind(&status)
            .fetch_one(&state.db)
            .await?
        };

        imported.push(member);
    }

    if !imported.is_empty() {
        log_activity(
            &state.db,
            Some(auth_user.id),
            "MEMBERS_IMPORTED",
            "library_member",
            Some(imported[0].id),
            Some(json!({ "count": imported.len() })),
        )
        .await?;
    }

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse {
            success: true,
            data: imported,
        }),
    ))
}

pub async fn import_returns(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(payload): Json<ImportReturnsPayload>,
) -> Result<impl IntoResponse, AppError> {
    let mut imported = Vec::new();

    let settings = sqlx::query_as::<_, LibrarySettings>(
        "SELECT id, issue_duration_days, fine_per_day::float8 AS fine_per_day, max_books_per_member, created_at, updated_at FROM library_settings LIMIT 1"
    )
    .fetch_one(&state.db)
    .await?;

    for ret in payload.returns {
        if ret.issue_no.trim().is_empty() {
            continue;
        }

        let issue = sqlx::query(
            r#"
            SELECT bi.id, bi.due_date, bi.status, b.title AS book_title, m.name AS member_name
            FROM book_issues bi
            JOIN books b ON b.id = bi.book_id
            JOIN library_members m ON m.id = bi.member_id
            WHERE bi.issue_no = $1
            "#
        )
        .bind(&ret.issue_no)
        .fetch_optional(&state.db)
        .await?;

        let issue_row = match issue {
            Some(row) => row,
            None => continue,
        };

        let status: String = issue_row.get("status");
        if status == "returned" {
            continue;
        }

        let issue_id: Uuid = issue_row.get("id");
        let due_date: chrono::NaiveDate = issue_row.get("due_date");
        let today = Utc::now().date_naive();
        let mut fine_amount = 0.0;

        if today > due_date {
            let days_overdue = today.signed_duration_since(due_date).num_days();
            fine_amount = (days_overdue as f64) * settings.fine_per_day;
        }

        let remarks = ret.remarks.unwrap_or_else(|| "—".to_string());

        let updated = sqlx::query_as::<_, BookIssue>(
            r#"
            UPDATE book_issues
            SET status = 'returned', return_date = CURRENT_DATE,
                fine_amount = $2, fine_paid = $3, remarks = $4, updated_at = now()
            WHERE id = $1
            RETURNING id, issue_no, member_id, book_id, issued_by, issue_date, due_date, return_date, fine_amount::float8 AS fine_amount, fine_paid, status, remarks, created_at, updated_at
            "#
        )
        .bind(issue_id)
        .bind(fine_amount)
        .bind(ret.fine_paid)
        .bind(&remarks)
        .fetch_one(&state.db)
        .await?;

        let book_title: String = issue_row.get("book_title");
        let member_name: String = issue_row.get("member_name");

        log_activity(
            &state.db,
            Some(auth_user.id),
            "BOOK_RETURNED",
            "book_issue",
            Some(issue_id),
            Some(json!({ "book": book_title, "member": member_name, "fine": fine_amount, "imported": true })),
        )
        .await?;

        imported.push(updated);
    }

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse {
            success: true,
            data: imported,
        }),
    ))
}
