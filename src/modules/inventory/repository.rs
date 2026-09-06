// src/modules/inventory/repository.rs
//! Inventory Data Access Layer (Repository)

use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;
use super::models::{
    CategorySimpleRow, InventoryCategoryRow, InventoryIssueRow,
    InventoryItemRow, LowStockItemRow, RawIssueRow, RawItemRow, SingleItemRow, StatsRow,
};

// ─── Stats & Low Stock Queries ───────────────────────────────────────────────

pub async fn find_stats(pool: &PgPool) -> Result<StatsRow, sqlx::Error> {
    sqlx::query_as::<_, StatsRow>(
        r#"
        SELECT
            (SELECT COUNT(*)::bigint FROM inventory_items) AS total_items,
            (SELECT COUNT(*)::bigint FROM inventory_categories) AS total_categories,
            (SELECT COUNT(*)::bigint FROM inventory_items WHERE available_qty <= low_stock_threshold) AS low_stock_items,
            (SELECT COALESCE(SUM(available_qty::float8 * unit_price::float8), 0.0) FROM inventory_items) AS total_value
        "#
    )
    .fetch_one(pool)
    .await
}

pub async fn find_low_stock(pool: &PgPool) -> Result<Vec<LowStockItemRow>, sqlx::Error> {
    sqlx::query_as::<_, LowStockItemRow>(
        r#"
        SELECT i.id, i.category_id, i.name, i.required_qty, i.available_qty, 
               i.unit_price::float8 AS unit_price, i.low_stock_threshold, i.unit, 
               i.description, i.created_by, i.created_at, i.updated_at,
               c.name AS category_name,
               (i.required_qty - i.available_qty) AS shortage
        FROM inventory_items i
        JOIN inventory_categories c ON c.id = i.category_id
        WHERE i.available_qty <= i.low_stock_threshold
        ORDER BY (i.required_qty - i.available_qty) DESC
        "#
    )
    .fetch_all(pool)
    .await
}

// ─── Categories Queries ──────────────────────────────────────────────────────

pub async fn find_all_categories(pool: &PgPool) -> Result<Vec<InventoryCategoryRow>, sqlx::Error> {
    sqlx::query_as::<_, InventoryCategoryRow>(
        r#"
        SELECT c.id, c.name, c.description, c.created_at, c.updated_at,
               COUNT(i.id)::bigint AS total_items
        FROM inventory_categories c
        LEFT JOIN inventory_items i ON i.category_id = c.id
        GROUP BY c.id
        ORDER BY c.name ASC
        "#
    )
    .fetch_all(pool)
    .await
}

pub async fn find_category_by_name(pool: &PgPool, name: &str) -> Result<Option<Uuid>, sqlx::Error> {
    let row = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM inventory_categories WHERE LOWER(TRIM(name)) = LOWER(TRIM($1))"
    )
    .bind(name)
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

pub async fn create_category(
    pool: &PgPool,
    name: &str,
    description: Option<&str>,
) -> Result<CategorySimpleRow, sqlx::Error> {
    sqlx::query_as::<_, CategorySimpleRow>(
        r#"
        INSERT INTO inventory_categories (name, description)
        VALUES ($1, $2)
        RETURNING id, name, description, created_at, updated_at
        "#
    )
    .bind(name)
    .bind(description)
    .fetch_one(pool)
    .await
}

pub async fn update_category(
    pool: &PgPool,
    id: Uuid,
    name: &str,
    description: Option<&str>,
) -> Result<CategorySimpleRow, sqlx::Error> {
    sqlx::query_as::<_, CategorySimpleRow>(
        r#"
        UPDATE inventory_categories
        SET name = $1, description = $2, updated_at = NOW()
        WHERE id = $3
        RETURNING id, name, description, created_at, updated_at
        "#
    )
    .bind(name)
    .bind(description)
    .bind(id)
    .fetch_one(pool)
    .await
}

