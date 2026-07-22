// src/modules/inventory/handlers.rs
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use uuid::Uuid;
use crate::AppState;
use crate::errors::AppError;
use crate::middleware::AuthUser;
use crate::modules::auth::models::UserSubRole;
use super::models::*;

// ─── STATS & LOW STOCK ────────────────────────────────────

pub async fn get_stats(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize_sub_role(&[UserSubRole::InventoryManager])?;

    let row = sqlx::query_as::<_, StatsRow>(
        r#"
        SELECT
            (SELECT COUNT(*)::bigint FROM inventory_items) AS total_items,
            (SELECT COUNT(*)::bigint FROM inventory_categories) AS total_categories,
            (SELECT COUNT(*)::bigint FROM inventory_items WHERE available_qty <= low_stock_threshold) AS low_stock_items,
            (SELECT COALESCE(SUM(available_qty::float8 * unit_price::float8), 0.0) FROM inventory_items) AS total_value
        "#
    )
    .fetch_one(&state.db)
    .await?;

    Ok((StatusCode::OK, Json(json!({ "success": true, "data": row }))))
}

pub async fn get_low_stock(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize_sub_role(&[UserSubRole::InventoryManager])?;

    let items = sqlx::query_as::<_, LowStockItemRow>(
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
    .fetch_all(&state.db)
    .await?;

    Ok((StatusCode::OK, Json(json!({ "success": true, "data": items }))))
}

// ─── CATEGORIES ──────────────────────────────────────────

pub async fn get_categories(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize_sub_role(&[UserSubRole::InventoryManager])?;

    let categories = sqlx::query_as::<_, InventoryCategoryRow>(
        r#"
        SELECT c.id, c.name, c.description, c.created_at, c.updated_at,
               COUNT(i.id)::bigint AS total_items
        FROM inventory_categories c
        LEFT JOIN inventory_items i ON i.category_id = c.id
        GROUP BY c.id
        ORDER BY c.name ASC
        "#
    )
    .fetch_all(&state.db)
    .await?;

    Ok((StatusCode::OK, Json(json!({ "success": true, "data": categories }))))
}

pub async fn create_category(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(payload): Json<CreateCategoryPayload>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize_sub_role(&[UserSubRole::InventoryManager])?;

    if payload.name.trim().is_empty() {
        return Err(AppError::BadRequest("Category name is required".to_string()));
    }

    let existing = sqlx::query_scalar::<_, Uuid>("SELECT id FROM inventory_categories WHERE name = $1")
        .bind(payload.name.trim())
        .fetch_optional(&state.db)
        .await?;

    if existing.is_some() {
        return Err(AppError::Conflict("Category already exists".to_string()));
    }

    let cat = sqlx::query_as::<_, CategorySimpleRow>(
        r#"
        INSERT INTO inventory_categories (name, description)
        VALUES ($1, $2)
        RETURNING id, name, description, created_at, updated_at
        "#
    )
    .bind(payload.name.trim())
    .bind(payload.description.as_deref())
    .fetch_one(&state.db)
    .await?;

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "success": true,
            "data": {
                "id": cat.id,
                "name": cat.name,
                "description": cat.description,
                "created_at": cat.created_at,
                "updated_at": cat.updated_at,
                "total_items": 0
            }
        })),
    ))
}

pub async fn edit_category(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateCategoryPayload>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize_sub_role(&[UserSubRole::InventoryManager])?;

    let existing = sqlx::query_as::<_, CategorySimpleRow>("SELECT id, name, description, created_at, updated_at FROM inventory_categories WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Category not found".to_string()))?;

    let name = payload.name.unwrap_or(existing.name);
    let description = payload.description.or(existing.description);

    let cat = sqlx::query_as::<_, CategorySimpleRow>(
        r#"
        UPDATE inventory_categories
        SET name = $2, description = $3, updated_at = now()
        WHERE id = $1
        RETURNING id, name, description, created_at, updated_at
        "#
    )
    .bind(id)
    .bind(name)
    .bind(description.as_deref())
    .fetch_one(&state.db)
    .await?;

    Ok((
        StatusCode::OK,
        Json(json!({
            "success": true,
            "data": cat
        })),
    ))
}

