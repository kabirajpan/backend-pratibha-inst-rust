// backend-rust/src/modules/library/service.rs
//! Library Business Logic & Domain Service Layer

use chrono::{Duration, NaiveDate, Utc};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;
use crate::errors::AppError;
use crate::utils::activity::log_activity;
use super::models::{
    Book, BookIssue, BookIssueWithDetails, BookWithStatus, CreateBookPayload,
    CreateMemberPayload, EditBookIssuePayload, GetBooksQuery, GetIssuesQuery,
    GetMembersQuery, ImportBookRow, ImportMemberRow, ImportReturnRow,
    IssueBookPayload, LibraryActivityLog, LibraryMember, LibraryMemberWithStats,
    LibrarySettings, LibraryStats, ReturnBookPayload, UpdateBookPayload,
    UpdateFinePayload, UpdateMemberPayload, UpdateSettingsPayload,
};
use super::repository;

// ─── Overdue Sync ─────────────────────────────────────────────────────────────

pub async fn sync_overdue(pool: &PgPool) -> Result<(), AppError> {
    repository::sync_overdue_status(pool)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to sync overdue status: {}", e)))
}

// ─── Stats & Activity ─────────────────────────────────────────────────────────

pub async fn get_stats(pool: &PgPool) -> Result<LibraryStats, AppError> {
    sync_overdue(pool).await?;

    let total_books = sqlx::query_scalar::<_, i64>("SELECT COALESCE(SUM(quantity), 0)::bigint FROM books")
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let unique_titles = sqlx::query_scalar::<_, i64>("SELECT COUNT(*)::bigint FROM books")
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let total_members = sqlx::query_scalar::<_, i64>("SELECT COUNT(*)::bigint FROM library_members")
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let active_members = sqlx::query_scalar::<_, i64>("SELECT COUNT(*)::bigint FROM library_members WHERE status = 'active'")
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let issued_books = sqlx::query_scalar::<_, i64>("SELECT COUNT(*)::bigint FROM book_issues WHERE status = 'issued'")
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let overdue_books = sqlx::query_scalar::<_, i64>("SELECT COUNT(*)::bigint FROM book_issues WHERE status = 'overdue'")
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let available_books = (total_books - issued_books - overdue_books).max(0);

    let total_fines = sqlx::query_scalar::<_, f64>("SELECT COALESCE(SUM(fine_amount), 0.0)::float8 FROM book_issues")
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let collected_fines = sqlx::query_scalar::<_, f64>("SELECT COALESCE(SUM(fine_amount), 0.0)::float8 FROM book_issues WHERE fine_paid = true")
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let pending_fines = total_fines - collected_fines;

    Ok(LibraryStats {
        total_books,
        unique_titles,
        available_books,
        issued_books,
        overdue_books,
        total_members,
        active_members,
        total_fines,
        collected_fines,
        pending_fines,
    })
}

pub async fn get_activity(pool: &PgPool) -> Result<Vec<LibraryActivityLog>, AppError> {
    let logs = sqlx::query_as::<_, LibraryActivityLog>(
        r#"
        SELECT bi.id, bi.issue_no, bi.status AS action, bi.updated_at AS timestamp,
            m.name AS member_name, m.student_id,
            b.title AS book_title,
            u.name AS actor_name
        FROM book_issues bi
        JOIN library_members m ON m.id = bi.member_id
        JOIN books b ON b.id = bi.book_id
        LEFT JOIN users u ON u.id = bi.issued_by
        ORDER BY bi.updated_at DESC
        LIMIT 10
        "#
    )
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::Internal(format!("Failed to fetch library activity: {}", e)))?;

    Ok(logs)
}

// ─── Books ───────────────────────────────────────────────────────────────────

