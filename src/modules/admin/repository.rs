// backend-rust/src/modules/admin/repository.rs
//! Admin & Students Data Access Layer (Repository)

use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;
use super::models::{
    AdminUserItem, AuditLogItem, CreateStudentPayload, GetAuditLogsQuery,
    GetStudentsQuery, Student, StudentCredentialRecord, StudentWithCount, SystemSettings,
    UpdateStudentPayload, UpdateSystemSettingsPayload,
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
               s.tuition_fee::float8 AS tuition_fee,
               s.tuition_duration::int AS tuition_duration,
               s.tuition_start_date,
               s.tuition_end_date,
               hs.room_no AS hostel_room, 
               hs.bed_no AS hostel_bed, 
               COALESCE(s.hostel_fee, hs.fee_amount)::float8 AS hostel_fee,
               s.hostel_duration::int AS hostel_duration,
               s.hostel_start_date,
               s.hostel_end_date,
               ts.vehicle_no AS transport_vehicle, 
               ts.route AS transport_route, 
               COALESCE(s.transport_fee, ts.fee_amount)::float8 AS transport_fee,
               s.transport_duration::int AS transport_duration,
               s.transport_start_date,
               s.transport_end_date,
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
    tuition_start_date: Option<NaiveDate>,
    tuition_end_date: Option<NaiveDate>,
    transport_start_date: Option<NaiveDate>,
    transport_end_date: Option<NaiveDate>,
    hostel_start_date: Option<NaiveDate>,
    hostel_end_date: Option<NaiveDate>,
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
            course_name, photo_url, signature_url, year,
            tuition_fee, tuition_duration, tuition_start_date, tuition_end_date,
            transport_fee, transport_duration, transport_start_date, transport_end_date,
            hostel_fee, hostel_duration, hostel_start_date, hostel_end_date
        )
        VALUES (
            $1, $2, $3, $4, $5, $6, $7,
            $8, $9, $10, $11, $12,
            $13, $14, $15, $16,
            $17, $18, $19, $20, $21,
            $22, $23, $24, $25,
            $26, $27, $28, $29,
            $30, $31, $32, $33,
            $34, $35, $36, $37
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
    .bind(p.tuition_fee)
    .bind(p.tuition_duration)
    .bind(tuition_start_date)
    .bind(tuition_end_date)
    .bind(p.transport_fee)
    .bind(p.transport_duration)
    .bind(transport_start_date)
    .bind(transport_end_date)
    .bind(p.hostel_fee)
    .bind(p.hostel_duration)
    .bind(hostel_start_date)
    .bind(hostel_end_date)
    .fetch_one(pool)
    .await
}

