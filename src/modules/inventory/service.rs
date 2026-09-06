// src/modules/inventory/service.rs
//! Inventory Business Logic & Domain Service Layer

use chrono::NaiveDate;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;
use crate::errors::AppError;
use super::models::{
    CategorySimpleRow, CreateCategoryPayload, CreateItemPayload, GetItemsQuery,
    ImportIssueRow, ImportItemRow, InventoryCategoryRow, InventoryIssueRow,
    InventoryItemRow, IssueItemPayload, LowStockItemRow, RawIssueRow,
    RawItemRow, ReturnItemPayload, SingleItemRow, StatsRow,
    UpdateCategoryPayload, UpdateItemPayload,
};
use super::repository;

// ─── Stats & Low Stock ───────────────────────────────────────────────────────

pub async fn get_stats(pool: &PgPool) -> Result<StatsRow, AppError> {
    repository::find_stats(pool)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to fetch inventory stats: {}", e)))
}

pub async fn get_low_stock(pool: &PgPool) -> Result<Vec<LowStockItemRow>, AppError> {
    repository::find_low_stock(pool)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to fetch low stock items: {}", e)))
}

// ─── Categories ──────────────────────────────────────────────────────────────

pub async fn list_categories(pool: &PgPool) -> Result<Vec<InventoryCategoryRow>, AppError> {
    repository::find_all_categories(pool)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to fetch categories: {}", e)))
}

pub async fn create_category(pool: &PgPool, payload: CreateCategoryPayload) -> Result<CategorySimpleRow, AppError> {
    if payload.name.trim().is_empty() {
        return Err(AppError::BadRequest("Category name is required".to_string()));
    }

    let existing = repository::find_category_by_name(pool, payload.name.trim())
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    if existing.is_some() {
        return Err(AppError::Conflict("Category with this name already exists".to_string()));
    }

    repository::create_category(pool, payload.name.trim(), payload.description.as_deref())
        .await
        .map_err(|e| AppError::Internal(format!("Failed to create category: {}", e)))
}

pub async fn update_category(
    pool: &PgPool,
    id: Uuid,
    payload: UpdateCategoryPayload,
) -> Result<CategorySimpleRow, AppError> {
    let name = payload.name.unwrap_or_default();
    if name.trim().is_empty() {
        return Err(AppError::BadRequest("Category name cannot be empty".to_string()));
    }

    repository::update_category(pool, id, name.trim(), payload.description.as_deref())
        .await
        .map_err(|e| AppError::Internal(format!("Failed to update category: {}", e)))
}

pub async fn delete_category(pool: &PgPool, id: Uuid) -> Result<(), AppError> {
    let affected = repository::delete_category(pool, id)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to delete category: {}", e)))?;

    if affected == 0 {
        return Err(AppError::NotFound("Category not found".to_string()));
    }

    Ok(())
}

// ─── Items ───────────────────────────────────────────────────────────────────

pub async fn list_items(
    pool: &PgPool,
    q: &GetItemsQuery,
) -> Result<(Vec<InventoryItemRow>, i32, i64, i64), AppError> {
    let page = q.page.unwrap_or(1).max(1);
    let limit = q.limit.unwrap_or(50).max(1);
    let offset = (page - 1) * limit;

    let mut sql = r#"
        SELECT i.id, i.category_id, i.name, i.required_qty, i.available_qty, 
               i.unit_price::float8 AS unit_price, i.low_stock_threshold, i.unit, 
               i.description, i.created_by, i.created_at, i.updated_at,
               c.name AS category_name,
               u.name AS entry_by_name,
               (i.available_qty <= i.low_stock_threshold) AS is_low_stock,
               (i.available_qty::float8 * i.unit_price::float8) AS total_value,
               COUNT(*) OVER()::int AS total_count
        FROM inventory_items i
        JOIN inventory_categories c ON c.id = i.category_id
        LEFT JOIN users u ON u.id = i.created_by
        WHERE 1=1
    "#
    .to_string();

    let mut binders = Vec::new();
    let mut idx = 1usize;

    if let Some(ref search) = q.search {
        let term = format!("%{}%", search.trim());
        sql.push_str(&format!(" AND (i.name ILIKE ${idx} OR c.name ILIKE ${idx} OR i.description ILIKE ${idx})"));
        binders.push(term);
        idx += 1;
    }

    if let Some(cat_id) = q.category_id {
        sql.push_str(&format!(" AND i.category_id = ${idx}"));
        binders.push(cat_id.to_string());
        idx += 1;
    }

    sql.push_str(" ORDER BY i.name ASC");
    sql.push_str(&format!(" LIMIT ${idx} OFFSET ${}", idx + 1));

    let items = repository::find_items(pool, &sql, binders, limit, offset)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to fetch items: {}", e)))?;

    let total = if items.is_empty() { 0 } else { items[0].total_count };

    Ok((items, total, page, limit))
}

