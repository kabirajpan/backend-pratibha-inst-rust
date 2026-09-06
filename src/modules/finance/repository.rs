// backend-rust/src/modules/finance/repository.rs
//! Finance Data Access Layer (Repository)

use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;
use super::models::{
    Expense, ExpenseWithCount, FeeRecord, FeeRecordWithDetails,
};

// ─── Fee Collections Queries ──────────────────────────────────────────────────

pub async fn find_fee_records(
    pool: &PgPool,
    sql: &str,
    binders: Vec<String>,
    limit: i64,
    offset: i64,
) -> Result<Vec<FeeRecordWithDetails>, sqlx::Error> {
    let mut db_query = sqlx::query_as::<_, FeeRecordWithDetails>(sql);
    for val in binders {
        db_query = db_query.bind(val);
    }
    db_query = db_query.bind(limit).bind(offset);

    db_query.fetch_all(pool).await
}

pub async fn find_fee_record_by_id(pool: &PgPool, id: Uuid) -> Result<Option<FeeRecordWithDetails>, sqlx::Error> {
    sqlx::query_as::<_, FeeRecordWithDetails>(
        r#"
        SELECT f.id, f.student_id, f.fee_type, f.room, f.bus_route, f.bus_no, f.receipt_book_no, f.receipt_no, f.receipt_date, f.payment_date,
               f.amount::float8 AS amount, f.utr_no, f.payment_mode, f.due_fees::float8 AS due_fees, f.remarks, f.discount::float8 AS discount, f.created_at,
               COALESCE(NULLIF(s.name, ''), f.student_id) AS student_name, s.class_name AS class_name, s.course_name AS course_name, 1::int AS total_count
        FROM fee_collections f
        LEFT JOIN students s ON s.student_id = f.student_id
        WHERE f.id = $1
        "#
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn delete_fee_record(pool: &PgPool, id: Uuid) -> Result<Option<FeeRecord>, sqlx::Error> {
    sqlx::query_as::<_, FeeRecord>(
        r#"
        DELETE FROM fee_collections WHERE id = $1
        RETURNING id, student_id, fee_type, room, bus_route, bus_no, receipt_book_no, receipt_no,
                  receipt_date, payment_date, amount::float8 AS amount, utr_no, payment_mode,
                  due_fees::float8 AS due_fees, remarks, discount::float8 AS discount, created_at
        "#
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

// ─── Expenses Ledger Queries ──────────────────────────────────────────────────

pub async fn find_expenses(
    pool: &PgPool,
    sql: &str,
    binders: Vec<String>,
    limit: i64,
    offset: i64,
) -> Result<Vec<ExpenseWithCount>, sqlx::Error> {
    let mut db_query = sqlx::query_as::<_, ExpenseWithCount>(sql);
    for val in binders {
        db_query = db_query.bind(val);
    }
    db_query = db_query.bind(limit).bind(offset);

    db_query.fetch_all(pool).await
}

pub async fn find_expense_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Expense>, sqlx::Error> {
    sqlx::query_as::<_, Expense>(
        r#"
        SELECT id, ref_no, description, amount::float8 AS amount, category, date, payment_mode,
               remarks, utr, receipt, party_name, spent_by, voucher_no, created_at
        FROM expenses WHERE id = $1
        "#
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn insert_expense(
    pool: &PgPool,
    ref_no: &str,
    description: &str,
    amount: f64,
    category: &str,
    date: NaiveDate,
    payment_mode: &str,
    remarks: Option<&str>,
    utr: Option<&str>,
    receipt: Option<&str>,
    party_name: Option<&str>,
    spent_by: Option<&str>,
    voucher_no: Option<&str>,
) -> Result<Expense, sqlx::Error> {
    sqlx::query_as::<_, Expense>(
        r#"
        INSERT INTO expenses (ref_no, description, amount, category, date, payment_mode, remarks, utr, receipt, party_name, spent_by, voucher_no)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        RETURNING id, ref_no, description, amount::float8 AS amount, category, date, payment_mode,
                  remarks, utr, receipt, party_name, spent_by, voucher_no, created_at
        "#
    )
    .bind(ref_no)
    .bind(description)
    .bind(amount)
    .bind(category)
    .bind(date)
    .bind(payment_mode)
    .bind(remarks)
    .bind(utr)
    .bind(receipt)
    .bind(party_name)
    .bind(spent_by)
    .bind(voucher_no)
    .fetch_one(pool)
    .await
}

pub async fn update_expense(
    pool: &PgPool,
    e: &Expense,
) -> Result<Expense, sqlx::Error> {
    sqlx::query_as::<_, Expense>(
        r#"
        UPDATE expenses
        SET description = $1, amount = $2, category = $3, date = $4, payment_mode = $5, remarks = $6,
            utr = $7, receipt = $8, party_name = $9, spent_by = $10, voucher_no = $11
        WHERE id = $12
        RETURNING id, ref_no, description, amount::float8 AS amount, category, date, payment_mode,
                  remarks, utr, receipt, party_name, spent_by, voucher_no, created_at
        "#
    )
    .bind(&e.description)
    .bind(e.amount)
    .bind(&e.category)
    .bind(e.date)
    .bind(&e.payment_mode)
    .bind(&e.remarks)
    .bind(&e.utr)
    .bind(&e.receipt)
    .bind(&e.party_name)
    .bind(&e.spent_by)
    .bind(&e.voucher_no)
    .bind(e.id)
    .fetch_one(pool)
    .await
}

pub async fn delete_expense(pool: &PgPool, id: Uuid) -> Result<Option<Expense>, sqlx::Error> {
    sqlx::query_as::<_, Expense>(
        r#"
        DELETE FROM expenses WHERE id = $1
        RETURNING id, ref_no, description, amount::float8 AS amount, category, date, payment_mode,
                  remarks, utr, receipt, party_name, spent_by, voucher_no, created_at
        "#
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}
