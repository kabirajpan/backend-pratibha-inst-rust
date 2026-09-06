// backend-rust/src/modules/hostel/repository.rs
//! Hostel Data Access Layer (Repository)

use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;
use super::models::{
    GetHostelRoomsQuery, GetHostelStudentsQuery, HostelRoom,
    HostelRoomWithOccupancy, HostelStudent, HostelStudentWithDetails,
};

// ─── Rooms Data Access ────────────────────────────────────────────────────────

pub async fn find_rooms(pool: &PgPool, q: &GetHostelRoomsQuery) -> Result<Vec<HostelRoomWithOccupancy>, sqlx::Error> {
    let mut sql = r#"
        SELECT hr.id, hr.room_no, hr.block, hr.floor, hr.capacity, hr.room_type,
               hr.fee_per_term::float8 AS fee_per_term, hr.status, hr.remarks,
               hr.created_at, hr.updated_at,
               COALESCE((SELECT COUNT(*) FROM hostel_students hs WHERE hs.room_no = hr.room_no AND hs.status = 'active'), 0)::bigint AS occupied_beds
        FROM hostel_rooms hr
        WHERE 1=1
    "#
    .to_string();

    let mut binders = Vec::new();
    let mut idx = 1usize;

    if let Some(ref search) = q.search {
        let term = format!("%{}%", search.trim());
        sql.push_str(&format!(" AND (hr.room_no ILIKE ${idx} OR hr.block ILIKE ${idx} OR hr.floor ILIKE ${idx})"));
        binders.push(term);
        idx += 1;
    }

    if let Some(ref block) = q.block {
        if block != "All Blocks" {
            sql.push_str(&format!(" AND hr.block = ${idx}"));
            binders.push(block.clone());
            idx += 1;
        }
    }

    if let Some(ref rtype) = q.room_type {
        if rtype != "All Types" {
            sql.push_str(&format!(" AND hr.room_type = ${idx}"));
            binders.push(rtype.clone());
            idx += 1;
        }
    }

    if let Some(ref status) = q.status {
        if status != "All Statuses" {
            sql.push_str(&format!(" AND hr.status = ${idx}"));
            binders.push(status.clone());
        }
    }

    sql.push_str(" ORDER BY hr.block ASC, hr.room_no ASC");

    let mut db_query = sqlx::query_as::<_, HostelRoomWithOccupancy>(&sql);
    for val in binders {
        db_query = db_query.bind(val);
    }

    db_query.fetch_all(pool).await
}