pub async fn list_books(
    pool: &PgPool,
    q: &GetBooksQuery,
) -> Result<(Vec<BookWithStatus>, i32, i64, i64), AppError> {
    let page = q.page.unwrap_or(1).max(1);
    let limit = q.limit.unwrap_or(50).max(1);
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

    if let Some(ref r#type) = q.r#type {
        sql.push_str(&format!(" AND b.type = ${}", count_idx));
        binders.push(r#type.clone());
        count_idx += 1;
    }

    if let Some(ref search) = q.search {
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

    let list = repository::find_books(pool, &sql, binders, limit, offset)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to fetch books: {}", e)))?;

    let total = if list.is_empty() { 0 } else { list[0].total_count };

    Ok((list, total, page, limit))
}

pub async fn get_book(pool: &PgPool, id: Uuid) -> Result<BookWithStatus, AppError> {
    repository::find_book_by_id(pool, id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Book not found".to_string()))
}

pub async fn create_book(
    pool: &PgPool,
    actor_id: Uuid,
    payload: CreateBookPayload,
) -> Result<Book, AppError> {
    payload.validate()?;

    let added_date = match &payload.added_date {
        Some(d) if !d.is_empty() => NaiveDate::parse_from_str(d, "%Y-%m-%d")
            .map_err(|_| AppError::BadRequest("Invalid added_date format".to_string()))?,
        _ => Utc::now().date_naive(),
    };

    let r#type = payload.r#type.unwrap_or_else(|| "book".to_string());
    let price = payload.price.unwrap_or(0.0);
    let quantity = payload.quantity.unwrap_or(1);

    let book = repository::insert_book(
        pool,
        payload.acc_no.as_deref(),
        payload.title.trim(),
        payload.author.trim(),
        payload.subject.as_deref(),
        price,
        quantity,
        added_date,
        payload.sl_no.as_deref(),
        &r#type,
        payload.volume.as_deref(),
        payload.number_val.as_deref(),
        payload.month.as_deref(),
        payload.year.as_deref(),
        payload.publisher.as_deref(),
    )
    .await
    .map_err(|e| AppError::Internal(format!("Failed to create book: {}", e)))?;

    log_activity(
        pool,
        Some(actor_id),
        "BOOK_ADDED",
        "library",
        Some(book.id),
        Some(json!({ "title": book.title, "author": book.author })),
    )
    .await?;

    Ok(book)
}

pub async fn update_book(
    pool: &PgPool,
    actor_id: Uuid,
    id: Uuid,
    payload: UpdateBookPayload,
) -> Result<Book, AppError> {
    payload.validate()?;

    let existing = repository::find_raw_book_by_id(pool, id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Book not found".to_string()))?;

    let mut updated = existing;
    if let Some(acc) = payload.acc_no { updated.acc_no = Some(acc); }
    if let Some(t) = payload.title { updated.title = t; }
    if let Some(a) = payload.author { updated.author = a; }
    if let Some(s) = payload.subject { updated.subject = Some(s); }
    if let Some(p) = payload.price { updated.price = p; }
    if let Some(q) = payload.quantity { updated.quantity = q; }
    if let Some(ref d) = payload.added_date {
        if let Ok(parsed) = NaiveDate::parse_from_str(d, "%Y-%m-%d") {
            updated.added_date = parsed;
        }
    }
    if let Some(sl) = payload.sl_no { updated.sl_no = Some(sl); }
    if let Some(t) = payload.r#type { updated.r#type = t; }
    if let Some(v) = payload.volume { updated.volume = Some(v); }
    if let Some(n) = payload.number_val { updated.number_val = Some(n); }
    if let Some(m) = payload.month { updated.month = Some(m); }
    if let Some(y) = payload.year { updated.year = Some(y); }
    if let Some(publ) = payload.publisher { updated.publisher = Some(publ); }

    let book = repository::update_book(pool, &updated)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to update book: {}", e)))?;

    log_activity(
        pool,
        Some(actor_id),
        "BOOK_UPDATED",
        "library",
        Some(book.id),
        Some(json!({ "title": book.title })),
    )
    .await?;

    Ok(book)
}

pub async fn delete_book(pool: &PgPool, actor_id: Uuid, id: Uuid) -> Result<Book, AppError> {
    let deleted = repository::delete_book(pool, id)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to delete book: {}", e)))?
        .ok_or_else(|| AppError::NotFound("Book not found".to_string()))?;

    log_activity(
        pool,
        Some(actor_id),
        "BOOK_DELETED",
        "library",
        Some(deleted.id),
        Some(json!({ "title": deleted.title })),
    )
    .await?;

    Ok(deleted)
}

// ─── Members ─────────────────────────────────────────────────────────────────

pub async fn list_members(
    pool: &PgPool,
    q: &GetMembersQuery,
) -> Result<(Vec<LibraryMemberWithStats>, i32, i64, i64), AppError> {
    let page = q.page.unwrap_or(1).max(1);
    let limit = q.limit.unwrap_or(50).max(1);
    let offset = (page - 1) * limit;

    let mut sql = r#"
        SELECT m.id, m.student_id, m.user_id, m.name,
               COALESCE(s.class_name, m.class) AS class,
               COALESCE(s.course_name, m.course) AS course,
               m.phone, m.status, m.created_at, m.updated_at,
            COALESCE(issued.count, 0)::bigint AS currently_issued,
            COALESCE(total.count, 0)::bigint AS total_issued,
            COUNT(*) OVER()::int AS total_count
        FROM library_members m
        LEFT JOIN students s ON s.student_id = m.student_id
        LEFT JOIN (
            SELECT member_id, COUNT(*) AS count
            FROM book_issues
            WHERE status = 'issued' OR status = 'overdue'
            GROUP BY member_id
        ) issued ON issued.member_id = m.id
        LEFT JOIN (
            SELECT member_id, COUNT(*) AS count
            FROM book_issues
            GROUP BY member_id
        ) total ON total.member_id = m.id
        WHERE 1=1
    "#
    .to_string();

    let mut binders = Vec::new();
    let mut idx = 1usize;

    if let Some(ref search) = q.search {
        let term = format!("%{}%", search.trim());
        sql.push_str(&format!(" AND (m.name ILIKE ${idx} OR m.student_id ILIKE ${idx} OR m.phone ILIKE ${idx})"));
        binders.push(term);
        idx += 1;
    }

    if let Some(ref status) = q.status {
        if status != "All Statuses" {
            sql.push_str(&format!(" AND m.status = ${idx}"));
            binders.push(status.clone());
            idx += 1;
        }
    }

    sql.push_str(" ORDER BY m.name ASC");
    sql.push_str(&format!(" LIMIT ${idx} OFFSET ${}", idx + 1));

    let list = repository::find_members(pool, &sql, binders, limit, offset)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to fetch members: {}", e)))?;

    let total = if list.is_empty() { 0 } else { list[0].total_count };

    Ok((list, total, page, limit))
}

pub async fn get_member(pool: &PgPool, id: Uuid) -> Result<LibraryMemberWithStats, AppError> {
    repository::find_member_by_id(pool, id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Member not found".to_string()))
}

// ─── Settings ────────────────────────────────────────────────────────────────

pub async fn get_settings(pool: &PgPool) -> Result<LibrarySettings, AppError> {
    repository::find_settings(pool)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to fetch library settings: {}", e)))
}

pub async fn update_settings(
    pool: &PgPool,
    actor_id: Uuid,
    payload: UpdateSettingsPayload,
) -> Result<LibrarySettings, AppError> {
    payload.validate()?;

    let current = repository::find_settings(pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let issue_limit = payload.max_books_per_member.unwrap_or(current.issue_limit);
    let return_days = payload.issue_duration_days.unwrap_or(current.return_days);
    let fine_per_day = payload.fine_per_day.unwrap_or(current.fine_per_day);

    let updated = repository::update_settings(pool, issue_limit, return_days, fine_per_day)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to update library settings: {}", e)))?;

    log_activity(
        pool,
        Some(actor_id),
        "LIBRARY_SETTINGS_UPDATED",
        "library",
        Some(updated.id),
        Some(json!({ "issue_limit": updated.issue_limit, "fine_per_day": updated.fine_per_day })),
    )
    .await?;

    Ok(updated)
}