pub async fn remove_category(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize_sub_role(&[UserSubRole::InventoryManager])?;

    let item_check = sqlx::query_scalar::<_, Uuid>("SELECT id FROM inventory_items WHERE category_id = $1 LIMIT 1")
        .bind(id)
        .fetch_optional(&state.db)
        .await?;

    if item_check.is_some() {
        return Err(AppError::BadRequest("Cannot delete category with existing items".to_string()));
    }

    let result = sqlx::query_scalar::<_, Uuid>("DELETE FROM inventory_categories WHERE id = $1 RETURNING id")
        .bind(id)
        .fetch_optional(&state.db)
        .await?;

    if result.is_none() {
        return Err(AppError::NotFound("Category not found".to_string()));
    }

    Ok((StatusCode::OK, Json(json!({ "success": true, "message": "Category deleted successfully" }))))
}

// ─── ITEMS ───────────────────────────────────────────────

pub async fn get_items(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Query(q): Query<GetItemsQuery>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize_sub_role(&[UserSubRole::InventoryManager])?;

    let page = q.page.unwrap_or(1).max(1);
    let limit = q.limit.unwrap_or(50).max(1);
    let offset = (page - 1) * limit;

    let items = sqlx::query_as::<_, InventoryItemRow>(
        r#"
        SELECT i.id, i.category_id, i.name, i.required_qty, i.available_qty, 
               i.unit_price::float8 AS unit_price, i.low_stock_threshold, i.unit, 
               i.description, i.created_by, i.created_at, i.updated_at,
               c.name AS category_name, u.name AS entry_by_name,
               (i.available_qty <= i.low_stock_threshold) AS is_low_stock,
               (i.available_qty::float8 * i.unit_price::float8) AS total_value,
               COUNT(*) OVER()::int AS total_count
        FROM inventory_items i
        JOIN inventory_categories c ON c.id = i.category_id
        LEFT JOIN users u ON u.id = i.created_by
        WHERE ($1::text IS NULL OR i.name ILIKE $1)
          AND ($2::uuid IS NULL OR i.category_id = $2)
        ORDER BY i.created_at DESC
        LIMIT $3 OFFSET $4
        "#
    )
    .bind(q.search.as_ref().map(|s| format!("%{}%", s)))
    .bind(q.category_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.db)
    .await?;

    let total_count = items.first().map(|r| r.total_count).unwrap_or(0);
    let total_pages = (total_count as f64 / limit as f64).ceil() as i64;

    Ok((
        StatusCode::OK,
        Json(json!({
            "status": "success",
            "success": true,
            "data": items,
            "pagination": {
                "total": total_count,
                "page": page,
                "limit": limit,
                "pages": total_pages
            }
        })),
    ))
}

pub async fn get_item(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize_sub_role(&[UserSubRole::InventoryManager])?;

    let row = sqlx::query_as::<_, SingleItemRow>(
        r#"
        SELECT i.id, i.category_id, i.name, i.required_qty, i.available_qty, 
               i.unit_price::float8 AS unit_price, i.low_stock_threshold, i.unit, 
               i.description, i.created_by, i.created_at, i.updated_at,
               c.name AS category_name, u.name AS entry_by_name,
               (i.available_qty <= i.low_stock_threshold) AS is_low_stock,
               (i.available_qty::float8 * i.unit_price::float8) AS total_value
        FROM inventory_items i
        JOIN inventory_categories c ON c.id = i.category_id
        LEFT JOIN users u ON u.id = i.created_by
        WHERE i.id = $1
        "#
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Item not found".to_string()))?;

    Ok((
        StatusCode::OK,
        Json(json!({
            "success": true,
            "data": row
        })),
    ))
}

pub async fn create_item(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(payload): Json<CreateItemPayload>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize_sub_role(&[UserSubRole::InventoryManager])?;

    if payload.name.trim().is_empty() {
        return Err(AppError::BadRequest("Item name is required".to_string()));
    }

    let cat_exists = sqlx::query_scalar::<_, Uuid>("SELECT id FROM inventory_categories WHERE id = $1")
        .bind(payload.category_id)
        .fetch_optional(&state.db)
        .await?;

    if cat_exists.is_none() {
        return Err(AppError::NotFound("Category not found".to_string()));
    }

    let unit = payload.unit.unwrap_or_else(|| "pcs".to_string());
    let unit_price = payload.unit_price.unwrap_or(0.0);

    let item = sqlx::query_as::<_, RawItemRow>(
        r#"
        INSERT INTO inventory_items (category_id, name, required_qty, available_qty, unit_price, low_stock_threshold, unit, description, created_by)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING id, category_id, name, required_qty, available_qty, unit_price::float8 AS unit_price, low_stock_threshold, unit, description, created_by, created_at, updated_at
        "#
    )
    .bind(payload.category_id)
    .bind(payload.name.trim())
    .bind(payload.required_qty.unwrap_or(0))
    .bind(payload.available_qty.unwrap_or(0))
    .bind(unit_price)
    .bind(payload.low_stock_threshold.unwrap_or(0))
    .bind(unit)
    .bind(payload.description.as_deref())
    .bind(auth_user.id)
    .fetch_one(&state.db)
    .await?;

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "success": true,
            "data": item
        })),
    ))
}

