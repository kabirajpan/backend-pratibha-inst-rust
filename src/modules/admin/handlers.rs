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
use crate::modules::auth::models::{ApiResponse, ApiMessageResponse, UserRole};
use crate::utils::password::hash_password;
use super::models::*;

// ─── STUDENTS ─────────────────────────────────────────────

pub async fn get_students(
    State(state): State<AppState>,
    _auth_user: AuthUser, // any authenticated user (staff or admin) can list students
    Query(q): Query<GetStudentsQuery>,
) -> Result<impl IntoResponse, AppError> {
    let page = q.page.unwrap_or(1).max(1);
    let limit = q.limit.unwrap_or(50).max(1);
    let offset = (page - 1) * limit;

    let mut sql = "SELECT *, COUNT(*) OVER()::int AS total_count FROM students WHERE 1=1".to_string();
    let mut binders: Vec<String> = Vec::new();
    let mut idx = 1usize;

    if let Some(ref search) = q.search {
        let term = format!("%{}%", search);
        sql.push_str(&format!(" AND (name ILIKE ${idx} OR student_id ILIKE ${idx})"));
        binders.push(term);
        idx += 1;
    }

    if let Some(ref class_name) = q.class_name {
        sql.push_str(&format!(" AND class_name = ${idx}"));
        binders.push(class_name.clone());
        idx += 1;
    }

    if let Some(ref session) = q.session {
        sql.push_str(&format!(" AND session = ${idx}"));
        binders.push(session.clone());
        idx += 1;
    }

    sql.push_str(" ORDER BY student_id ASC");
    sql.push_str(&format!(" LIMIT ${idx} OFFSET ${}", idx + 1));

    let mut db_query = sqlx::query_as::<_, StudentWithCount>(&sql);
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

pub async fn get_student(
    State(state): State<AppState>,
    _auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let student = sqlx::query_as::<_, Student>("SELECT * FROM students WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.db)
        .await?;

    let student = student.ok_or_else(|| AppError::NotFound("Student not found".to_string()))?;

    Ok(Json(ApiResponse { success: true, data: student }))
}

pub async fn create_student(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(payload): Json<CreateStudentPayload>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize(&[UserRole::Admin])?;
    payload.validate()?;

    // Check duplicate student_id
    let existing = sqlx::query("SELECT id FROM students WHERE student_id = $1")
        .bind(&payload.student_id)
        .fetch_optional(&state.db)
        .await?;

    if existing.is_some() {
        return Err(AppError::Conflict(format!("Student with ID {} is already registered", payload.student_id)));
    }

    let dob = chrono::NaiveDate::parse_from_str(&payload.dob, "%Y-%m-%d")
        .map_err(|_| AppError::BadRequest("Invalid dob format".to_string()))?;

    let admission_date = match &payload.admission_date {
        Some(d) if !d.is_empty() => Some(
            chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d")
                .map_err(|_| AppError::BadRequest("Invalid admission_date format".to_string()))?
        ),
        _ => None,
    };

    let student = sqlx::query_as::<_, Student>(
        r#"
        INSERT INTO students (
            student_id, name, class_name, email, phone, dob, status,
            father_name, mother_name, parent_phone,
            current_address, permanent_address,
            aadhar_no, bank_name, account_no, ifsc_code,
            admission_no, admission_date, session, course_name,
            blood_group, gender, photo_url, signature_url
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7,
            $8, $9, $10,
            $11, $12,
            $13, $14, $15, $16,
            $17, $18, $19, $20,
            $21, $22, $23, $24
        ) RETURNING *
        "#
    )
    .bind(&payload.student_id)
    .bind(&payload.name)
    .bind(&payload.class_name)
    .bind(&payload.email)
    .bind(&payload.phone)
    .bind(dob)
    .bind(payload.status.as_deref().unwrap_or("active"))
    .bind(&payload.father_name)
    .bind(&payload.mother_name)
    .bind(&payload.parent_phone)
    .bind(&payload.current_address)
    .bind(&payload.permanent_address)
    .bind(&payload.aadhar_no)
    .bind(&payload.bank_name)
    .bind(&payload.account_no)
    .bind(&payload.ifsc_code)
    .bind(&payload.admission_no)
    .bind(admission_date)
    .bind(&payload.session)
    .bind(&payload.course_name)
    .bind(&payload.blood_group)
    .bind(&payload.gender)
    .bind(&payload.photo_url)
    .bind(&payload.signature_url)
    .fetch_one(&state.db)
    .await?;

    // Auto-create user account if email is provided
    // Password pattern: first 2 initials (uppercase) + DDMMYYYY from DOB
    let mut default_password: Option<String> = None;

    if let Some(ref email) = payload.email {
        if !email.is_empty() {
            let first_name: String = payload.name.trim()
                .split_whitespace()
                .next()
                .unwrap_or("XX")
                .chars()
                .filter(|c| c.is_alphabetic())
                .collect();
            let initials = format!("{:X>2}", &first_name.to_uppercase()[..first_name.len().min(2)]);
            let dob_part = format!("{}{}{}", dob.format("%d"), dob.format("%m"), dob.format("%Y"));
            let raw_password = format!("{}{}", initials, dob_part);
            let password_hash = hash_password(&raw_password)?;
            default_password = Some(raw_password);

            sqlx::query(
                r#"
                INSERT INTO users (name, email, password_hash, role)
                VALUES ($1, $2, $3, 'student')
                ON CONFLICT (email) DO UPDATE SET name = EXCLUDED.name
                "#
            )
            .bind(&payload.name)
            .bind(email)
            .bind(&password_hash)
            .execute(&state.db)
            .await?;
        }
    }

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "success": true,
            "data": {
                "id": student.id,
                "student_id": student.student_id,
                "name": student.name,
                "class_name": student.class_name,
                "email": student.email,
                "phone": student.phone,
                "dob": student.dob,
                "status": student.status,
                "defaultPassword": default_password,
                "created_at": student.created_at
            }
        })),
    ))
}

