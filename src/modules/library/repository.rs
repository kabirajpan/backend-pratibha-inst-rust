// backend-rust/src/modules/library/repository.rs
//! Library Data Access Layer (Repository)

use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;
use super::models::{
    Book, BookIssue, BookIssueWithDetails, BookWithStatus,
    LibraryActivityLog, LibraryMember, LibraryMemberWithStats,
    LibrarySettings,
};

// ─── Overdue Status Sync ──────────────────────────────────────────────────────

pub async fn sync_overdue_status(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE book_issues bi
        SET status = 'overdue',
            fine_amount = (CURRENT_DATE - bi.due_date) * s.fine_per_day
        FROM library_settings s
        WHERE bi.status IN ('issued', 'overdue') AND bi.due_date < CURRENT_DATE
        "#
    )
    .execute(pool)
    .await?;

    Ok(())
}

// ─── Books Queries ────────────────────────────────────────────────────────────

pub async fn find_books(
    pool: &PgPool,
    sql: &str,
    binders: Vec<String>,
    limit: i64,
    offset: i64,
) -> Result<Vec<BookWithStatus>, sqlx::Error> {
    let mut db_query = sqlx::query_as::<_, BookWithStatus>(sql);
    for val in binders {
        db_query = db_query.bind(val);
    }
    db_query = db_query.bind(limit).bind(offset);

    db_query.fetch_all(pool).await
}

