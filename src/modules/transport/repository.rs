// backend-rust/src/modules/transport/repository.rs
//! Transport Data Access Layer (Repository)

use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;
use super::models::{
    AddExpensePayload, AddVehiclePayload, TransportExpense,
    TransportExpenseWithCount, TransportStudent, TransportStudentWithDetails,
    UpdateExpensePayload, UpdateTransportStudentPayload, UpdateVehiclePayload,
    Vehicle, VehicleWithCount,
};

// ─── Vehicles Data Access ─────────────────────────────────────────────────────

pub async fn find_vehicles(
    pool: &PgPool,
    sql: &str,
    binders: Vec<String>,
    limit: i64,
    offset: i64,
) -> Result<Vec<VehicleWithCount>, sqlx::Error> {
    let mut db_query = sqlx::query_as::<_, VehicleWithCount>(sql);
    for val in binders {
        db_query = db_query.bind(val);
    }
    db_query = db_query.bind(limit).bind(offset);

    db_query.fetch_all(pool).await
}

pub async fn find_vehicle_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Vehicle>, sqlx::Error> {
    sqlx::query_as::<_, Vehicle>(
        "SELECT id, reg_no, type, capacity, driver, route, status, remarks, created_at, updated_at FROM vehicles WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn find_vehicle_by_reg_no(pool: &PgPool, reg_no: &str) -> Result<Option<Uuid>, sqlx::Error> {
    let row = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM vehicles WHERE UPPER(TRIM(reg_no)) = UPPER(TRIM($1))"
    )
    .bind(reg_no)
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

pub async fn find_duplicate_vehicle_reg_no(pool: &PgPool, reg_no: &str, exclude_id: Uuid) -> Result<Option<Uuid>, sqlx::Error> {
    let row = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM vehicles WHERE UPPER(TRIM(reg_no)) = UPPER(TRIM($1)) AND id != $2"
    )
    .bind(reg_no)
    .bind(exclude_id)
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

pub async fn insert_vehicle(
    pool: &PgPool,
    reg_no_up: &str,
    type_val: &str,
    capacity: i32,
    driver: &str,
    route: &str,
    status: &str,
    remarks: &str,
) -> Result<Vehicle, sqlx::Error> {
    sqlx::query_as::<_, Vehicle>(
        r#"
        INSERT INTO vehicles (reg_no, type, capacity, driver, route, status, remarks)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id, reg_no, type, capacity, driver, route, status, remarks, created_at, updated_at
        "#
    )
    .bind(reg_no_up)
    .bind(type_val)
    .bind(capacity)
    .bind(driver)
    .bind(route)
    .bind(status)
    .bind(remarks)
    .fetch_one(pool)
    .await
}

pub async fn update_vehicle(
    pool: &PgPool,
    v: &Vehicle,
) -> Result<Vehicle, sqlx::Error> {
    sqlx::query_as::<_, Vehicle>(
        r#"
        UPDATE vehicles
        SET reg_no = $1, type = $2, capacity = $3, driver = $4, route = $5, status = $6, remarks = $7, updated_at = NOW()
        WHERE id = $8
        RETURNING id, reg_no, type, capacity, driver, route, status, remarks, created_at, updated_at
        "#
    )
    .bind(&v.reg_no)
    .bind(&v.type_val)
    .bind(v.capacity)
    .bind(&v.driver)
    .bind(&v.route)
    .bind(&v.status)
    .bind(&v.remarks)
    .bind(v.id)
    .fetch_one(pool)
    .await
}

pub async fn delete_vehicle(pool: &PgPool, id: Uuid) -> Result<Option<Vehicle>, sqlx::Error> {
    sqlx::query_as::<_, Vehicle>(
        r#"
        DELETE FROM vehicles WHERE id = $1
        RETURNING id, reg_no, type, capacity, driver, route, status, remarks, created_at, updated_at
        "#
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

// ─── Expenses Data Access ─────────────────────────────────────────────────────

pub async fn find_expenses(
    pool: &PgPool,
    sql: &str,
    binders: Vec<String>,
    limit: i64,
    offset: i64,
) -> Result<Vec<TransportExpenseWithCount>, sqlx::Error> {
    let mut db_query = sqlx::query_as::<_, TransportExpenseWithCount>(sql);
    for val in binders {
        db_query = db_query.bind(val);
    }
    db_query = db_query.bind(limit).bind(offset);

    db_query.fetch_all(pool).await
}

pub async fn find_expense_by_id(pool: &PgPool, id: Uuid) -> Result<Option<TransportExpense>, sqlx::Error> {
    sqlx::query_as::<_, TransportExpense>(
        r#"
        SELECT id, date, vehicle_no, type, vendor, liters::float8 AS liters, rate::float8 AS rate, amount::float8 AS amount, payment_mode, remarks, created_at, utr_no
        FROM transport_expenses WHERE id = $1
        "#
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn insert_expense_record(
    pool: &PgPool,
    parsed_date: NaiveDate,
    vehicle_no: &str,
    type_val: &str,
    vendor: &str,
    liters: Option<f64>,
    rate: Option<f64>,
    amount: f64,
    payment_mode: &str,
    remarks: Option<&str>,
    utr_no: &str,
) -> Result<TransportExpense, sqlx::Error> {
    let mut tx = pool.begin().await?;

    let expense = sqlx::query_as::<_, TransportExpense>(
        r#"
        INSERT INTO transport_expenses (date, vehicle_no, type, vendor, liters, rate, amount, payment_mode, remarks, utr_no)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        RETURNING id, date, vehicle_no, type, vendor, liters::float8 AS liters, rate::float8 AS rate, amount::float8 AS amount, payment_mode, remarks, created_at, utr_no
        "#
    )
    .bind(parsed_date)
    .bind(vehicle_no)
    .bind(type_val)
    .bind(vendor)
    .bind(liters)
    .bind(rate)
    .bind(amount)
    .bind(payment_mode)
    .bind(remarks)
    .bind(utr_no)
    .fetch_one(&mut *tx)
    .await?;

    // Also sync to master expenses table
    let ref_no = format!("TRP-{}", &expense.id.to_string()[..8].to_uppercase());
    let description = format!("Transport {} expense: {} (Vehicle: {})", type_val, vendor, vehicle_no);

    sqlx::query(
        r#"
        INSERT INTO expenses (ref_no, description, amount, category, date, payment_mode, remarks, utr, party_name, spent_by, voucher_no)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        ON CONFLICT (ref_no) DO NOTHING
        "#
    )
    .bind(&ref_no)
    .bind(&description)
    .bind(amount)
    .bind("Transport")
    .bind(parsed_date)
    .bind(payment_mode)
    .bind(remarks)
    .bind(utr_no)
    .bind(vendor)
    .bind("Transport Dept")
    .bind(&ref_no)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(expense)
}

pub async fn update_expense_record(
    pool: &PgPool,
    e: &TransportExpense,
) -> Result<TransportExpense, sqlx::Error> {
    sqlx::query_as::<_, TransportExpense>(
        r#"
        UPDATE transport_expenses
        SET date = $1, vehicle_no = $2, type = $3, vendor = $4, liters = $5, rate = $6, amount = $7, payment_mode = $8, remarks = $9, utr_no = $10
        WHERE id = $11
        RETURNING id, date, vehicle_no, type, vendor, liters::float8 AS liters, rate::float8 AS rate, amount::float8 AS amount, payment_mode, remarks, created_at, utr_no
        "#
    )
    .bind(e.date)
    .bind(&e.vehicle_no)
    .bind(&e.type_val)
    .bind(&e.vendor)
    .bind(e.liters)
    .bind(e.rate)
    .bind(e.amount)
    .bind(&e.payment_mode)
    .bind(&e.remarks)
    .bind(&e.utr_no)
    .bind(e.id)
    .fetch_one(pool)
    .await
}

pub async fn delete_expense_record(pool: &PgPool, id: Uuid) -> Result<Option<TransportExpense>, sqlx::Error> {
    sqlx::query_as::<_, TransportExpense>(
        r#"
        DELETE FROM transport_expenses WHERE id = $1
        RETURNING id, date, vehicle_no, type, vendor, liters::float8 AS liters, rate::float8 AS rate, amount::float8 AS amount, payment_mode, remarks, created_at, utr_no
        "#
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

// ─── Students Data Access ─────────────────────────────────────────────────────

pub async fn find_transport_students(
    pool: &PgPool,
    sql: &str,
    binders: Vec<String>,
    limit: i64,
    offset: i64,
) -> Result<Vec<TransportStudentWithDetails>, sqlx::Error> {
    let mut db_query = sqlx::query_as::<_, TransportStudentWithDetails>(sql);
    for val in binders {
        db_query = db_query.bind(val);
    }
    db_query = db_query.bind(limit).bind(offset);

    db_query.fetch_all(pool).await
}

pub async fn find_transport_student_by_id(pool: &PgPool, id: Uuid) -> Result<Option<TransportStudentWithDetails>, sqlx::Error> {
    sqlx::query_as::<_, TransportStudentWithDetails>(
        r#"
        SELECT ts.id, ts.student_id, ts.vehicle_no, ts.route, ts.pickup_point, ts.fee_amount::float8 AS fee_amount, ts.status, ts.remarks, ts.created_at, ts.updated_at,
               COALESCE(s.name, ts.student_id) AS student_name, s.class_name AS class_name, s.course_name AS course_name,
               1::int AS total_count
        FROM transport_students ts
        LEFT JOIN students s ON LOWER(TRIM(s.student_id)) = LOWER(TRIM(ts.student_id))
        WHERE ts.id = $1
        "#
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn find_raw_transport_student_by_id(pool: &PgPool, id: Uuid) -> Result<Option<TransportStudent>, sqlx::Error> {
    sqlx::query_as::<_, TransportStudent>(
        "SELECT id, student_id, vehicle_no, route, pickup_point, fee_amount::float8 AS fee_amount, status, remarks, created_at, updated_at FROM transport_students WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn find_master_student(pool: &PgPool, student_id: &str) -> Result<Option<String>, sqlx::Error> {
    let row = sqlx::query_scalar::<_, String>(
        "SELECT student_id FROM students WHERE LOWER(TRIM(student_id)) = LOWER(TRIM($1))"
    )
    .bind(student_id)
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

pub async fn upsert_transport_student(
    pool: &PgPool,
    student_id: &str,
    vehicle_no: Option<&str>,
    route: Option<&str>,
    pickup_point: Option<&str>,
    fee_amount: f64,
    status: &str,
    remarks: &str,
) -> Result<TransportStudent, sqlx::Error> {
    sqlx::query_as::<_, TransportStudent>(
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
            updated_at = NOW()
        RETURNING id, student_id, vehicle_no, route, pickup_point, fee_amount::float8 AS fee_amount, status, remarks, created_at, updated_at
        "#
    )
    .bind(student_id)
    .bind(vehicle_no)
    .bind(route)
    .bind(pickup_point)
    .bind(fee_amount)
    .bind(status)
    .bind(remarks)
    .fetch_one(pool)
    .await
}

pub async fn update_transport_student(
    pool: &PgPool,
    ts: &TransportStudent,
) -> Result<TransportStudent, sqlx::Error> {
    sqlx::query_as::<_, TransportStudent>(
        r#"
        UPDATE transport_students
        SET vehicle_no = $1, route = $2, pickup_point = $3, fee_amount = $4, status = $5, remarks = $6, updated_at = NOW()
        WHERE id = $7
        RETURNING id, student_id, vehicle_no, route, pickup_point, fee_amount::float8 AS fee_amount, status, remarks, created_at, updated_at
        "#
    )
    .bind(&ts.vehicle_no)
    .bind(&ts.route)
    .bind(&ts.pickup_point)
    .bind(ts.fee_amount)
    .bind(&ts.status)
    .bind(&ts.remarks)
    .bind(ts.id)
    .fetch_one(pool)
    .await
}

pub async fn delete_transport_student(pool: &PgPool, id: Uuid) -> Result<Option<TransportStudent>, sqlx::Error> {
    sqlx::query_as::<_, TransportStudent>(
        r#"
        DELETE FROM transport_students WHERE id = $1
        RETURNING id, student_id, vehicle_no, route, pickup_point, fee_amount::float8 AS fee_amount, status, remarks, created_at, updated_at
        "#
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}
