// backend-rust/src/modules/auth/repository.rs
//! Auth Data Access Layer (Repository)

use sqlx::PgPool;
use uuid::Uuid;
use super::models::{User, UserRole, UserSubRole};

/// Find user by email address
pub async fn find_by_email(pool: &PgPool, email: &str) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as::<_, User>(
        "SELECT id, name, email, password_hash, role, sub_role, is_active, created_at, updated_at FROM users WHERE LOWER(email) = LOWER($1)"
    )
    .bind(email)
    .fetch_optional(pool)
    .await
}

/// Find user by ID
pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as::<_, User>(
        "SELECT id, name, email, password_hash, role, sub_role, is_active, created_at, updated_at FROM users WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

/// Check if email is pre-registered in student master
pub async fn find_student_by_email(pool: &PgPool, email: &str) -> Result<Option<String>, sqlx::Error> {
    let row = sqlx::query_scalar::<_, String>(
        "SELECT student_id FROM students WHERE LOWER(email) = LOWER($1)"
    )
    .bind(email)
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

/// Insert new user into database
pub async fn create_user(
    pool: &PgPool,
    name: &str,
    email: &str,
    password_hash: &str,
    role: &UserRole,
    sub_role: Option<&UserSubRole>,
) -> Result<User, sqlx::Error> {
    sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (name, email, password_hash, role, sub_role)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, name, email, password_hash, role, sub_role, is_active, created_at, updated_at
        "#
    )
    .bind(name)
    .bind(email)
    .bind(password_hash)
    .bind(role)
    .bind(sub_role)
    .fetch_one(pool)
    .await
}

/// Fetch all staff users
pub async fn find_all_staff(pool: &PgPool) -> Result<Vec<User>, sqlx::Error> {
    sqlx::query_as::<_, User>(
        "SELECT id, name, email, password_hash, role, sub_role, is_active, created_at, updated_at FROM users WHERE role = 'staff' ORDER BY name ASC"
    )
    .fetch_all(pool)
    .await
}

/// Update user password hash
pub async fn update_password_hash(pool: &PgPool, user_id: Uuid, new_hash: &str) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE users SET password_hash = $1, updated_at = NOW() WHERE id = $2")
        .bind(new_hash)
        .bind(user_id)
        .execute(pool)
        .await?;

    Ok(())
}