pub async fn find_room_by_id(pool: &PgPool, id: Uuid) -> Result<Option<HostelRoom>, sqlx::Error> {
    sqlx::query_as::<_, HostelRoom>(
        "SELECT id, room_no, block, floor, capacity, room_type, fee_per_term::float8 AS fee_per_term, status, remarks, created_at, updated_at FROM hostel_rooms WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn find_room_by_no(pool: &PgPool, room_no: &str) -> Result<Option<Uuid>, sqlx::Error> {
    let row = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM hostel_rooms WHERE UPPER(TRIM(room_no)) = UPPER(TRIM($1))"
    )
    .bind(room_no)
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

pub async fn upsert_room(
    pool: &PgPool,
    room_no: &str,
    block: &str,
    floor: &str,
    capacity: i32,
    room_type: &str,
    fee_per_term: f64,
    status: &str,
    remarks: &str,
) -> Result<HostelRoom, sqlx::Error> {
    sqlx::query_as::<_, HostelRoom>(
        r#"
        INSERT INTO hostel_rooms (room_no, block, floor, capacity, room_type, fee_per_term, status, remarks)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        ON CONFLICT (room_no) DO UPDATE
        SET block = EXCLUDED.block,
            floor = EXCLUDED.floor,
            capacity = EXCLUDED.capacity,
            room_type = EXCLUDED.room_type,
            fee_per_term = EXCLUDED.fee_per_term,
            status = EXCLUDED.status,
            remarks = EXCLUDED.remarks,
            updated_at = NOW()
        RETURNING id, room_no, block, floor, capacity, room_type, fee_per_term::float8 AS fee_per_term, status, remarks, created_at, updated_at
        "#
    )
    .bind(room_no)
    .bind(block)
    .bind(floor)
    .bind(capacity)
    .bind(room_type)
    .bind(fee_per_term)
    .bind(status)
    .bind(remarks)
    .fetch_one(pool)
    .await
}

pub async fn update_room(
    pool: &PgPool,
    room: &HostelRoom,
) -> Result<HostelRoom, sqlx::Error> {
    sqlx::query_as::<_, HostelRoom>(
        r#"
        UPDATE hostel_rooms
        SET room_no = $1, block = $2, floor = $3, capacity = $4, room_type = $5, fee_per_term = $6, status = $7, remarks = $8, updated_at = NOW()
        WHERE id = $9
        RETURNING id, room_no, block, floor, capacity, room_type, fee_per_term::float8 AS fee_per_term, status, remarks, created_at, updated_at
        "#
    )
    .bind(&room.room_no)
    .bind(&room.block)
    .bind(&room.floor)
    .bind(room.capacity)
    .bind(&room.room_type)
    .bind(room.fee_per_term)
    .bind(&room.status)
    .bind(&room.remarks)
    .bind(room.id)
    .fetch_one(pool)
    .await
}

pub async fn delete_room(pool: &PgPool, id: Uuid) -> Result<u64, sqlx::Error> {
    let res = sqlx::query("DELETE FROM hostel_rooms WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(res.rows_affected())
}

// ─── Students Data Access ─────────────────────────────────────────────────────

pub async fn find_students(
    pool: &PgPool,
    q: &GetHostelStudentsQuery,
    limit: i64,
    offset: i64,
) -> Result<Vec<HostelStudentWithDetails>, sqlx::Error> {
    let mut sql = r#"
        SELECT hs.id, hs.student_id, hs.room_no, hs.bed_no, hs.check_in_date,
               hs.fee_amount::float8 AS fee_amount, hs.status, hs.emergency_contact, hs.remarks,
               hs.created_at, hs.updated_at,
               COALESCE(s.name, hs.student_id) AS student_name,
               s.class_name AS class_name,
               s.course_name AS course_name,
               COUNT(*) OVER()::int AS total_count
        FROM hostel_students hs
        LEFT JOIN students s ON LOWER(TRIM(s.student_id)) = LOWER(TRIM(hs.student_id))
        WHERE 1=1
    "#
    .to_string();

    let mut binders = Vec::new();
    let mut idx = 1usize;

    if let Some(ref search) = q.search {
        let term = format!("%{}%", search.trim());
        sql.push_str(&format!(" AND (COALESCE(s.name, '') ILIKE ${idx} OR hs.student_id ILIKE ${idx} OR hs.room_no ILIKE ${idx} OR COALESCE(hs.bed_no, '') ILIKE ${idx})"));
        binders.push(term);
        idx += 1;
    }

    if let Some(ref room_no) = q.room_no {
        if room_no != "All Rooms" {
            sql.push_str(&format!(" AND hs.room_no = ${idx}"));
            binders.push(room_no.clone());
            idx += 1;
        }
    }

    if let Some(ref status) = q.status {
        if status != "All Statuses" {
            sql.push_str(&format!(" AND hs.status = ${idx}"));
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

    sql.push_str(" ORDER BY COALESCE(s.name, hs.student_id) ASC");
    sql.push_str(&format!(" LIMIT ${idx} OFFSET ${}", idx + 1));

    let mut db_query = sqlx::query_as::<_, HostelStudentWithDetails>(&sql);
    for val in binders {
        db_query = db_query.bind(val);
    }
    db_query = db_query.bind(limit).bind(offset);

    db_query.fetch_all(pool).await
}

pub async fn find_student_by_id(pool: &PgPool, id: Uuid) -> Result<Option<HostelStudent>, sqlx::Error> {
    sqlx::query_as::<_, HostelStudent>(
        "SELECT id, student_id, room_no, bed_no, check_in_date, fee_amount::float8 AS fee_amount, status, emergency_contact, remarks, created_at, updated_at FROM hostel_students WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn find_master_student_id(pool: &PgPool, student_id: &str) -> Result<Option<String>, sqlx::Error> {
    let row = sqlx::query_scalar::<_, String>(
        "SELECT student_id FROM students WHERE LOWER(TRIM(student_id)) = LOWER(TRIM($1))"
    )
    .bind(student_id)
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

pub async fn upsert_student(
    pool: &PgPool,
    student_id: &str,
    room_no: &str,
    bed_no: &str,
    check_in_date: NaiveDate,
    fee_amount: f64,
    status: &str,
    emergency_contact: &str,
    remarks: &str,
) -> Result<HostelStudent, sqlx::Error> {
    sqlx::query_as::<_, HostelStudent>(
        r#"
        INSERT INTO hostel_students (student_id, room_no, bed_no, check_in_date, fee_amount, status, emergency_contact, remarks)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        ON CONFLICT (student_id) DO UPDATE
        SET room_no = EXCLUDED.room_no,
            bed_no = EXCLUDED.bed_no,
            check_in_date = EXCLUDED.check_in_date,
            fee_amount = EXCLUDED.fee_amount,
            status = EXCLUDED.status,
            emergency_contact = EXCLUDED.emergency_contact,
            remarks = EXCLUDED.remarks,
            updated_at = NOW()
        RETURNING id, student_id, room_no, bed_no, check_in_date, fee_amount::float8 AS fee_amount, status, emergency_contact, remarks, created_at, updated_at
        "#
    )
    .bind(student_id)
    .bind(room_no)
    .bind(bed_no)
    .bind(check_in_date)
    .bind(fee_amount)
    .bind(status)
    .bind(emergency_contact)
    .bind(remarks)
    .fetch_one(pool)
    .await
}

pub async fn update_student(
    pool: &PgPool,
    student: &HostelStudent,
) -> Result<HostelStudent, sqlx::Error> {
    sqlx::query_as::<_, HostelStudent>(
        r#"
        UPDATE hostel_students
        SET room_no = $1, bed_no = $2, check_in_date = $3, fee_amount = $4, status = $5, emergency_contact = $6, remarks = $7, updated_at = NOW()
        WHERE id = $8
        RETURNING id, student_id, room_no, bed_no, check_in_date, fee_amount::float8 AS fee_amount, status, emergency_contact, remarks, created_at, updated_at
        "#
    )
    .bind(&student.room_no)
    .bind(&student.bed_no)
    .bind(student.check_in_date)
    .bind(student.fee_amount)
    .bind(&student.status)
    .bind(&student.emergency_contact)
    .bind(&student.remarks)
    .bind(student.id)
    .fetch_one(pool)
    .await
}

pub async fn delete_student(pool: &PgPool, id: Uuid) -> Result<u64, sqlx::Error> {
    let res = sqlx::query("DELETE FROM hostel_students WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(res.rows_affected())
}