pub async fn edit_item(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateItemPayload>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize_sub_role(&[UserSubRole::InventoryManager])?;

    let existing = sqlx::query_as::<_, RawItemRow>(
        "SELECT id, category_id, name, required_qty, available_qty, unit_price::float8 AS unit_price, low_stock_threshold, unit, description, created_by, created_at, updated_at FROM inventory_items WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Item not found".to_string()))?;

    let category_id = payload.category_id.unwrap_or(existing.category_id);
    let name = payload.name.unwrap_or(existing.name);
    let required_qty = payload.required_qty.unwrap_or(existing.required_qty);
    let available_qty = payload.available_qty.unwrap_or(existing.available_qty);
    let unit_price = payload.unit_price.unwrap_or(existing.unit_price);
    let low_stock_threshold = payload.low_stock_threshold.unwrap_or(existing.low_stock_threshold);
    let unit = payload.unit.unwrap_or(existing.unit);
    let description = payload.description.or(existing.description);

    let updated = sqlx::query_as::<_, RawItemRow>(
        r#"
        UPDATE inventory_items
        SET category_id = $2, name = $3, required_qty = $4, available_qty = $5, unit_price = $6, low_stock_threshold = $7, unit = $8, description = $9, updated_at = now()
        WHERE id = $1
        RETURNING id, category_id, name, required_qty, available_qty, unit_price::float8 AS unit_price, low_stock_threshold, unit, description, created_by, created_at, updated_at
        "#
    )
    .bind(id)
    .bind(category_id)
    .bind(name)
    .bind(required_qty)
    .bind(available_qty)
    .bind(unit_price)
    .bind(low_stock_threshold)
    .bind(unit)
    .bind(description.as_deref())
    .fetch_one(&state.db)
    .await?;

    Ok((
        StatusCode::OK,
        Json(json!({
            "success": true,
            "data": updated
        })),
    ))
}

pub async fn remove_item(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize_sub_role(&[UserSubRole::InventoryManager])?;

    let result = sqlx::query_scalar::<_, Uuid>("DELETE FROM inventory_items WHERE id = $1 RETURNING id")
        .bind(id)
        .fetch_optional(&state.db)
        .await?;

    if result.is_none() {
        return Err(AppError::NotFound("Item not found".to_string()));
    }

    Ok((StatusCode::OK, Json(json!({ "success": true, "message": "Item deleted successfully" }))))
}