pub async fn find_book_by_id(pool: &PgPool, id: Uuid) -> Result<Option<BookWithStatus>, sqlx::Error> {
    sqlx::query_as::<_, BookWithStatus>(
        r#"
        SELECT b.id, b.acc_no, b.title, b.author, b.subject, b.price::float8 AS price, b.quantity, b.added_date, b.sl_no, b.type, b.volume, b.number_val, b.month, b.year, b.publisher, b.created_at, b.updated_at,
            (b.quantity - COALESCE(active.count, 0))::int AS available_quantity,
            CASE
                WHEN b.quantity = 0 THEN 'unavailable'
                WHEN (b.quantity - COALESCE(active.count, 0)) <= 0 THEN 'issued'
                ELSE 'available'
            END AS status,
            1::int AS total_count
        FROM books b
        LEFT JOIN (
            SELECT book_id, COUNT(*) AS count
            FROM book_issues
            WHERE status = 'issued' OR status = 'overdue'
            GROUP BY book_id
        ) active ON active.book_id = b.id
        WHERE b.id = $1
        "#
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn find_raw_book_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Book>, sqlx::Error> {
    sqlx::query_as::<_, Book>(
        "SELECT id, acc_no, title, author, subject, price::float8 AS price, quantity, added_date, sl_no, type, volume, number_val, month, year, publisher, created_at, updated_at FROM books WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn find_book_by_acc_no(pool: &PgPool, acc_no: &str) -> Result<Option<Uuid>, sqlx::Error> {
    let row = sqlx::query_scalar::<_, Uuid>("SELECT id FROM books WHERE LOWER(TRIM(acc_no)) = LOWER(TRIM($1))")
        .bind(acc_no)
        .fetch_optional(pool)
        .await?;

    Ok(row)
}

pub async fn insert_book(
    pool: &PgPool,
    acc_no: Option<&str>,
    title: &str,
    author: &str,
    subject: Option<&str>,
    price: f64,
    quantity: i32,
    added_date: NaiveDate,
    sl_no: Option<&str>,
    r#type: &str,
    volume: Option<&str>,
    number_val: Option<&str>,
    month: Option<&str>,
    year: Option<&str>,
    publisher: Option<&str>,
) -> Result<Book, sqlx::Error> {
    sqlx::query_as::<_, Book>(
        r#"
        INSERT INTO books (acc_no, title, author, subject, price, quantity, added_date, sl_no, type, volume, number_val, month, year, publisher)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
        RETURNING id, acc_no, title, author, subject, price::float8 AS price, quantity, added_date, sl_no, type, volume, number_val, month, year, publisher, created_at, updated_at
        "#
    )
    .bind(acc_no)
    .bind(title)
    .bind(author)
    .bind(subject)
    .bind(price)
    .bind(quantity)
    .bind(added_date)
    .bind(sl_no)
    .bind(r#type)
    .bind(volume)
    .bind(number_val)
    .bind(month)
    .bind(year)
    .bind(publisher)
    .fetch_one(pool)
    .await
}

pub async fn update_book(
    pool: &PgPool,
    b: &Book,
) -> Result<Book, sqlx::Error> {
    sqlx::query_as::<_, Book>(
        r#"
        UPDATE books
        SET acc_no = $1, title = $2, author = $3, subject = $4, price = $5, quantity = $6, added_date = $7,
            sl_no = $8, type = $9, volume = $10, number_val = $11, month = $12, year = $13, publisher = $14, updated_at = NOW()
        WHERE id = $15
        RETURNING id, acc_no, title, author, subject, price::float8 AS price, quantity, added_date, sl_no, type, volume, number_val, month, year, publisher, created_at, updated_at
        "#
    )
    .bind(&b.acc_no)
    .bind(&b.title)
    .bind(&b.author)
    .bind(&b.subject)
    .bind(b.price)
    .bind(b.quantity)
    .bind(b.added_date)
    .bind(&b.sl_no)
    .bind(&b.r#type)
    .bind(&b.volume)
    .bind(&b.number_val)
    .bind(&b.month)
    .bind(&b.year)
    .bind(&b.publisher)
    .bind(b.id)
    .fetch_one(pool)
    .await
}

pub async fn delete_book(pool: &PgPool, id: Uuid) -> Result<Option<Book>, sqlx::Error> {
    sqlx::query_as::<_, Book>(
        "DELETE FROM books WHERE id = $1 RETURNING id, acc_no, title, author, subject, price::float8 AS price, quantity, added_date, sl_no, type, volume, number_val, month, year, publisher, created_at, updated_at"
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

// ─── Members Queries ──────────────────────────────────────────────────────────

pub async fn find_members(
    pool: &PgPool,
    sql: &str,
    binders: Vec<String>,
    limit: i64,
    offset: i64,
) -> Result<Vec<LibraryMemberWithStats>, sqlx::Error> {
    let mut db_query = sqlx::query_as::<_, LibraryMemberWithStats>(sql);
    for val in binders {
        db_query = db_query.bind(val);
    }
    db_query = db_query.bind(limit).bind(offset);

    db_query.fetch_all(pool).await
}

pub async fn find_member_by_id(pool: &PgPool, id: Uuid) -> Result<Option<LibraryMemberWithStats>, sqlx::Error> {
    sqlx::query_as::<_, LibraryMemberWithStats>(
        r#"
        SELECT m.id, m.student_id, m.user_id, m.name,
               COALESCE(s.class_name, m.class) AS class,
               COALESCE(s.course_name, m.course) AS course,
               m.phone, m.status, m.created_at, m.updated_at,
            COALESCE(issued.count, 0)::bigint AS currently_issued,
            COALESCE(total.count, 0)::bigint AS total_issued,
            1::int AS total_count
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
        WHERE m.id = $1
        "#
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn find_member_by_student_id(pool: &PgPool, student_id: &str) -> Result<Option<Uuid>, sqlx::Error> {
    let row = sqlx::query_scalar::<_, Uuid>("SELECT id FROM library_members WHERE LOWER(TRIM(student_id)) = LOWER(TRIM($1))")
        .bind(student_id)
        .fetch_optional(pool)
        .await?;

    Ok(row)
}

pub async fn insert_member(
    pool: &PgPool,
    student_id: &str,
    name: &str,
    class_name: Option<&str>,
    course_name: Option<&str>,
    phone: Option<&str>,
    status: &str,
) -> Result<LibraryMember, sqlx::Error> {
    sqlx::query_as::<_, LibraryMember>(
        r#"
        INSERT INTO library_members (student_id, name, class, course, phone, status, class_name, course_name)
        VALUES ($1, $2, $3, $4, $5, $6, $3, $4)
        RETURNING id, student_id, user_id, name, class, course, phone, status, created_at, updated_at
        "#
    )
    .bind(student_id)
    .bind(name)
    .bind(class_name)
    .bind(course_name)
    .bind(phone)
    .bind(status)
    .fetch_one(pool)
    .await
}

pub async fn update_member(
    pool: &PgPool,
    m: &LibraryMember,
) -> Result<LibraryMember, sqlx::Error> {
    sqlx::query_as::<_, LibraryMember>(
        r#"
        UPDATE library_members
        SET name = $1, class = $2, course = $3, phone = $4, status = $5, class_name = $2, course_name = $3, updated_at = NOW()
        WHERE id = $6
        RETURNING id, student_id, user_id, name, class, course, phone, status, created_at, updated_at
        "#
    )
    .bind(&m.name)
    .bind(&m.class)
    .bind(&m.course)
    .bind(&m.phone)
    .bind(&m.status)
    .bind(m.id)
    .fetch_one(pool)
    .await
}

// ─── Issues Queries ───────────────────────────────────────────────────────────

pub async fn find_issues(
    pool: &PgPool,
    sql: &str,
    binders: Vec<String>,
    limit: i64,
    offset: i64,
) -> Result<Vec<BookIssueWithDetails>, sqlx::Error> {
    let mut db_query = sqlx::query_as::<_, BookIssueWithDetails>(sql);
    for val in binders {
        db_query = db_query.bind(val);
    }
    db_query = db_query.bind(limit).bind(offset);

    db_query.fetch_all(pool).await
}

pub async fn find_issue_by_id(pool: &PgPool, id: Uuid) -> Result<Option<BookIssue>, sqlx::Error> {
    sqlx::query_as::<_, BookIssue>(
        "SELECT id, issue_no, member_id, book_id, issued_by, issue_date, due_date, return_date, fine_amount::float8 AS fine_amount, fine_paid, status, remarks, created_at, updated_at FROM book_issues WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn delete_issue(pool: &PgPool, id: Uuid) -> Result<Option<BookIssue>, sqlx::Error> {
    sqlx::query_as::<_, BookIssue>(
        "DELETE FROM book_issues WHERE id = $1 RETURNING id, issue_no, member_id, book_id, issued_by, issue_date, due_date, return_date, fine_amount::float8 AS fine_amount, fine_paid, status, remarks, created_at, updated_at"
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

// ─── Settings Queries ─────────────────────────────────────────────────────────

pub async fn find_settings(pool: &PgPool) -> Result<LibrarySettings, sqlx::Error> {
    sqlx::query_as::<_, LibrarySettings>(
        "SELECT id, issue_limit, return_days, fine_per_day::float8 AS fine_per_day, updated_at FROM library_settings LIMIT 1"
    )
    .fetch_one(pool)
    .await
}

pub async fn update_settings(
    pool: &PgPool,
    issue_limit: i32,
    return_days: i32,
    fine_per_day: f64,
) -> Result<LibrarySettings, sqlx::Error> {
    sqlx::query_as::<_, LibrarySettings>(
        r#"
        UPDATE library_settings
        SET issue_limit = $1, return_days = $2, fine_per_day = $3, updated_at = NOW()
        WHERE id = (SELECT id FROM library_settings LIMIT 1)
        RETURNING id, issue_limit, return_days, fine_per_day::float8 AS fine_per_day, updated_at
        "#
    )
    .bind(issue_limit)
    .bind(return_days)
    .bind(fine_per_day)
    .fetch_one(pool)
    .await
}