pub async fn update_student(
    pool: &PgPool,
    id: Uuid,
    p: &UpdateStudentPayload,
    dob: Option<NaiveDate>,
    admission_date: Option<NaiveDate>,
    tuition_start_date: Option<NaiveDate>,
    tuition_end_date: Option<NaiveDate>,
    transport_start_date: Option<NaiveDate>,
    transport_end_date: Option<NaiveDate>,
    hostel_start_date: Option<NaiveDate>,
    hostel_end_date: Option<NaiveDate>,
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
            tuition_fee = COALESCE($26, tuition_fee),
            tuition_duration = COALESCE($27, tuition_duration),
            tuition_start_date = COALESCE($28, tuition_start_date),
            tuition_end_date = COALESCE($29, tuition_end_date),
            transport_fee = COALESCE($30, transport_fee),
            transport_duration = COALESCE($31, transport_duration),
            transport_start_date = COALESCE($32, transport_start_date),
            transport_end_date = COALESCE($33, transport_end_date),
            hostel_fee = COALESCE($34, hostel_fee),
            hostel_duration = COALESCE($35, hostel_duration),
            hostel_start_date = COALESCE($36, hostel_start_date),
            hostel_end_date = COALESCE($37, hostel_end_date),
            updated_at = NOW()
        WHERE id = $38
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
    .bind(p.tuition_fee)
    .bind(p.tuition_duration)
    .bind(tuition_start_date)
    .bind(tuition_end_date)
    .bind(p.transport_fee)
    .bind(p.transport_duration)
    .bind(transport_start_date)
    .bind(transport_end_date)
    .bind(p.hostel_fee)
    .bind(p.hostel_duration)
    .bind(hostel_start_date)
    .bind(hostel_end_date)
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
        ON CONFLICT (email) DO UPDATE SET name = EXCLUDED.name, password_hash = EXCLUDED.password_hash
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

// ─── System Settings Queries ─────────────────────────────────────────────────

pub async fn find_system_settings(pool: &PgPool) -> Result<SystemSettings, sqlx::Error> {
    sqlx::query_as::<_, SystemSettings>(
        r#"
        SELECT id, email_service_enabled, student_welcome_email_enabled,
               fee_receipt_email_enabled, staff_welcome_email_enabled,
               announcement_email_enabled, sms_service_enabled,
               fee_receipt_sms_enabled, whatsapp_service_enabled, updated_at
        FROM system_settings
        WHERE id = 'global'
        LIMIT 1
        "#
    )
    .fetch_one(pool)
    .await
}

pub async fn update_system_settings(
    pool: &PgPool,
    payload: &UpdateSystemSettingsPayload,
) -> Result<SystemSettings, sqlx::Error> {
    let current = match find_system_settings(pool).await {
        Ok(s) => s,
        Err(_) => SystemSettings {
            id: "global".to_string(),
            email_service_enabled: true,
            student_welcome_email_enabled: true,
            fee_receipt_email_enabled: true,
            staff_welcome_email_enabled: true,
            announcement_email_enabled: true,
            sms_service_enabled: true,
            fee_receipt_sms_enabled: true,
            whatsapp_service_enabled: true,
            updated_at: chrono::Utc::now(),
        },
    };

    let email_enabled = payload.email_service_enabled.unwrap_or(current.email_service_enabled);
    let student_welcome = payload.student_welcome_email_enabled.unwrap_or(current.student_welcome_email_enabled);
    let fee_receipt = payload.fee_receipt_email_enabled.unwrap_or(current.fee_receipt_email_enabled);
    let staff_welcome = payload.staff_welcome_email_enabled.unwrap_or(current.staff_welcome_email_enabled);
    let announcement = payload.announcement_email_enabled.unwrap_or(current.announcement_email_enabled);
    let sms_enabled = payload.sms_service_enabled.unwrap_or(current.sms_service_enabled);
    let fee_receipt_sms = payload.fee_receipt_sms_enabled.unwrap_or(current.fee_receipt_sms_enabled);
    let whatsapp_enabled = payload.whatsapp_service_enabled.unwrap_or(current.whatsapp_service_enabled);

    sqlx::query_as::<_, SystemSettings>(
        r#"
        INSERT INTO system_settings (
            id, email_service_enabled, student_welcome_email_enabled,
            fee_receipt_email_enabled, staff_welcome_email_enabled,
            announcement_email_enabled, sms_service_enabled,
            fee_receipt_sms_enabled, whatsapp_service_enabled, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW())
        ON CONFLICT (id) DO UPDATE SET
            email_service_enabled = EXCLUDED.email_service_enabled,
            student_welcome_email_enabled = EXCLUDED.student_welcome_email_enabled,
            fee_receipt_email_enabled = EXCLUDED.fee_receipt_email_enabled,
            staff_welcome_email_enabled = EXCLUDED.staff_welcome_email_enabled,
            announcement_email_enabled = EXCLUDED.announcement_email_enabled,
            sms_service_enabled = EXCLUDED.sms_service_enabled,
            fee_receipt_sms_enabled = EXCLUDED.fee_receipt_sms_enabled,
            whatsapp_service_enabled = EXCLUDED.whatsapp_service_enabled,
            updated_at = NOW()
        RETURNING id, email_service_enabled, student_welcome_email_enabled,
                  fee_receipt_email_enabled, staff_welcome_email_enabled,
                  announcement_email_enabled, sms_service_enabled,
                  fee_receipt_sms_enabled, whatsapp_service_enabled, updated_at
        "#
    )
    .bind("global")
    .bind(email_enabled)
    .bind(student_welcome)
    .bind(fee_receipt)
    .bind(staff_welcome)
    .bind(announcement)
    .bind(sms_enabled)
    .bind(fee_receipt_sms)
    .bind(whatsapp_enabled)
    .fetch_one(pool)
    .await
}

pub async fn find_students_for_credentials(
    pool: &PgPool,
    student_ids: &[String],
) -> Result<Vec<StudentCredentialRecord>, sqlx::Error> {
    sqlx::query_as::<_, StudentCredentialRecord>(
        r#"
        SELECT student_id, name, email, dob, class_name
        FROM students
        WHERE student_id = ANY($1)
        ORDER BY student_id ASC
        "#,
    )
    .bind(student_ids)
    .fetch_all(pool)
    .await
}

