// backend-rust/src/modules/transport/service.rs
//! Transport Business Logic & Domain Service Layer

use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;
use crate::errors::AppError;
use crate::utils::activity::log_audit;
use super::models::{
    AddExpensePayload, AddTransportStudentPayload, AddVehiclePayload,
    GetExpensesQuery, GetTransportStudentsQuery, GetVehiclesQuery,
    ImportTransportStudentRow, TransportExpense, TransportExpenseWithCount,
    TransportStudent, TransportStudentWithDetails, UpdateExpensePayload,
    UpdateTransportStudentPayload, UpdateVehiclePayload, Vehicle,
    VehicleWithCount,
};
use super::repository;

// ─── Vehicles Services ────────────────────────────────────────────────────────

pub async fn list_vehicles(
    pool: &PgPool,
    q: &GetVehiclesQuery,
) -> Result<(Vec<VehicleWithCount>, i32, i64, i64), AppError> {
    let page = q.page.unwrap_or(1).max(1);
    let limit = q.limit.unwrap_or(50).max(1);
    let offset = (page - 1) * limit;

    let mut sql = "SELECT *, COUNT(*) OVER()::int AS total_count FROM vehicles WHERE 1=1".to_string();
    let mut binders = Vec::new();
    let mut idx = 1usize;

    if let Some(ref search) = q.search {
        let term = format!("%{}%", search.trim());
        sql.push_str(&format!(" AND (reg_no ILIKE ${idx} OR driver ILIKE ${idx} OR route ILIKE ${idx})"));
        binders.push(term);
        idx += 1;
    }

    if let Some(ref r#type) = q.r#type {
        if r#type != "All Types" {
            sql.push_str(&format!(" AND type = ${idx}"));
            binders.push(r#type.clone());
            idx += 1;
        }
    }

    if let Some(ref status) = q.status {
        if status != "All Statuses" {
            sql.push_str(&format!(" AND status = ${idx}"));
            binders.push(status.clone());
            idx += 1;
        }
    }

    sql.push_str(" ORDER BY reg_no ASC");
    sql.push_str(&format!(" LIMIT ${idx} OFFSET ${}", idx + 1));

    let list = repository::find_vehicles(pool, &sql, binders, limit, offset)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to fetch vehicles: {}", e)))?;

    let total = if list.is_empty() { 0 } else { list[0].total_count };

    Ok((list, total, page, limit))
}

