// backend-rust/src/modules/admin/repository.rs
//! Admin & Students Data Access Layer (Repository)

use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;
use super::models::{
    AdminUserItem, AuditLogItem, CreateStudentPayload, GetAuditLogsQuery,
    GetStudentsQuery, Student, StudentWithCount, UpdateStudentPayload,
};

// ─── Students Queries ────────────────────────────────────────────────────────

pub async fn find_students(
    pool: &PgPool,
    q: &GetStudentsQuery,
    limit: i32,
    offset: i32,
) -> Result<Vec<StudentWithCount>, sqlx::Error> {
    let mut sql = r#"
        SELECT s.id, s.student_id, s.name, s.class_name, s.email, s.phone, s.dob, s.status,
               s.gender, s.blood_group, s.father_name, s.mother_name, s.parent_phone,
               s.current_address, s.permanent_address, s.aadhar_no, s.bank_name,
               s.account_no, s.ifsc_code, s.admission_no, s.admission_date, s.session,
               s.course_name, s.year, s.photo_url, s.signature_url, s.created_at, s.updated_at,
               hs.room_no AS hostel_room, 
               hs.bed_no AS hostel_bed, 
               hs.fee_amount::float8 AS hostel_fee, 
               ts.vehicle_no AS transport_vehicle, 
               ts.route AS transport_route, 
               ts.fee_amount::float8 AS transport_fee, 
               COUNT(*) OVER()::int AS total_count 
        FROM students s
        LEFT JOIN hostel_students hs ON LOWER(TRIM(hs.student_id)) = LOWER(TRIM(s.student_id)) AND hs.status = 'active'
        LEFT JOIN transport_students ts ON LOWER(TRIM(ts.student_id)) = LOWER(TRIM(s.student_id)) AND ts.status = 'active'
        WHERE 1=1
    "#.to_string();

    let mut binders: Vec<String> = Vec::new();
    let mut idx = 1usize;

    if let Some(ref search) = q.search {
        let term = format!("%{}%", search);
        sql.push_str(&format!(" AND (s.name ILIKE ${idx} OR s.student_id ILIKE ${idx})"));
        binders.push(term);
        idx += 1;
    }

    if let Some(ref class_name) = q.class_name {
        let clean = class_name.trim();
        if !clean.is_empty() && clean.to_lowercase() != "all" {
            sql.push_str(&format!(" AND s.class_name = ${idx}"));
            binders.push(clean.to_string());
            idx += 1;
        }
    }

    if let Some(ref course_name) = q.course_name {
        let clean = course_name.trim();
        if !clean.is_empty() && clean.to_lowercase() != "all" {
            sql.push_str(&format!(" AND s.course_name = ${idx}"));
            binders.push(clean.to_string());
            idx += 1;
        }
    }

    if let Some(ref session) = q.session {
        sql.push_str(&format!(" AND s.session = ${idx}"));
        binders.push(session.clone());
        idx += 1;
    }

    match q.sort_by.as_deref() {
        Some("id") | Some("number") => sql.push_str(" ORDER BY s.student_id ASC"),
        Some("name") => sql.push_str(" ORDER BY s.name ASC"),
        _ => sql.push_str(" ORDER BY s.updated_at DESC, s.created_at DESC"),
    }
    sql.push_str(&format!(" LIMIT ${idx} OFFSET ${}", idx + 1));

    let mut db_query = sqlx::query_as::<_, StudentWithCount>(&sql);
    for val in binders {
        db_query = db_query.bind(val);
    }
    db_query = db_query.bind(limit).bind(offset);

    db_query.fetch_all(pool).await
}