pub async fn edit_student(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateStudentPayload>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize(&[UserRole::Admin])?;
    payload.validate()?;

    // Fetch existing record
    let existing = sqlx::query_as::<_, Student>("SELECT * FROM students WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.db)
        .await?;
    let existing = existing.ok_or_else(|| AppError::NotFound("Student not found".to_string()))?;

    // Check duplicate student_id if changing it
    if let Some(ref new_sid) = payload.student_id {
        let dup = sqlx::query("SELECT id FROM students WHERE student_id = $1 AND id != $2")
            .bind(new_sid)
            .bind(id)
            .fetch_optional(&state.db)
            .await?;
        if dup.is_some() {
            return Err(AppError::Conflict("Student ID already assigned to another student".to_string()));
        }
    }

    let admission_date: Option<chrono::NaiveDate> = match &payload.admission_date {
        Some(d) if !d.is_empty() => Some(
            chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d")
                .map_err(|_| AppError::BadRequest("Invalid admission_date format".to_string()))?
        ),
        _ => existing.admission_date,
    };
    let dob: Option<chrono::NaiveDate> = match &payload.dob {
        Some(d) if !d.is_empty() => Some(
            chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d")
                .map_err(|_| AppError::BadRequest("Invalid dob format".to_string()))?
        ),
        _ => existing.dob,
    };

    let student = sqlx::query_as::<_, Student>(
        r#"
        UPDATE students
        SET
            student_id        = COALESCE($2,  student_id),
            name              = COALESCE($3,  name),
            class_name        = COALESCE($4,  class_name),
            email             = COALESCE($5,  email),
            phone             = COALESCE($6,  phone),
            dob               = COALESCE($7,  dob),
            status            = COALESCE($8,  status),
            father_name       = COALESCE($9,  father_name),
            mother_name       = COALESCE($10, mother_name),
            parent_phone      = COALESCE($11, parent_phone),
            current_address   = COALESCE($12, current_address),
            permanent_address = COALESCE($13, permanent_address),
            aadhar_no         = COALESCE($14, aadhar_no),
            bank_name         = COALESCE($15, bank_name),
            account_no        = COALESCE($16, account_no),
            ifsc_code         = COALESCE($17, ifsc_code),
            admission_no      = COALESCE($18, admission_no),
            admission_date    = $19,
            session           = COALESCE($20, session),
            course_name       = COALESCE($21, course_name),
            blood_group       = COALESCE($22, blood_group),
            gender            = COALESCE($23, gender),
            photo_url         = COALESCE($24, photo_url),
            signature_url     = COALESCE($25, signature_url),
            updated_at        = now()
        WHERE id = $1
        RETURNING *
        "#
    )
    .bind(id)
    .bind(&payload.student_id)
    .bind(&payload.name)
    .bind(&payload.class_name)
    .bind(&payload.email)
    .bind(&payload.phone)
    .bind(dob)
    .bind(&payload.status)
    .bind(&payload.father_name)
    .bind(&payload.mother_name)
    .bind(&payload.parent_phone)
    .bind(&payload.current_address)
    .bind(&payload.permanent_address)
    .bind(&payload.aadhar_no)
    .bind(&payload.bank_name)
    .bind(&payload.account_no)
    .bind(&payload.ifsc_code)
    .bind(&payload.admission_no)
    .bind(admission_date)
    .bind(&payload.session)
    .bind(&payload.course_name)
    .bind(&payload.blood_group)
    .bind(&payload.gender)
    .bind(&payload.photo_url)
    .bind(&payload.signature_url)
    .fetch_one(&state.db)
    .await?;

    // Sync email change to users table if email changed
    if let Some(ref new_email) = payload.email {
        if let Some(ref old_email) = existing.email {
            if new_email != old_email {
                sqlx::query("UPDATE users SET email = $1 WHERE email = $2 AND role = 'student'")
                    .bind(new_email)
                    .bind(old_email)
                    .execute(&state.db)
                    .await?;
            }
        }
    }

    Ok(Json(ApiResponse { success: true, data: student }))
}

pub async fn remove_student(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize(&[UserRole::Admin])?;

    let deleted = sqlx::query_as::<_, Student>("DELETE FROM students WHERE id = $1 RETURNING *")
        .bind(id)
        .fetch_optional(&state.db)
        .await?;

    deleted.ok_or_else(|| AppError::NotFound("Student not found".to_string()))?;

    Ok(Json(ApiMessageResponse {
        success: true,
        message: "Student deleted".to_string(),
    }))
}

pub async fn import_students(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(payload): Json<ImportStudentsPayload>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize(&[UserRole::Admin])?;

    if payload.students.is_empty() {
        return Err(AppError::BadRequest("students array must not be empty".to_string()));
    }

    let mut results = Vec::new();

    for s in payload.students {
        s.validate()?;

        // Check if already exists and upsert
        let existing = sqlx::query_as::<_, Student>(
            "SELECT * FROM students WHERE student_id = $1"
        )
        .bind(&s.student_id)
        .fetch_optional(&state.db)
        .await?;

        let student = if let Some(existing) = existing {
            // Update
            let dob = match chrono::NaiveDate::parse_from_str(&s.dob, "%Y-%m-%d") {
                Ok(d) => Some(d),
                Err(_) => existing.dob,
            };
            let admission_date = match &s.admission_date {
                Some(d) if !d.is_empty() => chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok(),
                _ => existing.admission_date,
            };

            sqlx::query_as::<_, Student>(
                r#"
                UPDATE students
                SET name = $2, class_name = COALESCE($3, class_name), email = COALESCE($4, email),
                    phone = COALESCE($5, phone), dob = $6, status = COALESCE($7, status),
                    father_name = COALESCE($8, father_name), mother_name = COALESCE($9, mother_name),
                    parent_phone = COALESCE($10, parent_phone), session = COALESCE($11, session),
                    course_name = COALESCE($12, course_name), admission_no = COALESCE($13, admission_no),
                    admission_date = $14, updated_at = now()
                WHERE student_id = $1
                RETURNING *
                "#
            )
            .bind(&s.student_id)
            .bind(&s.name)
            .bind(&s.class_name)
            .bind(&s.email)
            .bind(&s.phone)
            .bind(dob)
            .bind(&s.status)
            .bind(&s.father_name)
            .bind(&s.mother_name)
            .bind(&s.parent_phone)
            .bind(&s.session)
            .bind(&s.course_name)
            .bind(&s.admission_no)
            .bind(admission_date)
            .fetch_one(&state.db)
            .await?
        } else {
            // Create new
            let dob = chrono::NaiveDate::parse_from_str(&s.dob, "%Y-%m-%d")
                .map_err(|_| AppError::BadRequest(format!("Invalid dob for student {}", s.student_id)))?;
            let admission_date = match &s.admission_date {
                Some(d) if !d.is_empty() => chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok(),
                _ => None,
            };

            sqlx::query_as::<_, Student>(
                r#"
                INSERT INTO students (
                    student_id, name, class_name, email, phone, dob, status,
                    father_name, mother_name, parent_phone,
                    current_address, permanent_address,
                    aadhar_no, bank_name, account_no, ifsc_code,
                    admission_no, admission_date, session, course_name,
                    blood_group, gender, photo_url, signature_url
                ) VALUES (
                    $1, $2, $3, $4, $5, $6, $7,
                    $8, $9, $10, $11, $12, $13, $14, $15, $16,
                    $17, $18, $19, $20, $21, $22, $23, $24
                ) RETURNING *
                "#
            )
            .bind(&s.student_id)
            .bind(&s.name)
            .bind(&s.class_name)
            .bind(&s.email)
            .bind(&s.phone)
            .bind(dob)
            .bind(s.status.as_deref().unwrap_or("active"))
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
            .bind(&s.course_name)
            .bind(&s.blood_group)
            .bind(&s.gender)
            .bind(&s.photo_url)
            .bind(&s.signature_url)
            .fetch_one(&state.db)
            .await?
        };

        // Auto-create user account if email provided
        if let Some(ref email) = student.email {
            if !email.is_empty() {
                if let Some(dob) = student.dob {
                    let first_name: String = student.name.trim()
                        .split_whitespace()
                        .next()
                        .unwrap_or("XX")
                        .chars()
                        .filter(|c| c.is_alphabetic())
                        .collect();
                    let initials = format!("{:X>2}", &first_name.to_uppercase()[..first_name.len().min(2)]);
                    let dob_part = format!("{}{}{}", dob.format("%d"), dob.format("%m"), dob.format("%Y"));
                    let raw_password = format!("{}{}", initials, dob_part);
                    if let Ok(password_hash) = hash_password(&raw_password) {
                        let _ = sqlx::query(
                            r#"
                            INSERT INTO users (name, email, password_hash, role)
                            VALUES ($1, $2, $3, 'student')
                            ON CONFLICT (email) DO UPDATE SET name = EXCLUDED.name
                            "#
                        )
                        .bind(&student.name)
                        .bind(email)
                        .bind(&password_hash)
                        .execute(&state.db)
                        .await;
                    }
                }
            }
        }

        results.push(student);
    }

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse { success: true, data: results }),
    ))
}