pub async fn get_vehicle(pool: &PgPool, id: Uuid) -> Result<Vehicle, AppError> {
    repository::find_vehicle_by_id(pool, id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Vehicle not found".to_string()))
}

pub async fn create_vehicle(pool: &PgPool, payload: AddVehiclePayload) -> Result<Vehicle, AppError> {
    payload.validate()?;

    let reg_no_up = payload.reg_no.trim().to_uppercase();

    let existing = repository::find_vehicle_by_reg_no(pool, &reg_no_up)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    if existing.is_some() {
        return Err(AppError::Conflict(format!("Vehicle {} is already registered", reg_no_up)));
    }

    let capacity = payload.capacity.unwrap_or(40);
    let driver = payload.driver.as_deref().unwrap_or("—");
    let route = payload.route.as_deref().unwrap_or("—");
    let status = payload.status.as_deref().unwrap_or("active");
    let remarks = payload.remarks.as_deref().unwrap_or("—");

    repository::insert_vehicle(
        pool,
        &reg_no_up,
        payload.type_val.trim(),
        capacity,
        driver,
        route,
        status,
        remarks,
    )
    .await
    .map_err(|e| AppError::Internal(format!("Failed to create vehicle: {}", e)))
}

pub async fn update_vehicle(
    pool: &PgPool,
    id: Uuid,
    payload: UpdateVehiclePayload,
) -> Result<Vehicle, AppError> {
    payload.validate()?;

    let existing = repository::find_vehicle_by_id(pool, id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Vehicle not found".to_string()))?;

    let mut updated = existing;

    if let Some(ref reg_no) = payload.reg_no {
        let reg_no_up = reg_no.trim().to_uppercase();
        let duplicate = repository::find_duplicate_vehicle_reg_no(pool, &reg_no_up, id)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        if duplicate.is_some() {
            return Err(AppError::Conflict("Registration number already assigned to another vehicle".to_string()));
        }
        updated.reg_no = reg_no_up;
    }

    if let Some(type_val) = payload.type_val { updated.type_val = type_val; }
    if let Some(capacity) = payload.capacity { updated.capacity = capacity; }
    if let Some(driver) = payload.driver { updated.driver = driver; }
    if let Some(route) = payload.route { updated.route = route; }
    if let Some(status) = payload.status { updated.status = status; }
    if let Some(remarks) = payload.remarks { updated.remarks = remarks; }

    repository::update_vehicle(pool, &updated)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to update vehicle: {}", e)))
}

pub async fn delete_vehicle(pool: &PgPool, id: Uuid) -> Result<Vehicle, AppError> {
    repository::delete_vehicle(pool, id)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to delete vehicle: {}", e)))?
        .ok_or_else(|| AppError::NotFound("Vehicle not found".to_string()))
}

// ─── Expenses Services ────────────────────────────────────────────────────────

pub async fn list_expenses(
    pool: &PgPool,
    q: &GetExpensesQuery,
) -> Result<(Vec<TransportExpenseWithCount>, i32, i64, i64), AppError> {
    let page = q.page.unwrap_or(1).max(1);
    let limit = q.limit.unwrap_or(50).max(1);
    let offset = (page - 1) * limit;

    let mut sql = "SELECT id, date, vehicle_no, type, vendor, liters::float8 AS liters, rate::float8 AS rate, amount::float8 AS amount, payment_mode, remarks, created_at, utr_no, COUNT(*) OVER()::int AS total_count FROM transport_expenses WHERE 1=1".to_string();
    let mut binders = Vec::new();
    let mut idx = 1usize;

    if let Some(ref search) = q.search {
        let term = format!("%{}%", search.trim());
        sql.push_str(&format!(" AND (vendor ILIKE ${idx} OR remarks ILIKE ${idx} OR utr_no ILIKE ${idx})"));
        binders.push(term);
        idx += 1;
    }

    if let Some(ref r#type) = q.r#type {
        if r#type != "All Types" {
            sql.push_str(&format!(" AND type = ${idx}"));
            binders.push(r#type.clone());
            idx += 1;
        }
    }

    if let Some(ref vehicle_no) = q.vehicle_no {
        if vehicle_no != "All Vehicles" {
            sql.push_str(&format!(" AND vehicle_no = ${idx}"));
            binders.push(vehicle_no.clone());
            idx += 1;
        }
    }

    if let Some(ref start_date) = q.start_date {
        if let Ok(parsed) = NaiveDate::parse_from_str(start_date, "%Y-%m-%d") {
            sql.push_str(&format!(" AND date >= ${idx}"));
            binders.push(parsed.to_string());
            idx += 1;
        }
    }

    if let Some(ref end_date) = q.end_date {
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

pub async fn get_expense(pool: &PgPool, id: Uuid) -> Result<TransportExpense, AppError> {
    repository::find_expense_by_id(pool, id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Expense record not found".to_string()))
}

pub async fn create_expense(
    pool: &PgPool,
    user_role_str: &str,
    payload: AddExpensePayload,
) -> Result<TransportExpense, AppError> {
    payload.validate()?;

    let vehicle_no_up = payload.vehicle_no.trim().to_uppercase();

    // Verify vehicle exists
    let vehicle = repository::find_vehicle_by_reg_no(pool, &vehicle_no_up)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    if vehicle.is_none() {
        return Err(AppError::NotFound(format!("Vehicle {} is not registered", vehicle_no_up)));
    }

    let parsed_date = NaiveDate::parse_from_str(&payload.date, "%Y-%m-%d")
        .map_err(|_| AppError::BadRequest("Invalid date format".to_string()))?;

    let payment_mode = payload.payment_mode.as_deref().unwrap_or("Online");
    let utr_no = payload.utr_no.as_deref().unwrap_or("—");

    let expense = repository::insert_expense_record(
        pool,
        parsed_date,
        &vehicle_no_up,
        payload.type_val.trim(),
        payload.vendor.trim(),
        payload.liters,
        payload.rate,
        payload.amount,
        payment_mode,
        payload.remarks.as_deref(),
        utr_no,
    )
    .await
    .map_err(|e| AppError::Internal(format!("Failed to record expense: {}", e)))?;

    let _ = log_audit(
        pool,
        "Staff User",
        user_role_str,
        "EXPENSE_RECORDED",
        "transport",
        &format!("Vehicle {} - {} amount ₹{}", expense.vehicle_no, expense.type_val, expense.amount),
    )
    .await;

    Ok(expense)
}

pub async fn update_expense(
    pool: &PgPool,
    id: Uuid,
    payload: UpdateExpensePayload,
) -> Result<TransportExpense, AppError> {
    payload.validate()?;

    let existing = repository::find_expense_by_id(pool, id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Expense record not found".to_string()))?;

    let mut updated = existing;

    if let Some(ref vehicle_no) = payload.vehicle_no {
        let vehicle_no_up = vehicle_no.trim().to_uppercase();
        let vehicle = repository::find_vehicle_by_reg_no(pool, &vehicle_no_up)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        if vehicle.is_none() {
            return Err(AppError::NotFound(format!("Vehicle {} is not registered", vehicle_no_up)));
        }
        updated.vehicle_no = vehicle_no_up;
    }

    if let Some(ref date_str) = payload.date {
        if let Ok(parsed) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
            updated.date = parsed;
        }
    }

    if let Some(type_val) = payload.type_val { updated.type_val = type_val; }
    if let Some(vendor) = payload.vendor { updated.vendor = vendor; }
    if let Some(liters) = payload.liters { updated.liters = Some(liters); }
    if let Some(rate) = payload.rate { updated.rate = Some(rate); }
    if let Some(amount) = payload.amount { updated.amount = amount; }
    if let Some(payment_mode) = payload.payment_mode { updated.payment_mode = payment_mode; }
    if let Some(remarks) = payload.remarks { updated.remarks = Some(remarks); }
    if let Some(utr_no) = payload.utr_no { updated.utr_no = utr_no; }

    repository::update_expense_record(pool, &updated)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to update expense: {}", e)))
}

pub async fn delete_expense(pool: &PgPool, id: Uuid) -> Result<TransportExpense, AppError> {
    repository::delete_expense_record(pool, id)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to delete expense: {}", e)))?
        .ok_or_else(|| AppError::NotFound("Expense record not found".to_string()))
}

// ─── Students Services ────────────────────────────────────────────────────────

pub async fn list_transport_students(
    pool: &PgPool,
    q: &GetTransportStudentsQuery,
) -> Result<(Vec<TransportStudentWithDetails>, i32, i64, i64), AppError> {
    let page = q.page.unwrap_or(1).max(1);
    let limit = q.limit.unwrap_or(50).max(1);
    let offset = (page - 1) * limit;

    let mut sql = r#"
        SELECT ts.id, ts.student_id, ts.vehicle_no, ts.route, ts.pickup_point, ts.fee_amount::float8 AS fee_amount, ts.status, ts.remarks, ts.created_at, ts.updated_at,
               COALESCE(s.name, ts.student_id) AS student_name,
               s.class_name AS class_name,
               s.course_name AS course_name,
               COUNT(*) OVER()::int AS total_count
        FROM transport_students ts
        LEFT JOIN students s ON LOWER(TRIM(s.student_id)) = LOWER(TRIM(ts.student_id))
        WHERE 1=1
    "#
    .to_string();

    let mut binders = Vec::new();
    let mut idx = 1usize;

    if let Some(ref search) = q.search {
        let term = format!("%{}%", search.trim());
        sql.push_str(&format!(" AND (COALESCE(s.name, '') ILIKE ${idx} OR ts.student_id ILIKE ${idx} OR COALESCE(ts.pickup_point, '') ILIKE ${idx} OR COALESCE(ts.route, '') ILIKE ${idx} OR COALESCE(ts.vehicle_no, '') ILIKE ${idx})"));
        binders.push(term);
        idx += 1;
    }

    if let Some(ref route) = q.route {
        if route != "All Routes" {
            sql.push_str(&format!(" AND ts.route = ${idx}"));
            binders.push(route.clone());
            idx += 1;
        }
    }

    if let Some(ref status) = q.status {
        if status != "All Statuses" {
            sql.push_str(&format!(" AND ts.status = ${idx}"));
            binders.push(status.clone());
            idx += 1;
        }
    }

    if let Some(ref class_name) = q.class_name {
        if class_name != "All Classes" {
            sql.push_str(&format!(" AND COALESCE(s.class_name, '') = ${idx}"));
            binders.push(class_name.clone());
            idx += 1;
        }
    }

    if let Some(ref vehicle_no) = q.vehicle_no {
        if vehicle_no != "All Buses" {
            sql.push_str(&format!(" AND ts.vehicle_no = ${idx}"));
            binders.push(vehicle_no.clone());
            idx += 1;
        }
    }

    sql.push_str(" ORDER BY COALESCE(s.name, ts.student_id) ASC");
    sql.push_str(&format!(" LIMIT ${idx} OFFSET ${}", idx + 1));

    let list = repository::find_transport_students(pool, &sql, binders, limit, offset)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to fetch transport students: {}", e)))?;

    let total = if list.is_empty() { 0 } else { list[0].total_count };

    Ok((list, total, page, limit))
}

pub async fn get_transport_student(pool: &PgPool, id: Uuid) -> Result<TransportStudentWithDetails, AppError> {
    repository::find_transport_student_by_id(pool, id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Student transport record not found".to_string()))
}

pub async fn create_transport_student(
    pool: &PgPool,
    payload: AddTransportStudentPayload,
) -> Result<TransportStudent, AppError> {
    payload.validate()?;

    let master_sid = repository::find_master_student(pool, &payload.student_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let student_id = master_sid.unwrap_or_else(|| payload.student_id.trim().to_string());

    let mut vehicle_no_up = payload.vehicle_no.clone();

    if let Some(ref vehicle_no) = payload.vehicle_no {
        let v_up = vehicle_no.trim().to_uppercase();
        if !v_up.is_empty() && v_up != "—" {
            let vehicle_exists = repository::find_vehicle_by_reg_no(pool, &v_up)
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?;

            if vehicle_exists.is_none() {
                return Err(AppError::BadRequest(format!(
                    "Vehicle '{}' is not registered in transport fleet. Please select or register a valid vehicle first.",
                    v_up
                )));
            }
            vehicle_no_up = Some(v_up);
        }
    }

    let fee_amount = payload.fee_amount.unwrap_or(0.0);
    let status = payload.status.as_deref().unwrap_or("active");
    let remarks = payload.remarks.as_deref().unwrap_or("—");

    repository::upsert_transport_student(
        pool,
        &student_id,
        vehicle_no_up.as_deref(),
        payload.route.as_deref(),
        payload.pickup_point.as_deref(),
        fee_amount,
        status,
        remarks,
    )
    .await
    .map_err(|e| AppError::Internal(format!("Failed to create transport student: {}", e)))
}

pub async fn update_transport_student(
    pool: &PgPool,
    id: Uuid,
    payload: UpdateTransportStudentPayload,
) -> Result<TransportStudent, AppError> {
    payload.validate()?;

    let existing = repository::find_raw_transport_student_by_id(pool, id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Student transport record not found".to_string()))?;

    let mut updated = existing;

    if let Some(ref vehicle_no) = payload.vehicle_no {
        if !vehicle_no.is_empty() {
            let v_up = vehicle_no.trim().to_uppercase();
            let vehicle = repository::find_vehicle_by_reg_no(pool, &v_up)
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?;
            if vehicle.is_none() {
                return Err(AppError::NotFound(format!("Vehicle {} is not registered", v_up)));
            }
            updated.vehicle_no = Some(v_up);
        } else {
            updated.vehicle_no = None;
        }
    }

    if let Some(route) = payload.route { updated.route = Some(route); }
    if let Some(pickup_point) = payload.pickup_point { updated.pickup_point = Some(pickup_point); }
    if let Some(fee_amount) = payload.fee_amount { updated.fee_amount = fee_amount; }
    if let Some(status) = payload.status { updated.status = status; }
    if let Some(remarks) = payload.remarks { updated.remarks = Some(remarks); }

    repository::update_transport_student(pool, &updated)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to update transport student: {}", e)))
}

pub async fn delete_transport_student(pool: &PgPool, id: Uuid) -> Result<TransportStudent, AppError> {
    repository::delete_transport_student(pool, id)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to delete transport student: {}", e)))?
        .ok_or_else(|| AppError::NotFound("Student transport record not found".to_string()))
}

pub async fn import_transport_students(
    pool: &PgPool,
    rows: Vec<ImportTransportStudentRow>,
) -> Result<Vec<TransportStudent>, AppError> {
    let mut results = Vec::new();

    for row in rows {
        let student = repository::find_master_student(pool, &row.student_id)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        if student.is_none() {
            return Err(AppError::NotFound(format!("Student ID \"{}\" is not registered in the system", row.student_id)));
        }

        let mut vehicle_no_up = row.vehicle_no.clone();
        if let Some(ref vehicle_no) = row.vehicle_no {
            if !vehicle_no.is_empty() {
                let v_up = vehicle_no.trim().to_uppercase();
                let vehicle = repository::find_vehicle_by_reg_no(pool, &v_up)
                    .await
                    .map_err(|e| AppError::Internal(e.to_string()))?;
                if vehicle.is_none() {
                    return Err(AppError::NotFound(format!("Vehicle \"{}\" is not registered", vehicle_no)));
                }
                vehicle_no_up = Some(v_up);
            }
        }

        let fee_amount = row.fee_amount.unwrap_or(0.0);
        let status = row.status.as_deref().unwrap_or("active");
        let remarks = row.remarks.as_deref().unwrap_or("—");

        let record = repository::upsert_transport_student(
            pool,
            &row.student_id,
            vehicle_no_up.as_deref(),
            row.route.as_deref(),
            row.pickup_point.as_deref(),
            fee_amount,
            status,
            remarks,
        )
        .await
        .map_err(|e| AppError::Internal(format!("Failed to import student transport record: {}", e)))?;

        results.push(record);
    }

    Ok(results)
}