pub async fn import_items(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(payload): Json<ImportItemsPayload>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize_sub_role(&[UserSubRole::InventoryManager])?;

    let mut results = Vec::new();

    for item in payload.items {
        let mut category_id = item.category_id;

        if category_id.is_none() {
            if let Some(ref cat_name) = item.category_name {
                let existing_cat = sqlx::query_scalar::<_, Uuid>("SELECT id FROM inventory_categories WHERE name ILIKE $1")
                    .bind(cat_name)
                    .fetch_optional(&state.db)
                    .await?;

                if let Some(cat_id_found) = existing_cat {
                    category_id = Some(cat_id_found);
                } else {
                    let new_cat_id = sqlx::query_scalar::<_, Uuid>("INSERT INTO inventory_categories (name) VALUES ($1) RETURNING id")
                        .bind(cat_name)
                        .fetch_one(&state.db)
                        .await?;
                    category_id = Some(new_cat_id);
                }
            }
        }

        let cat_id = match category_id {
            Some(id) => id,
            None => {
                let first_cat = sqlx::query_scalar::<_, Uuid>("SELECT id FROM inventory_categories LIMIT 1")
                    .fetch_optional(&state.db)
                    .await?;
                match first_cat {
                    Some(id) => id,
                    None => return Err(AppError::BadRequest("No categories available to attach imported item".to_string())),
                }
            }
        };

        let existing_item = sqlx::query_as::<_, RawItemRow>(
            "SELECT id, category_id, name, required_qty, available_qty, unit_price::float8 AS unit_price, low_stock_threshold, unit, description, created_by, created_at, updated_at FROM inventory_items WHERE name = $1 AND category_id = $2"
        )
        .bind(item.name.trim())
        .bind(cat_id)
        .fetch_optional(&state.db)
        .await?;

        if let Some(existing) = existing_item {
            let req_qty = item.required_qty.unwrap_or(existing.required_qty);
            let avail_qty = item.available_qty.unwrap_or(existing.available_qty);
            let price = item.unit_price.unwrap_or(existing.unit_price);
            let unit = item.unit.unwrap_or(existing.unit);
            let desc = item.description.or(existing.description);

            let updated = sqlx::query_as::<_, RawItemRow>(
                r#"
                UPDATE inventory_items
                SET required_qty = $1, available_qty = $2, unit_price = $3, unit = $4, description = $5, updated_at = now()
                WHERE id = $6
                RETURNING id, category_id, name, required_qty, available_qty, unit_price::float8 AS unit_price, low_stock_threshold, unit, description, created_by, created_at, updated_at
                "#
            )
            .bind(req_qty)
            .bind(avail_qty)
            .bind(price)
            .bind(unit)
            .bind(desc.as_deref())
            .bind(existing.id)
            .fetch_one(&state.db)
            .await?;

            results.push(json!({
                "id": updated.id,
                "category_id": updated.category_id,
                "name": updated.name,
                "required_qty": updated.required_qty,
                "available_qty": updated.available_qty,
                "unit_price": updated.unit_price,
                "unit": updated.unit,
                "description": updated.description,
                "_changeStatus": "modified"
            }));
        } else {
            let req_qty = item.required_qty.unwrap_or(0);
            let avail_qty = item.available_qty.unwrap_or(0);
            let price = item.unit_price.unwrap_or(0.0);
            let unit = item.unit.unwrap_or_else(|| "pcs".to_string());

            let created = sqlx::query_as::<_, RawItemRow>(
                r#"
                INSERT INTO inventory_items (category_id, name, required_qty, available_qty, unit_price, unit, description, created_by)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                RETURNING id, category_id, name, required_qty, available_qty, unit_price::float8 AS unit_price, low_stock_threshold, unit, description, created_by, created_at, updated_at
                "#
            )
            .bind(cat_id)
            .bind(item.name.trim())
            .bind(req_qty)
            .bind(avail_qty)
            .bind(price)
            .bind(unit)
            .bind(item.description.as_deref())
            .bind(auth_user.id)
            .fetch_one(&state.db)
            .await?;

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

    Ok((StatusCode::OK, Json(json!({ "success": true, "data": results }))))
}

// ─── ISSUES & RETURNS ────────────────────────────────────

pub async fn get_issues(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize_sub_role(&[UserSubRole::InventoryManager])?;

    let issues = sqlx::query_as::<_, InventoryIssueRow>(
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
    .fetch_all(&state.db)
    .await?;

    Ok((StatusCode::OK, Json(json!({ "success": true, "data": issues }))))
}

pub async fn issue_item(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(payload): Json<IssueItemPayload>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize_sub_role(&[UserSubRole::InventoryManager])?;

    if payload.qty <= 0 {
        return Err(AppError::BadRequest("Quantity must be greater than zero".to_string()));
    }

    let item = sqlx::query_as::<_, (i32, String)>("SELECT available_qty, unit FROM inventory_items WHERE id = $1")
        .bind(payload.item_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Item not found".to_string()))?;

    if item.0 < payload.qty {
        return Err(AppError::BadRequest(format!(
            "Insufficient stock. Only {} {} available.",
            item.0, item.1
        )));
    }

    // Deduct stock
    sqlx::query("UPDATE inventory_items SET available_qty = available_qty - $1, updated_at = now() WHERE id = $2")
        .bind(payload.qty)
        .bind(payload.item_id)
        .execute(&state.db)
        .await?;

    // Insert issue record
    let issue_date = payload.issue_date.unwrap_or_else(|| chrono::Utc::now().date_naive());

    let issue = sqlx::query_as::<_, RawIssueRow>(
        r#"
        INSERT INTO inventory_issues (item_id, qty, issued_to, issue_date, entry_by, status, remarks)
        VALUES ($1, $2, $3, $4, $5, 'issued', $6)
        RETURNING id, item_id, qty, issued_to, issue_date, return_date, status, entry_by, remarks, created_at, updated_at
        "#
    )
    .bind(payload.item_id)
    .bind(payload.qty)
    .bind(payload.issued_to.trim())
    .bind(issue_date)
    .bind(auth_user.id)
    .bind(payload.remarks.as_deref())
    .fetch_one(&state.db)
    .await?;

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "success": true,
            "data": issue
        })),
    ))
}

