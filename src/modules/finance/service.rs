// backend-rust/src/modules/finance/service.rs
//! Finance Business Logic & Domain Service Layer

use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;
use crate::errors::AppError;
use crate::utils::activity::log_audit;
use super::models::{
    AddExpensePayload, AddFeeRecordPayload, Expense, ExpenseWithCount,
    FeeRecord, FeeRecordWithDetails, GetExpensesQuery, UpdateExpensePayload,
    UpdateFeeRecordPayload,
};
use super::{hostel, repository, transport, tuition};

// ─── Fee Collections Services ─────────────────────────────────────────────────

pub async fn get_fee_record(pool: &PgPool, id: Uuid) -> Result<FeeRecordWithDetails, AppError> {
    repository::find_fee_record_by_id(pool, id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Fee record not found".to_string()))
}

pub async fn create_fee_record(
    pool: &PgPool,
    fee_type: &str,
    payload: AddFeeRecordPayload,
) -> Result<FeeRecord, AppError> {
    match fee_type {
        "transport" => transport::create_record(pool, payload).await,
        "hostel" => hostel::create_record(pool, payload).await,
        _ => tuition::create_record(pool, fee_type, payload).await,
    }
}

pub async fn update_fee_record(
    pool: &PgPool,
    id_or_type: &str,
    payload: UpdateFeeRecordPayload,
) -> Result<FeeRecord, AppError> {
    // If id_or_type is a valid UUID, look up fee_type from DB
    if let Ok(id) = id_or_type.parse::<Uuid>() {
        let record = repository::find_fee_record_by_id(pool, id)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?
            .ok_or_else(|| AppError::NotFound("Fee record not found".to_string()))?;

        match record.fee_type.as_str() {
            "transport" => transport::update_record(pool, id, payload).await,
            "hostel" => hostel::update_record(pool, id, payload).await,
            _ => tuition::update_record(pool, id, payload).await,
        }
    } else {
        // Fallback matching by fee_type string
        match id_or_type {
            "transport" => {
                let id = payload.id.ok_or_else(|| AppError::BadRequest("ID is required".to_string()))?;
                transport::update_record(pool, id, payload).await
            }
            "hostel" => {
                let id = payload.id.ok_or_else(|| AppError::BadRequest("ID is required".to_string()))?;
                hostel::update_record(pool, id, payload).await
            }
            _ => {
                let id = payload.id.ok_or_else(|| AppError::BadRequest("ID is required".to_string()))?;
                tuition::update_record(pool, id, payload).await
            }
        }
    }
}

pub async fn delete_fee_record(pool: &PgPool, id: Uuid) -> Result<FeeRecord, AppError> {
    repository::delete_fee_record(pool, id)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to delete fee record: {}", e)))?
        .ok_or_else(|| AppError::NotFound("Fee record not found".to_string()))
}

// ─── Expenses Services ────────────────────────────────────────────────────────

pub async fn list_expenses(
    pool: &PgPool,
    q: &GetExpensesQuery,
) -> Result<(Vec<ExpenseWithCount>, i32, i64, i64), AppError> {
    let page = q.page.unwrap_or(1).max(1);
    let limit = q.limit.unwrap_or(50).max(1);
    let offset = (page - 1) * limit;

    let mut sql = "SELECT id, ref_no, description, amount::float8 AS amount, category, date, payment_mode, remarks, utr, receipt, party_name, spent_by, voucher_no, created_at, COUNT(*) OVER()::int AS total_count FROM expenses WHERE 1=1".to_string();
    let mut binders = Vec::new();
    let mut idx = 1usize;

    if let Some(ref search) = q.search {
        let term = format!("%{}%", search.trim());
        sql.push_str(&format!(" AND (description ILIKE ${idx} OR party_name ILIKE ${idx} OR remarks ILIKE ${idx} OR voucher_no ILIKE ${idx})"));
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

    let start_d = q.start_date.as_ref().or(q.from_date.as_ref());
    if let Some(start_date) = start_d {
        if let Ok(parsed) = NaiveDate::parse_from_str(start_date, "%Y-%m-%d") {
            sql.push_str(&format!(" AND date >= ${idx}"));
            binders.push(parsed.to_string());
            idx += 1;
        }
    }

    let end_d = q.end_date.as_ref().or(q.to_date.as_ref());
    if let Some(end_date) = end_d {
        if let Ok(parsed) = NaiveDate::parse_from_str(end_date, "%Y-%m-%d") {
            sql.push_str(&format!(" AND date <= ${idx}"));
            binders.push(parsed.to_string());
            idx += 1;
        }
    }

    sql.push_str(" ORDER BY date DESC, created_at DESC");
    sql.push_str(&format!(" LIMIT ${idx} OFFSET ${}", idx + 1));

    let list = repository::find_expenses(pool, &sql, binders, limit, offset)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to fetch expenses: {}", e)))?;

    let total = if list.is_empty() { 0 } else { list[0].total_count };

    Ok((list, total, page, limit))
}

pub async fn get_expense(pool: &PgPool, id: Uuid) -> Result<Expense, AppError> {
    repository::find_expense_by_id(pool, id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Expense record not found".to_string()))
}

pub async fn create_expense(
    pool: &PgPool,
    user_role_str: &str,
    payload: AddExpensePayload,
) -> Result<Expense, AppError> {
    payload.validate()?;

    let parsed_date = match payload.date.as_deref() {
        Some(d) if !d.is_empty() => NaiveDate::parse_from_str(d, "%Y-%m-%d")
            .map_err(|_| AppError::BadRequest("Invalid date format".to_string()))?,
        _ => chrono::Utc::now().date_naive(),
    };

    let ref_no = format!("EXP-{}", &Uuid::new_v4().to_string()[..8].to_uppercase());
    let payment_mode = payload.payment_mode.as_deref().unwrap_or("Online");
    let category = payload.category.as_deref().unwrap_or("general");

    let expense = repository::insert_expense(
        pool,
        &ref_no,
        payload.description.trim(),
        payload.amount,
        category,
        parsed_date,
        payment_mode,
        payload.remarks.as_deref(),
        payload.utr.as_deref(),
        payload.receipt.as_deref(),
        payload.party_name.as_deref(),
        payload.spent_by.as_deref(),
        payload.voucher_no.as_deref(),
    )
    .await
    .map_err(|e| AppError::Internal(format!("Failed to create expense: {}", e)))?;

    let _ = log_audit(
        pool,
        "Staff User",
        user_role_str,
        "EXPENSE_RECORDED",
        "finance",
        &format!("Category {} - amount ₹{}", expense.category, expense.amount),
    )
    .await;

    Ok(expense)
}

pub async fn update_expense(
    pool: &PgPool,
    id: Uuid,
    payload: UpdateExpensePayload,
) -> Result<Expense, AppError> {
    payload.validate()?;

    let existing = repository::find_expense_by_id(pool, id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Expense record not found".to_string()))?;

    let mut updated = existing;
    if let Some(desc) = payload.description { updated.description = desc; }
    if let Some(amount) = payload.amount { updated.amount = amount; }
    if let Some(cat) = payload.category { updated.category = cat; }
    if let Some(ref date_str) = payload.date {
        if let Ok(parsed) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
            updated.date = parsed;
        }
    }
    if let Some(pm) = payload.payment_mode { updated.payment_mode = pm; }
    if let Some(rem) = payload.remarks { updated.remarks = Some(rem); }
    if let Some(utr) = payload.utr { updated.utr = utr; }
    if let Some(receipt) = payload.receipt { updated.receipt = receipt; }
    if let Some(party) = payload.party_name { updated.party_name = party; }
    if let Some(spent) = payload.spent_by { updated.spent_by = spent; }
    if let Some(voucher) = payload.voucher_no { updated.voucher_no = voucher; }

    repository::update_expense(pool, &updated)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to update expense: {}", e)))
}

pub async fn delete_expense(pool: &PgPool, id: Uuid) -> Result<Expense, AppError> {
    repository::delete_expense(pool, id)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to delete expense: {}", e)))?
        .ok_or_else(|| AppError::NotFound("Expense record not found".to_string()))
}