pub async fn delete_category(pool: &PgPool, id: Uuid) -> Result<u64, sqlx::Error> {
    let res = sqlx::query("DELETE FROM inventory_categories WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(res.rows_affected())
}

// ─── Items Queries ───────────────────────────────────────────────────────────

pub async fn find_items(
    pool: &PgPool,
    sql: &str,
    binders: Vec<String>,
    limit: i64,
    offset: i64,
) -> Result<Vec<InventoryItemRow>, sqlx::Error> {
    let mut db_query = sqlx::query_as::<_, InventoryItemRow>(sql);
    for val in binders {
        db_query = db_query.bind(val);
    }
    db_query = db_query.bind(limit).bind(offset);

    db_query.fetch_all(pool).await
}

pub async fn find_item_by_id(pool: &PgPool, id: Uuid) -> Result<Option<SingleItemRow>, sqlx::Error> {
    sqlx::query_as::<_, SingleItemRow>(
        r#"
        SELECT i.id, i.category_id, i.name, i.required_qty, i.available_qty, 
               i.unit_price::float8 AS unit_price, i.low_stock_threshold, i.unit, 
               i.description, i.created_by, i.created_at, i.updated_at,
               c.name AS category_name,
               u.name AS entry_by_name,
               (i.available_qty <= i.low_stock_threshold) AS is_low_stock,
               (i.available_qty::float8 * i.unit_price::float8) AS total_value
        FROM inventory_items i
        JOIN inventory_categories c ON c.id = i.category_id
        LEFT JOIN users u ON u.id = i.created_by
        WHERE i.id = $1
        "#
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn find_raw_item_by_id(pool: &PgPool, id: Uuid) -> Result<Option<RawItemRow>, sqlx::Error> {
    sqlx::query_as::<_, RawItemRow>(
        r#"
        SELECT id, category_id, name, required_qty, available_qty, unit_price::float8 AS unit_price,
               low_stock_threshold, unit, description, created_by, created_at, updated_at
        FROM inventory_items
        WHERE id = $1
        "#
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn find_item_id_by_name(pool: &PgPool, name: &str) -> Result<Option<Uuid>, sqlx::Error> {
    let row = sqlx::query_scalar::<_, Uuid>("SELECT id FROM inventory_items WHERE LOWER(TRIM(name)) = LOWER(TRIM($1))")
        .bind(name)
        .fetch_optional(pool)
        .await?;

    Ok(row)
}

pub async fn insert_item(
    pool: &PgPool,
    category_id: Uuid,
    name: &str,
    required_qty: i32,
    available_qty: i32,
    unit_price: f64,
    low_stock_threshold: i32,
    unit: &str,
    description: Option<&str>,
    created_by: Option<Uuid>,
) -> Result<SingleItemRow, sqlx::Error> {
    let raw = sqlx::query_as::<_, RawItemRow>(
        r#"
        INSERT INTO inventory_items (
            category_id, name, required_qty, available_qty, unit_price, 
            low_stock_threshold, unit, description, created_by
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING id, category_id, name, required_qty, available_qty, unit_price::float8 AS unit_price,
                  low_stock_threshold, unit, description, created_by, created_at, updated_at
        "#
    )
    .bind(category_id)
    .bind(name)
    .bind(required_qty)
    .bind(available_qty)
    .bind(unit_price)
    .bind(low_stock_threshold)
    .bind(unit)
    .bind(description)
    .bind(created_by)
    .fetch_one(pool)
    .await?;

    find_item_by_id(pool, raw.id).await?.ok_or_else(|| sqlx::Error::RowNotFound)
}

pub async fn update_item(
    pool: &PgPool,
    id: Uuid,
    category_id: Uuid,
    name: &str,
    required_qty: i32,
    available_qty: i32,
    unit_price: f64,
    low_stock_threshold: i32,
    unit: &str,
    description: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE inventory_items
        SET category_id = $1, name = $2, required_qty = $3, available_qty = $4,
            unit_price = $5, low_stock_threshold = $6, unit = $7, description = $8, updated_at = NOW()
        WHERE id = $9
        "#
    )
    .bind(category_id)
    .bind(name)
    .bind(required_qty)
    .bind(available_qty)
    .bind(unit_price)
    .bind(low_stock_threshold)
    .bind(unit)
    .bind(description)
    .bind(id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn delete_item(pool: &PgPool, id: Uuid) -> Result<u64, sqlx::Error> {
    let res = sqlx::query("DELETE FROM inventory_items WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(res.rows_affected())
}

// ─── Issues Queries ──────────────────────────────────────────────────────────

pub async fn find_all_issues(pool: &PgPool) -> Result<Vec<InventoryIssueRow>, sqlx::Error> {
    sqlx::query_as::<_, InventoryIssueRow>(
        r#"
        SELECT iss.id, iss.item_id, iss.qty, iss.issued_to, iss.issue_date, iss.return_date,
               iss.status, iss.entry_by, iss.remarks, iss.created_at, iss.updated_at,
               item.name AS item_name, item.unit AS item_unit, u.name AS entry_by_name
        FROM inventory_issues iss
        JOIN inventory_items item ON item.id = iss.item_id
        LEFT JOIN users u ON u.id = iss.entry_by
        ORDER BY iss.issue_date DESC, iss.created_at DESC
        "#
    )
    .fetch_all(pool)
    .await
}

pub async fn find_raw_issue_by_id(pool: &PgPool, id: Uuid) -> Result<Option<RawIssueRow>, sqlx::Error> {
    sqlx::query_as::<_, RawIssueRow>(
        "SELECT id, item_id, qty, issued_to, issue_date, return_date, status, entry_by, remarks, created_at, updated_at FROM inventory_issues WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn insert_issue(
    pool: &PgPool,
    item_id: Uuid,
    qty: i32,
    issued_to: &str,
    issue_date: NaiveDate,
    entry_by: Option<Uuid>,
    remarks: Option<&str>,
) -> Result<RawIssueRow, sqlx::Error> {
    sqlx::query_as::<_, RawIssueRow>(
        r#"
        INSERT INTO inventory_issues (item_id, qty, issued_to, issue_date, entry_by, status, remarks)
        VALUES ($1, $2, $3, $4, $5, 'issued', $6)
        RETURNING id, item_id, qty, issued_to, issue_date, return_date, status, entry_by, remarks, created_at, updated_at
        "#
    )
    .bind(item_id)
    .bind(qty)
    .bind(issued_to)
    .bind(issue_date)
    .bind(entry_by)
    .bind(remarks)
    .fetch_one(pool)
    .await
}

pub async fn update_return_issue(
    pool: &PgPool,
    id: Uuid,
    return_date: NaiveDate,
    entry_by: Option<Uuid>,
) -> Result<RawIssueRow, sqlx::Error> {
    sqlx::query_as::<_, RawIssueRow>(
        r#"
        UPDATE inventory_issues
        SET status = 'returned', return_date = $2, entry_by = $3, updated_at = NOW()
        WHERE id = $1
        RETURNING id, item_id, qty, issued_to, issue_date, return_date, status, entry_by, remarks, created_at, updated_at
        "#
    )
    .bind(id)
    .bind(return_date)
    .bind(entry_by)
    .fetch_one(pool)
    .await
}

pub async fn adjust_item_available_qty(pool: &PgPool, item_id: Uuid, delta: i32) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE inventory_items SET available_qty = available_qty + $1, updated_at = NOW() WHERE id = $2")
        .bind(delta)
        .bind(item_id)
        .execute(pool)
        .await?;

    Ok(())
}