pub async fn return_item(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(payload): Json<ReturnItemPayload>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize_sub_role(&[UserSubRole::InventoryManager])?;

    let issue = sqlx::query_as::<_, RawIssueRow>("SELECT id, item_id, qty, issued_to, issue_date, return_date, status, entry_by, remarks, created_at, updated_at FROM inventory_issues WHERE id = $1")
        .bind(payload.id)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Issue record not found".to_string()))?;

    if issue.status == "returned" {
        return Err(AppError::BadRequest("Item already returned".to_string()));
    }

    // Return stock to item
    sqlx::query("UPDATE inventory_items SET available_qty = available_qty + $1, updated_at = now() WHERE id = $2")
        .bind(issue.qty)
        .bind(issue.item_id)
        .execute(&state.db)
        .await?;

    // Update issue record
    let return_date = payload.return_date.unwrap_or_else(|| chrono::Utc::now().date_naive());

    let updated = sqlx::query_as::<_, RawIssueRow>(
        r#"
        UPDATE inventory_issues
        SET status = 'returned', return_date = $2, entry_by = $3, updated_at = now()
        WHERE id = $1
        RETURNING id, item_id, qty, issued_to, issue_date, return_date, status, entry_by, remarks, created_at, updated_at
        "#
    )
    .bind(payload.id)
    .bind(return_date)
    .bind(auth_user.id)
    .fetch_one(&state.db)
    .await?;

    Ok((
        StatusCode::OK,
        Json(json!({
            "success": true,
            "data": updated
        })),
    ))
}

pub async fn import_issues(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(payload): Json<ImportIssuesPayload>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize_sub_role(&[UserSubRole::InventoryManager])?;

    let mut results = Vec::new();

    for record in payload.issues {
        let item_id = sqlx::query_scalar::<_, Uuid>("SELECT id FROM inventory_items WHERE name ILIKE $1")
            .bind(record.item_name.trim())
            .fetch_optional(&state.db)
            .await?
            .ok_or_else(|| AppError::BadRequest(format!("Item '{}' not found in inventory. Please add it first.", record.item_name)))?;

        let issue_date = record.issue_date.unwrap_or_else(|| chrono::Utc::now().date_naive());
        let status = record.status.unwrap_or_else(|| {
            if record.return_date.is_some() {
                "returned".to_string()
            } else {
                "issued".to_string()
            }
        });

        let issue = sqlx::query_as::<_, RawIssueRow>(
            r#"
            INSERT INTO inventory_issues (item_id, qty, issued_to, issue_date, return_date, status, entry_by, remarks)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, item_id, qty, issued_to, issue_date, return_date, status, entry_by, remarks, created_at, updated_at
            "#
        )
        .bind(item_id)
        .bind(record.qty.unwrap_or(1))
        .bind(record.issued_to.trim())
        .bind(issue_date)
        .bind(record.return_date)
        .bind(status)
        .bind(auth_user.id)
        .bind(record.remarks.as_deref())
        .fetch_one(&state.db)
        .await?;

        results.push(issue);
    }

    Ok((StatusCode::OK, Json(json!({ "success": true, "data": results }))))
}