pub async fn get_item(pool: &PgPool, id: Uuid) -> Result<SingleItemRow, AppError> {
    repository::find_item_by_id(pool, id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Inventory item not found".to_string()))
}

pub async fn create_item(
    pool: &PgPool,
    actor_id: Uuid,
    payload: CreateItemPayload,
) -> Result<SingleItemRow, AppError> {
    if payload.name.trim().is_empty() {
        return Err(AppError::BadRequest("Item name is required".to_string()));
    }

    let required_qty = payload.required_qty.unwrap_or(0);
    let available_qty = payload.available_qty.unwrap_or(0);
    if available_qty < 0 {
        return Err(AppError::BadRequest("Available quantity cannot be negative".to_string()));
    }

    let unit_price = payload.unit_price.unwrap_or(0.0);
    let low_stock_threshold = payload.low_stock_threshold.unwrap_or(5);
    let unit = payload.unit.as_deref().unwrap_or("pcs");

    repository::insert_item(
        pool,
        payload.category_id,
        payload.name.trim(),
        required_qty,
        available_qty,
        unit_price,
        low_stock_threshold,
        unit,
        payload.description.as_deref(),
        Some(actor_id),
    )
    .await
    .map_err(|e| AppError::Internal(format!("Failed to create item: {}", e)))
}

pub async fn update_item(
    pool: &PgPool,
    id: Uuid,
    payload: UpdateItemPayload,
) -> Result<SingleItemRow, AppError> {
    let existing = repository::find_raw_item_by_id(pool, id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Inventory item not found".to_string()))?;

    let category_id = payload.category_id.unwrap_or(existing.category_id);
    let name = payload.name.unwrap_or(existing.name);
    let required_qty = payload.required_qty.unwrap_or(existing.required_qty);
    let available_qty = payload.available_qty.unwrap_or(existing.available_qty);
    let unit_price = payload.unit_price.unwrap_or(existing.unit_price);
    let low_stock_threshold = payload.low_stock_threshold.unwrap_or(existing.low_stock_threshold);
    let unit = payload.unit.unwrap_or(existing.unit);
    let description = payload.description.or(existing.description);

    repository::update_item(
        pool,
        id,
        category_id,
        name.trim(),
        required_qty,
        available_qty,
        unit_price,
        low_stock_threshold,
        &unit,
        description.as_deref(),
    )
    .await
    .map_err(|e| AppError::Internal(format!("Failed to update item: {}", e)))?;

    get_item(pool, id).await
}

pub async fn delete_item(pool: &PgPool, id: Uuid) -> Result<(), AppError> {
    let affected = repository::delete_item(pool, id)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to delete item: {}", e)))?;

    if affected == 0 {
        return Err(AppError::NotFound("Inventory item not found".to_string()));
    }

    Ok(())
}

pub async fn import_items(
    pool: &PgPool,
    actor_id: Uuid,
    items: Vec<ImportItemRow>,
) -> Result<Vec<serde_json::Value>, AppError> {
    let mut results = Vec::new();

    for item in items {
        let cat_id = if let Some(id) = item.category_id {
            id
        } else if let Some(ref cname) = item.category_name {
            let cat_row = repository::find_category_by_name(pool, cname)
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?;

            match cat_row {
                Some(id) => id,
                None => {
                    let new_cat = repository::create_category(pool, cname.trim(), None)
                        .await
                        .map_err(|e| AppError::Internal(e.to_string()))?;
                    new_cat.id
                }
            }
        } else {
            return Err(AppError::BadRequest(format!("Category is required for item '{}'", item.name)));
        };

        let existing_id = repository::find_item_id_by_name(pool, item.name.trim())
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        if let Some(eid) = existing_id {
            let existing = repository::find_raw_item_by_id(pool, eid)
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?
                .ok_or_else(|| AppError::NotFound("Item not found".to_string()))?;

            let req_qty = item.required_qty.unwrap_or(existing.required_qty);
            let avail_qty = item.available_qty.unwrap_or(existing.available_qty);
            let price = item.unit_price.unwrap_or(existing.unit_price);
            let unit = item.unit.unwrap_or(existing.unit);
            let desc = item.description.or(existing.description);

            repository::update_item(
                pool,
                eid,
                cat_id,
                item.name.trim(),
                req_qty,
                avail_qty,
                price,
                existing.low_stock_threshold,
                &unit,
                desc.as_deref(),
            )
            .await
            .map_err(|e| AppError::Internal(format!("Failed to update item: {}", e)))?;

            results.push(json!({
                "id": eid,
                "category_id": cat_id,
                "name": item.name.trim(),
                "required_qty": req_qty,
                "available_qty": avail_qty,
                "unit_price": price,
                "unit": unit,
                "description": desc,
                "_changeStatus": "modified"
            }));
        } else {
            let req_qty = item.required_qty.unwrap_or(0);
            let avail_qty = item.available_qty.unwrap_or(0);
            let price = item.unit_price.unwrap_or(0.0);
            let unit = item.unit.unwrap_or_else(|| "pcs".to_string());

            let created = repository::insert_item(
                pool,
                cat_id,
                item.name.trim(),
                req_qty,
                avail_qty,
                price,
                5,
                &unit,
                item.description.as_deref(),
                Some(actor_id),
            )
            .await
            .map_err(|e| AppError::Internal(format!("Failed to create item: {}", e)))?;

            results.push(json!({
                "id": created.id,
                "category_id": created.category_id,
                "name": created.name,
                "required_qty": created.required_qty,
                "available_qty": created.available_qty,
                "unit_price": created.unit_price,
                "unit": created.unit,
                "description": created.description,
                "_changeStatus": "new"
            }));
        }
    }

    Ok(results)
}

// ─── Issues ──────────────────────────────────────────────────────────────────

pub async fn list_issues(pool: &PgPool) -> Result<Vec<InventoryIssueRow>, AppError> {
    repository::find_all_issues(pool)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to fetch issues: {}", e)))
}

pub async fn issue_item(
    pool: &PgPool,
    actor_id: Uuid,
    payload: IssueItemPayload,
) -> Result<RawIssueRow, AppError> {
    if payload.qty <= 0 {
        return Err(AppError::BadRequest("Quantity must be greater than zero".to_string()));
    }

    let item = repository::find_raw_item_by_id(pool, payload.item_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Item not found".to_string()))?;

    if item.available_qty < payload.qty {
        return Err(AppError::BadRequest(format!(
            "Insufficient stock. Only {} {} available.",
            item.available_qty, item.unit
        )));
    }

    repository::adjust_item_available_qty(pool, payload.item_id, -payload.qty)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let issue_date = payload.issue_date.unwrap_or_else(|| chrono::Utc::now().date_naive());

    repository::insert_issue(
        pool,
        payload.item_id,
        payload.qty,
        payload.issued_to.trim(),
        issue_date,
        Some(actor_id),
        payload.remarks.as_deref(),
    )
    .await
    .map_err(|e| AppError::Internal(format!("Failed to create issue: {}", e)))
}

pub async fn return_item(
    pool: &PgPool,
    actor_id: Uuid,
    payload: ReturnItemPayload,
) -> Result<RawIssueRow, AppError> {
    let issue = repository::find_raw_issue_by_id(pool, payload.id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Issue record not found".to_string()))?;

    if issue.status == "returned" {
        return Err(AppError::BadRequest("Item already returned".to_string()));
    }

    repository::adjust_item_available_qty(pool, issue.item_id, issue.qty)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let return_date = payload.return_date.unwrap_or_else(|| chrono::Utc::now().date_naive());

    repository::update_return_issue(pool, payload.id, return_date, Some(actor_id))
        .await
        .map_err(|e| AppError::Internal(format!("Failed to return item: {}", e)))
}

pub async fn import_issues(
    pool: &PgPool,
    actor_id: Uuid,
    issues: Vec<ImportIssueRow>,
) -> Result<Vec<RawIssueRow>, AppError> {
    let mut results = Vec::new();

    for record in issues {
        let item_id = repository::find_item_id_by_name(pool, record.item_name.trim())
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?
            .ok_or_else(|| AppError::BadRequest(format!(
                "Item '{}' not found in inventory. Please add it first.",
                record.item_name
            )))?;

        let issue_date = record.issue_date.unwrap_or_else(|| chrono::Utc::now().date_naive());
        let issue = repository::insert_issue(
            pool,
            item_id,
            record.qty.unwrap_or(1),
            record.issued_to.trim(),
            issue_date,
            Some(actor_id),
            record.remarks.as_deref(),
        )
        .await
        .map_err(|e| AppError::Internal(format!("Failed to import issue: {}", e)))?;

        results.push(issue);
    }

    Ok(results)
}