pub async fn find_student_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Student>, sqlx::Error> {
    sqlx::query_as::<_, Student>("SELECT * FROM students WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn find_student_by_student_id(pool: &PgPool, student_id: &str) -> Result<Option<Uuid>, sqlx::Error> {
    let row = sqlx::query_scalar::<_, Uuid>("SELECT id FROM students WHERE LOWER(TRIM(student_id)) = LOWER(TRIM($1))")
        .bind(student_id)
        .fetch_optional(pool)
        .await?;

    Ok(row)
}

pub async fn insert_student(
    pool: &PgPool,
    p: &CreateStudentPayload,
    dob: NaiveDate,
    admission_date: Option<NaiveDate>,
) -> Result<Student, sqlx::Error> {
    let status = p.status.as_deref().unwrap_or("active");
    let year = p.year.as_deref().unwrap_or("1st Year");

    sqlx::query_as::<_, Student>(
        r#"
        INSERT INTO students (
            student_id, name, class_name, email, phone, dob, status,
            gender, blood_group, father_name, mother_name, parent_phone,
            current_address, permanent_address, aadhar_no, bank_name,
            account_no, ifsc_code, admission_no, admission_date, session,
            course_name, photo_url, signature_url, year
        )
        VALUES (
            $1, $2, $3, $4, $5, $6, $7,
            $8, $9, $10, $11, $12,
            $13, $14, $15, $16,
            $17, $18, $19, $20, $21,
            $22, $23, $24, $25
        )
        RETURNING *
        "#
    )
    .bind(p.student_id.trim())
    .bind(p.name.trim())
    .bind(p.class_name.as_deref().map(|s| s.trim()))
    .bind(p.email.as_deref().map(|s| s.trim()))
    .bind(p.phone.as_deref().map(|s| s.trim()))
    .bind(dob)
    .bind(status)
    .bind(&p.gender)
    .bind(&p.blood_group)
    .bind(&p.father_name)
    .bind(&p.mother_name)
    .bind(&p.parent_phone)
    .bind(&p.current_address)
    .bind(&p.permanent_address)
    .bind(&p.aadhar_no)
    .bind(&p.bank_name)
    .bind(&p.account_no)
    .bind(&p.ifsc_code)
    .bind(&p.admission_no)
    .bind(admission_date)
    .bind(&p.session)
    .bind(p.course_name.as_deref().map(|s| s.trim()))
    .bind(&p.photo_url)
    .bind(&p.signature_url)
    .bind(year)
    .fetch_one(pool)
    .await
}

pub async fn update_student(
    pool: &PgPool,
    id: Uuid,
    p: &UpdateStudentPayload,
    dob: Option<NaiveDate>,
    admission_date: Option<NaiveDate>,
    existing: &Student,
) -> Result<Option<Student>, sqlx::Error> {
    sqlx::query_as::<_, Student>(
        r#"
        UPDATE students SET
            student_id = COALESCE($1, student_id),
            name = COALESCE($2, name),
            class_name = COALESCE($3, class_name),
            email = COALESCE($4, email),
            phone = COALESCE($5, phone),
            dob = COALESCE($6, dob),
            status = COALESCE($7, status),
            gender = COALESCE($8, gender),
            blood_group = COALESCE($9, blood_group),
            father_name = COALESCE($10, father_name),
            mother_name = COALESCE($11, mother_name),
            parent_phone = COALESCE($12, parent_phone),
            current_address = COALESCE($13, current_address),
            permanent_address = COALESCE($14, permanent_address),
            aadhar_no = COALESCE($15, aadhar_no),
            bank_name = COALESCE($16, bank_name),
            account_no = COALESCE($17, account_no),
            ifsc_code = COALESCE($18, ifsc_code),
            admission_no = COALESCE($19, admission_no),
            admission_date = COALESCE($20, admission_date),
            session = COALESCE($21, session),
            course_name = COALESCE($22, course_name),
            photo_url = COALESCE($23, photo_url),
            signature_url = COALESCE($24, signature_url),
            year = COALESCE($25, year),
            updated_at = NOW()
        WHERE id = $26
        RETURNING *
        "#
    )
    .bind(p.student_id.as_deref().map(|s| s.trim()))
    .bind(p.name.as_deref().map(|s| s.trim()))
    .bind(p.class_name.as_deref().map(|s| s.trim()))
    .bind(p.email.as_deref().map(|s| s.trim()))
    .bind(p.phone.as_deref().map(|s| s.trim()))
    .bind(dob)
    .bind(&p.status)
    .bind(&p.gender)
    .bind(&p.blood_group)
    .bind(&p.father_name)
    .bind(&p.mother_name)
    .bind(&p.parent_phone)
    .bind(&p.current_address)
    .bind(&p.permanent_address)
    .bind(&p.aadhar_no)
    .bind(&p.bank_name)
    .bind(&p.account_no)
    .bind(&p.ifsc_code)
    .bind(&p.admission_no)
    .bind(admission_date)
    .bind(&p.session)
    .bind(p.course_name.as_deref().map(|s| s.trim()))
    .bind(&p.photo_url)
    .bind(&p.signature_url)
    .bind(p.year.as_deref().or(existing.year.as_deref()))
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn delete_student(pool: &PgPool, id: Uuid) -> Result<Option<Student>, sqlx::Error> {
    sqlx::query_as::<_, Student>("DELETE FROM students WHERE id = $1 RETURNING *")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn upsert_student_import(
    pool: &PgPool,
    s: &CreateStudentPayload,
    dob: NaiveDate,
    admission_date: Option<NaiveDate>,
) -> Result<Student, sqlx::Error> {
    let status = s.status.as_deref().unwrap_or("active");
    let year = s.year.as_deref().unwrap_or("1st Year");

    sqlx::query_as::<_, Student>(
        r#"
        INSERT INTO students (
            student_id, name, class_name, email, phone, dob, status,
            gender, blood_group, father_name, mother_name, parent_phone,
            current_address, permanent_address, aadhar_no, bank_name,
            account_no, ifsc_code, admission_no, admission_date, session,
            course_name, photo_url, signature_url, year
        )
        VALUES (
            $1, $2, $3, $4, $5, $6, $7,
            $8, $9, $10, $11, $12,
            $13, $14, $15, $16,
            $17, $18, $19, $20, $21,
            $22, $23, $24, $25
        )
        ON CONFLICT (student_id) DO UPDATE SET
            name = EXCLUDED.name,
            class_name = EXCLUDED.class_name,
            email = EXCLUDED.email,
            phone = EXCLUDED.phone,
            dob = EXCLUDED.dob,
            status = EXCLUDED.status,
            gender = EXCLUDED.gender,
            blood_group = EXCLUDED.blood_group,
            father_name = EXCLUDED.father_name,
            mother_name = EXCLUDED.mother_name,
            parent_phone = EXCLUDED.parent_phone,
            current_address = EXCLUDED.current_address,
            permanent_address = EXCLUDED.permanent_address,
            aadhar_no = EXCLUDED.aadhar_no,
            bank_name = EXCLUDED.bank_name,
            account_no = EXCLUDED.account_no,
            ifsc_code = EXCLUDED.ifsc_code,
            admission_no = EXCLUDED.admission_no,
            admission_date = EXCLUDED.admission_date,
            session = EXCLUDED.session,
            course_name = EXCLUDED.course_name,
            photo_url = EXCLUDED.photo_url,
            signature_url = EXCLUDED.signature_url,
            year = EXCLUDED.year,
            updated_at = NOW()
        RETURNING *
        "#
    )
    .bind(s.student_id.trim())
    .bind(s.name.trim())
    .bind(s.class_name.as_deref().map(|s| s.trim()))
    .bind(s.email.as_deref().map(|s| s.trim()))
    .bind(s.phone.as_deref().map(|s| s.trim()))
    .bind(dob)
    .bind(status)
    .bind(&s.gender)
    .bind(&s.blood_group)
    .bind(&s.father_name)
    .bind(&s.mother_name)
    .bind(&s.parent_phone)
    .bind(&s.current_address)
    .bind(&s.permanent_address)
    .bind(&s.aadhar_no)
    .bind(&s.bank_name)
    .bind(&s.account_no)
    .bind(&s.ifsc_code)
    .bind(&s.admission_no)
    .bind(admission_date)
    .bind(&s.session)
    .bind(s.course_name.as_deref().map(|s| s.trim()))
    .bind(&s.photo_url)
    .bind(&s.signature_url)
    .bind(year)
    .fetch_one(pool)
    .await
}

pub async fn upsert_user_account(
    pool: &PgPool,
    name: &str,
    email: &str,
    password_hash: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO users (name, email, password_hash, role)
        VALUES ($1, $2, $3, 'student')
        ON CONFLICT (email) DO UPDATE SET name = EXCLUDED.name
        "#
    )
    .bind(name)
    .bind(email)
    .bind(password_hash)
    .execute(pool)
    .await?;

    Ok(())
}

// ─── Users Queries ───────────────────────────────────────────────────────────

pub async fn find_all_users(pool: &PgPool) -> Result<Vec<AdminUserItem>, sqlx::Error> {
    let users = sqlx::query(
        "SELECT id, name, email, role, sub_role, is_active, created_at FROM users ORDER BY role ASC, name ASC"
    )
    .fetch_all(pool)
    .await?;

    let data = users
        .into_iter()
        .map(|row| {
            use sqlx::Row;
            AdminUserItem {
                id: row.get("id"),
                name: row.get("name"),
                email: row.get("email"),
                role: row.try_get("role").unwrap_or_default(),
                sub_role: row.try_get("sub_role").ok().flatten(),
                is_active: row.get("is_active"),
                created_at: row.get("created_at"),
            }
        })
        .collect();

    Ok(data)
}

pub async fn toggle_user_active(pool: &PgPool, id: Uuid) -> Result<Option<bool>, sqlx::Error> {
    let updated = sqlx::query(
        "UPDATE users SET is_active = NOT is_active WHERE id = $1 RETURNING is_active"
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(updated.map(|row| {
        use sqlx::Row;
        row.get("is_active")
    }))
}

pub async fn delete_user(pool: &PgPool, id: Uuid) -> Result<Option<Uuid>, sqlx::Error> {
    let deleted = sqlx::query_scalar::<_, Uuid>("DELETE FROM users WHERE id = $1 RETURNING id")
        .bind(id)
        .fetch_optional(pool)
        .await?;

    Ok(deleted)
}

// ─── Audit Logs Queries ──────────────────────────────────────────────────────

pub async fn find_audit_logs(
    pool: &PgPool,
    q: &GetAuditLogsQuery,
    limit: i64,
    offset: i64,
) -> Result<Vec<AuditLogItem>, sqlx::Error> {
    let mut sql = "SELECT id, user_name, role, action, module, details, timestamp FROM audit_logs WHERE 1=1".to_string();
    let mut binders: Vec<String> = Vec::new();
    let mut idx = 1usize;

    if let Some(ref m) = q.module {
        if m != "all" && !m.is_empty() {
            sql.push_str(&format!(" AND module = ${idx}"));
            binders.push(m.clone());
            idx += 1;
        }
    }

    if let Some(ref s) = q.search {
        if !s.is_empty() {
            let term = format!("%{}%", s);
            sql.push_str(&format!(" AND (details ILIKE ${idx} OR user_name ILIKE ${idx} OR action ILIKE ${idx})"));
            binders.push(term);
            idx += 1;
        }
    }

    sql.push_str(&format!(" ORDER BY timestamp DESC LIMIT ${idx} OFFSET ${}", idx + 1));

    let mut query = sqlx::query_as::<_, AuditLogItem>(&sql);
    for b in binders {
        query = query.bind(b);
    }
    query = query.bind(limit).bind(offset);

    query.fetch_all(pool).await
}
