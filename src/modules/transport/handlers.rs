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
use crate::modules::auth::models::{ApiResponse, UserSubRole};
use crate::utils::activity::log_audit;
use super::models::*;

// ─── VEHICLES ─────────────────────────────────────────────

pub async fn get_vehicles(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Query(q): Query<GetVehiclesQuery>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize_sub_role(&[UserSubRole::TransportManager, UserSubRole::FinanceManager])?;

    let page = q.page.unwrap_or(1).max(1);
    let limit = q.limit.unwrap_or(50).max(1);
    let offset = (page - 1) * limit;

    let mut sql = "SELECT *, COUNT(*) OVER()::int AS total_count FROM vehicles WHERE 1=1".to_string();
    let mut binders = Vec::new();
    let mut idx = 1usize;

    if let Some(ref search) = q.search {
        let term = format!("%{}%", search);
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

    let mut db_query = sqlx::query_as::<_, VehicleWithCount>(&sql);
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

pub async fn get_vehicle(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize_sub_role(&[UserSubRole::TransportManager, UserSubRole::FinanceManager])?;

    let vehicle = sqlx::query_as::<_, Vehicle>(
        "SELECT id, reg_no, type, capacity, driver, route, status, remarks, created_at, updated_at FROM vehicles WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?;

    let vehicle = vehicle.ok_or_else(|| AppError::NotFound("Vehicle not found".to_string()))?;
    Ok(Json(ApiResponse { success: true, data: vehicle }))
}

pub async fn create_vehicle(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(payload): Json<AddVehiclePayload>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize_sub_role(&[UserSubRole::TransportManager, UserSubRole::FinanceManager])?;
    payload.validate()?;

    let reg_no_up = payload.reg_no.to_uppercase();

    // Check duplicate
    let existing = sqlx::query("SELECT id FROM vehicles WHERE reg_no = $1")
        .bind(&reg_no_up)
        .fetch_optional(&state.db)
        .await?;

    if existing.is_some() {
        return Err(AppError::Conflict(format!("Vehicle {} is already registered", reg_no_up)));
    }

    let capacity = payload.capacity.unwrap_or(40);
    let driver = payload.driver.as_deref().unwrap_or("—");
    let route = payload.route.as_deref().unwrap_or("—");
    let status = payload.status.as_deref().unwrap_or("active");
    let remarks = payload.remarks.as_deref().unwrap_or("—");

    let vehicle = sqlx::query_as::<_, Vehicle>(
        r#"
        INSERT INTO vehicles (reg_no, type, capacity, driver, route, status, remarks)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id, reg_no, type, capacity, driver, route, status, remarks, created_at, updated_at
        "#
    )
    .bind(&reg_no_up)
    .bind(&payload.type_val)
    .bind(capacity)
    .bind(driver)
    .bind(route)
    .bind(status)
    .bind(remarks)
    .fetch_one(&state.db)
    .await?;

    Ok((StatusCode::CREATED, Json(ApiResponse { success: true, data: vehicle })))
}

pub async fn edit_vehicle(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateVehiclePayload>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize_sub_role(&[UserSubRole::TransportManager, UserSubRole::FinanceManager])?;
    payload.validate()?;

    // Fetch existing
    let existing = sqlx::query_as::<_, Vehicle>(
        "SELECT id, reg_no, type, capacity, driver, route, status, remarks, created_at, updated_at FROM vehicles WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?;

    let mut existing = existing.ok_or_else(|| AppError::NotFound("Vehicle not found".to_string()))?;

    if let Some(ref reg_no) = payload.reg_no {
        let reg_no_up = reg_no.to_uppercase();
        let duplicate = sqlx::query("SELECT id FROM vehicles WHERE reg_no = $1 AND id != $2")
            .bind(&reg_no_up)
            .bind(id)
            .fetch_optional(&state.db)
            .await?;
        if duplicate.is_some() {
            return Err(AppError::Conflict("Registration number already assigned to another vehicle".to_string()));
        }
        existing.reg_no = reg_no_up;
    }

    if let Some(type_val) = payload.type_val { existing.type_val = type_val; }
    if let Some(capacity) = payload.capacity { existing.capacity = capacity; }
    if let Some(driver) = payload.driver { existing.driver = driver; }
    if let Some(route) = payload.route { existing.route = route; }
    if let Some(status) = payload.status { existing.status = status; }
    if let Some(remarks) = payload.remarks { existing.remarks = remarks; }

    let vehicle = sqlx::query_as::<_, Vehicle>(
        r#"
        UPDATE vehicles
        SET reg_no = $1, type = $2, capacity = $3, driver = $4, route = $5, status = $6, remarks = $7, updated_at = now()
        WHERE id = $8
        RETURNING id, reg_no, type, capacity, driver, route, status, remarks, created_at, updated_at
        "#
    )
    .bind(&existing.reg_no)
    .bind(&existing.type_val)
    .bind(existing.capacity)
    .bind(&existing.driver)
    .bind(&existing.route)
    .bind(&existing.status)
    .bind(&existing.remarks)
    .bind(id)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(ApiResponse { success: true, data: vehicle }))
}

pub async fn remove_vehicle(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize_sub_role(&[UserSubRole::TransportManager, UserSubRole::FinanceManager])?;

    let deleted = sqlx::query_as::<_, Vehicle>(
        r#"
        DELETE FROM vehicles WHERE id = $1
        RETURNING id, reg_no, type, capacity, driver, route, status, remarks, created_at, updated_at
        "#
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?;

    let vehicle = deleted.ok_or_else(|| AppError::NotFound("Vehicle not found".to_string()))?;
    Ok(Json(ApiResponse { success: true, data: vehicle }))
}

// ─── EXPENSES ─────────────────────────────────────────────

pub async fn get_expenses(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Query(q): Query<GetExpensesQuery>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize_sub_role(&[UserSubRole::TransportManager, UserSubRole::FinanceManager])?;

    let page = q.page.unwrap_or(1).max(1);
    let limit = q.limit.unwrap_or(50).max(1);
    let offset = (page - 1) * limit;

    let mut sql = "SELECT id, date, vehicle_no, type, vendor, liters::float8 AS liters, rate::float8 AS rate, amount::float8 AS amount, payment_mode, remarks, created_at, utr_no, COUNT(*) OVER()::int AS total_count FROM transport_expenses WHERE 1=1".to_string();
    let mut binders = Vec::new();
    let mut idx = 1usize;

    if let Some(ref search) = q.search {
        let term = format!("%{}%", search);
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

    let mut db_query = sqlx::query_as::<_, TransportExpenseWithCount>(&sql);
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
    auth_user.authorize_sub_role(&[UserSubRole::TransportManager, UserSubRole::FinanceManager])?;

    let expense = sqlx::query_as::<_, TransportExpense>(
        r#"
        SELECT id, date, vehicle_no, type, vendor, liters::float8 AS liters, rate::float8 AS rate, amount::float8 AS amount, payment_mode, remarks, created_at, utr_no
        FROM transport_expenses WHERE id = $1
        "#
    )
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
    auth_user.authorize_sub_role(&[UserSubRole::TransportManager, UserSubRole::FinanceManager])?;
    payload.validate()?;

    let vehicle_no_up = payload.vehicle_no.to_uppercase();

    // Verify vehicle exists (auto-create if missing for bulk imports)
    let vehicle = sqlx::query("SELECT id FROM vehicles WHERE reg_no = $1")
        .bind(&vehicle_no_up)
        .fetch_optional(&state.db)
        .await?;

    if vehicle.is_none() {
        let _ = sqlx::query(
            r#"
            INSERT INTO vehicles (reg_no, type, capacity, status)
            VALUES ($1, 'Bus', 40, 'active')
            ON CONFLICT (reg_no) DO NOTHING
            "#
        )
        .bind(&vehicle_no_up)
        .execute(&state.db)
        .await;
    }

    let parsed_date = chrono::NaiveDate::parse_from_str(&payload.date, "%Y-%m-%d").unwrap();
    let payment_mode = payload.payment_mode.as_deref().unwrap_or("Online");
    let remarks = payload.remarks.as_deref().unwrap_or("—");
    let utr_no = payload.utr_no.as_deref().unwrap_or("—");

    let mut tx = state.db.begin().await?;

    // Insert transport expense
    let expense = sqlx::query_as::<_, TransportExpense>(
        r#"
        INSERT INTO transport_expenses (date, vehicle_no, type, vendor, liters, rate, amount, payment_mode, remarks, utr_no)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        RETURNING id, date, vehicle_no, type, vendor, liters::float8 AS liters, rate::float8 AS rate, amount::float8 AS amount, payment_mode, remarks, created_at, utr_no
        "#
    )
    .bind(parsed_date)
    .bind(&vehicle_no_up)
    .bind(&payload.type_val)
    .bind(&payload.vendor)
    .bind(payload.liters)
    .bind(payload.rate)
    .bind(payload.amount)
    .bind(payment_mode)
    .bind(remarks)
    .bind(utr_no)
    .fetch_one(&mut *tx)
    .await?;

    // Call into accounts: create a general expense entry
    let ref_no = format!("EXP-TR-{}", rand::random_range(1000..10000));
    let description = format!(
        "Transport Operational Expense - {} for vehicle {} (Vendor: {})",
        payload.type_val, vehicle_no_up, payload.vendor
    );

    sqlx::query(
        r#"
        INSERT INTO expenses (ref_no, description, amount, category, date, payment_mode, remarks, utr, party_name, spent_by, voucher_no)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        "#
    )
    .bind(&ref_no)
    .bind(&description)
    .bind(payload.amount)
    .bind("Transport")
    .bind(parsed_date)
    .bind(payment_mode)
    .bind(remarks)
    .bind(utr_no)
    .bind(&payload.vendor)
    .bind("Transport Dept")
    .bind(&ref_no)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    let user_role_str = format!("{:?}", auth_user.role);
    let _ = log_audit(
        &state.db,
        "Staff User",
        &user_role_str,
        "EXPENSE_RECORDED",
        "transport",
        &format!("Vehicle {} - {} amount ₹{}", expense.vehicle_no, expense.type_val, expense.amount)
    ).await;

    Ok((StatusCode::CREATED, Json(ApiResponse { success: true, data: expense })))
}

pub async fn edit_expense(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateExpensePayload>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize_sub_role(&[UserSubRole::TransportManager, UserSubRole::FinanceManager])?;
    payload.validate()?;

    // Fetch existing
    let existing = sqlx::query_as::<_, TransportExpense>(
        r#"
        SELECT id, date, vehicle_no, type, vendor, liters::float8 AS liters, rate::float8 AS rate, amount::float8 AS amount, payment_mode, remarks, created_at, utr_no
        FROM transport_expenses WHERE id = $1
        "#
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?;

    let mut existing = existing.ok_or_else(|| AppError::NotFound("Expense record not found".to_string()))?;

    if let Some(ref vehicle_no) = payload.vehicle_no {
        let vehicle_no_up = vehicle_no.to_uppercase();
        let vehicle = sqlx::query("SELECT id FROM vehicles WHERE reg_no = $1")
            .bind(&vehicle_no_up)
            .fetch_optional(&state.db)
            .await?;
        if vehicle.is_none() {
            return Err(AppError::NotFound(format!("Vehicle {} is not registered", vehicle_no_up)));
        }
        existing.vehicle_no = vehicle_no_up;
    }

    if let Some(ref date_str) = payload.date {
        existing.date = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d").unwrap();
    }
    if let Some(type_val) = payload.type_val { existing.type_val = type_val; }
    if let Some(vendor) = payload.vendor { existing.vendor = vendor; }
    if let Some(liters) = payload.liters { existing.liters = Some(liters); }
    if let Some(rate) = payload.rate { existing.rate = Some(rate); }
    if let Some(amount) = payload.amount { existing.amount = amount; }
    if let Some(payment_mode) = payload.payment_mode { existing.payment_mode = payment_mode; }
    if let Some(remarks) = payload.remarks { existing.remarks = Some(remarks); }
    if let Some(utr_no) = payload.utr_no { existing.utr_no = utr_no; }

    let expense = sqlx::query_as::<_, TransportExpense>(
        r#"
        UPDATE transport_expenses
        SET date = $1, vehicle_no = $2, type = $3, vendor = $4, liters = $5, rate = $6, amount = $7, payment_mode = $8, remarks = $9, utr_no = $10
        WHERE id = $11
        RETURNING id, date, vehicle_no, type, vendor, liters::float8 AS liters, rate::float8 AS rate, amount::float8 AS amount, payment_mode, remarks, created_at, utr_no
        "#
    )
    .bind(existing.date)
    .bind(&existing.vehicle_no)
    .bind(&existing.type_val)
    .bind(&existing.vendor)
    .bind(existing.liters)
    .bind(existing.rate)
    .bind(existing.amount)
    .bind(&existing.payment_mode)
    .bind(existing.remarks)
    .bind(&existing.utr_no)
    .bind(id)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(ApiResponse { success: true, data: expense }))
}

pub async fn remove_expense(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize_sub_role(&[UserSubRole::TransportManager, UserSubRole::FinanceManager])?;

    let deleted = sqlx::query_as::<_, TransportExpense>(
        r#"
        DELETE FROM transport_expenses WHERE id = $1
        RETURNING id, date, vehicle_no, type, vendor, liters::float8 AS liters, rate::float8 AS rate, amount::float8 AS amount, payment_mode, remarks, created_at, utr_no
        "#
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?;

    let expense = deleted.ok_or_else(|| AppError::NotFound("Expense record not found".to_string()))?;
    Ok(Json(ApiResponse { success: true, data: expense }))
}

// ─── TRANSPORT STUDENTS ───────────────────────────────────

pub async fn get_transport_students(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Query(q): Query<GetTransportStudentsQuery>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize_sub_role(&[UserSubRole::TransportManager, UserSubRole::FinanceManager])?;

    let page = q.page.unwrap_or(1).max(1);
    let limit = q.limit.unwrap_or(50).max(1);
    let offset = (page - 1) * limit;

    let mut sql = r#"
        SELECT ts.id, ts.student_id, ts.vehicle_no, ts.route, ts.pickup_point, ts.fee_amount::float8 AS fee_amount, ts.status, ts.remarks, ts.created_at, ts.updated_at,
               COALESCE(s.name, ts.student_id) AS student_name, COALESCE(s.class_name, '—') AS class,
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

    let mut db_query = sqlx::query_as::<_, TransportStudentWithDetails>(&sql);
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

pub async fn get_transport_student(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize_sub_role(&[UserSubRole::TransportManager, UserSubRole::FinanceManager])?;

    let student = sqlx::query_as::<_, TransportStudentWithDetails>(
        r#"
        SELECT ts.id, ts.student_id, ts.vehicle_no, ts.route, ts.pickup_point, ts.fee_amount::float8 AS fee_amount, ts.status, ts.remarks, ts.created_at, ts.updated_at,
               COALESCE(s.name, ts.student_id) AS student_name, COALESCE(s.class_name, '—') AS class,
               1::int AS total_count
        FROM transport_students ts
        LEFT JOIN students s ON LOWER(TRIM(s.student_id)) = LOWER(TRIM(ts.student_id))
        WHERE ts.id = $1
        "#
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?;

    let student = student.ok_or_else(|| AppError::NotFound("Student transport record not found".to_string()))?;
    Ok(Json(ApiResponse { success: true, data: student }))
}

pub async fn create_transport_student(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(payload): Json<AddTransportStudentPayload>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize_sub_role(&[UserSubRole::TransportManager, UserSubRole::FinanceManager])?;
    payload.validate()?;

    // 1. Verify student exists in master student registry
    let student = sqlx::query_as::<_, (String,)>("SELECT student_id FROM students WHERE LOWER(TRIM(student_id)) = LOWER(TRIM($1))")
        .bind(&payload.student_id)
        .fetch_optional(&state.db)
        .await?;

    let student_id = match student {
        Some((sid,)) => sid,
        None => payload.student_id.trim().to_string(),
    };

    let mut vehicle_no_up = payload.vehicle_no.clone();

    // 2. Verify or auto-register vehicle if provided
    if let Some(ref vehicle_no) = payload.vehicle_no {
        if !vehicle_no.trim().is_empty() {
            let v_up = vehicle_no.trim().to_uppercase();
            let vehicle = sqlx::query("SELECT id FROM vehicles WHERE UPPER(reg_no) = $1")
                .bind(&v_up)
                .fetch_optional(&state.db)
                .await?;
            if vehicle.is_none() {
                let _ = sqlx::query(
                    "INSERT INTO vehicles (reg_no, type, capacity, driver_name, driver_phone, status, fitness_expiry, insurance_expiry, puc_expiry)
                     VALUES ($1, 'Bus', 40, '—', '—', 'active', now() + interval '1 year', now() + interval '1 year', now() + interval '1 year')
                     ON CONFLICT (reg_no) DO NOTHING"
                )
                .bind(&v_up)
                .execute(&state.db)
                .await;
            }
            vehicle_no_up = Some(v_up);
        }
    }

    let fee_amount = payload.fee_amount.unwrap_or(0.0);
    let status = payload.status.as_deref().unwrap_or("active");
    let remarks = payload.remarks.as_deref().unwrap_or("—");

    // 3. Insert or update existing transport student mapping
    let record = sqlx::query_as::<_, TransportStudent>(
        r#"
        INSERT INTO transport_students (student_id, vehicle_no, route, pickup_point, fee_amount, status, remarks)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT (student_id) DO UPDATE
        SET vehicle_no = EXCLUDED.vehicle_no,
            route = EXCLUDED.route,
            pickup_point = EXCLUDED.pickup_point,
            fee_amount = EXCLUDED.fee_amount,
            status = EXCLUDED.status,
            remarks = EXCLUDED.remarks,
            updated_at = now()
        RETURNING id, student_id, vehicle_no, route, pickup_point, fee_amount::float8 AS fee_amount, status, remarks, created_at, updated_at
        "#
    )
    .bind(&student_id)
    .bind(vehicle_no_up)
    .bind(&payload.route)
    .bind(&payload.pickup_point)
    .bind(fee_amount)
    .bind(status)
    .bind(remarks)
    .fetch_one(&state.db)
    .await?;

    Ok((StatusCode::CREATED, Json(ApiResponse { success: true, data: record })))
}

pub async fn edit_transport_student(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateTransportStudentPayload>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize_sub_role(&[UserSubRole::TransportManager, UserSubRole::FinanceManager])?;
    payload.validate()?;

    // Fetch existing
    let existing = sqlx::query_as::<_, TransportStudent>(
        "SELECT id, student_id, vehicle_no, route, pickup_point, fee_amount::float8 AS fee_amount, status, remarks, created_at, updated_at FROM transport_students WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?;

    let mut existing = existing.ok_or_else(|| AppError::NotFound("Student transport record not found".to_string()))?;

    if let Some(ref vehicle_no) = payload.vehicle_no {
        if !vehicle_no.is_empty() {
            let v_up = vehicle_no.to_uppercase();
            let vehicle = sqlx::query("SELECT id FROM vehicles WHERE reg_no = $1")
                .bind(&v_up)
                .fetch_optional(&state.db)
                .await?;
            if vehicle.is_none() {
                return Err(AppError::NotFound(format!("Vehicle {} is not registered", v_up)));
            }
            existing.vehicle_no = Some(v_up);
        } else {
            existing.vehicle_no = None;
        }
    }

    if let Some(route) = payload.route { existing.route = Some(route); }
    if let Some(pickup_point) = payload.pickup_point { existing.pickup_point = Some(pickup_point); }
    if let Some(fee_amount) = payload.fee_amount { existing.fee_amount = fee_amount; }
    if let Some(status) = payload.status { existing.status = status; }
    if let Some(remarks) = payload.remarks { existing.remarks = Some(remarks); }

    let record = sqlx::query_as::<_, TransportStudent>(
        r#"
        UPDATE transport_students
        SET vehicle_no = $1, route = $2, pickup_point = $3, fee_amount = $4, status = $5, remarks = $6, updated_at = now()
        WHERE id = $7
        RETURNING id, student_id, vehicle_no, route, pickup_point, fee_amount::float8 AS fee_amount, status, remarks, created_at, updated_at
        "#
    )
    .bind(existing.vehicle_no)
    .bind(existing.route)
    .bind(existing.pickup_point)
    .bind(existing.fee_amount)
    .bind(existing.status)
    .bind(existing.remarks)
    .bind(id)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(ApiResponse { success: true, data: record }))
}

pub async fn remove_transport_student(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize_sub_role(&[UserSubRole::TransportManager, UserSubRole::FinanceManager])?;

    let deleted = sqlx::query_as::<_, TransportStudent>(
        r#"
        DELETE FROM transport_students WHERE id = $1
        RETURNING id, student_id, vehicle_no, route, pickup_point, fee_amount::float8 AS fee_amount, status, remarks, created_at, updated_at
        "#
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?;

    let record = deleted.ok_or_else(|| AppError::NotFound("Student transport record not found".to_string()))?;
    Ok(Json(ApiResponse { success: true, data: record }))
}

pub async fn import_transport_students(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(payload): Json<Vec<ImportTransportStudentRow>>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize_sub_role(&[UserSubRole::TransportManager, UserSubRole::FinanceManager])?;

    let mut results = Vec::new();

    for row in payload {
        // 1. Verify student exists
        let student = sqlx::query("SELECT id FROM students WHERE student_id = $1")
            .bind(&row.student_id)
            .fetch_optional(&state.db)
            .await?;

        if student.is_none() {
            return Err(AppError::NotFound(format!("Student ID \"{}\" is not registered in the system", row.student_id)));
        }

        let mut vehicle_no_up = row.vehicle_no.clone();

        // 2. Verify vehicle if provided
        if let Some(ref vehicle_no) = row.vehicle_no {
            if !vehicle_no.is_empty() {
                let v_up = vehicle_no.to_uppercase();
                let vehicle = sqlx::query("SELECT id FROM vehicles WHERE reg_no = $1")
                    .bind(&v_up)
                    .fetch_optional(&state.db)
                    .await?;
                if vehicle.is_none() {
                    return Err(AppError::NotFound(format!("Vehicle \"{}\" is not registered", vehicle_no)));
                }
                vehicle_no_up = Some(v_up);
            }
        }

        let fee_amount = row.fee_amount.unwrap_or(0.0);
        let status = row.status.as_deref().unwrap_or("active");
        let remarks = row.remarks.as_deref().unwrap_or("—");

        // 3. Upsert student transport assignment
        let record = sqlx::query_as::<_, TransportStudent>(
            r#"
            INSERT INTO transport_students (student_id, vehicle_no, route, pickup_point, fee_amount, status, remarks)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (student_id) DO UPDATE SET
                vehicle_no = EXCLUDED.vehicle_no,
                route = EXCLUDED.route,
                pickup_point = EXCLUDED.pickup_point,
                fee_amount = EXCLUDED.fee_amount,
                status = EXCLUDED.status,
                remarks = EXCLUDED.remarks,
                updated_at = now()
            RETURNING id, student_id, vehicle_no, route, pickup_point, fee_amount::float8 AS fee_amount, status, remarks, created_at, updated_at
            "#
        )
        .bind(&row.student_id)
        .bind(vehicle_no_up)
        .bind(&row.route)
        .bind(&row.pickup_point)
        .bind(fee_amount)
        .bind(status)
        .bind(remarks)
        .fetch_one(&state.db)
        .await?;

        results.push(record);
    }

    Ok(Json(json!({ "success": true, "count": results.len(), "data": results })))
}