// ─── STAFF MANAGEMENT ─────────────────────────────────────

pub async fn get_all_users(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize(&[UserRole::Admin])?;

    let users = sqlx::query(
        "SELECT id, name, email, role, sub_role, is_active, created_at FROM users ORDER BY role ASC, name ASC"
    )
    .fetch_all(&state.db)
    .await?;

    let data: Vec<serde_json::Value> = users
        .iter()
        .map(|row| {
            use sqlx::Row;
            let id: Uuid = row.get("id");
            let name: String = row.get("name");
            let email: String = row.get("email");
            let role: String = row.try_get("role").unwrap_or_default();
            let sub_role: Option<String> = row.try_get("sub_role").ok().flatten();
            let is_active: bool = row.get("is_active");
            let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
            json!({
                "id": id,
                "name": name,
                "email": email,
                "role": role,
                "sub_role": sub_role,
                "is_active": is_active,
                "created_at": created_at
            })
        })
        .collect();

    Ok(Json(ApiResponse { success: true, data }))
}

pub async fn toggle_user_active(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize(&[UserRole::Admin])?;

    // Prevent self-deactivation
    if id == auth_user.id {
        return Err(AppError::BadRequest("Cannot deactivate your own account".to_string()));
    }

    let updated = sqlx::query(
        "UPDATE users SET is_active = NOT is_active WHERE id = $1 RETURNING id, name, email, role, is_active"
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?;

    if updated.is_none() {
        return Err(AppError::NotFound("User not found".to_string()));
    }

    let row = updated.unwrap();
    use sqlx::Row;
    let is_active: bool = row.get("is_active");

    Ok(Json(json!({
        "success": true,
        "data": {
            "id": id,
            "is_active": is_active
        }
    })))
}

pub async fn delete_user(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize(&[UserRole::Admin])?;

    if id == auth_user.id {
        return Err(AppError::BadRequest("Cannot delete your own account".to_string()));
    }

    let deleted = sqlx::query("DELETE FROM users WHERE id = $1 RETURNING id")
        .bind(id)
        .fetch_optional(&state.db)
        .await?;

    if deleted.is_none() {
        return Err(AppError::NotFound("User not found".to_string()));
    }

    Ok(Json(ApiMessageResponse {
        success: true,
        message: "User deleted".to_string(),
    }))
}
