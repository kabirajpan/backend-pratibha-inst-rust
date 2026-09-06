// backend-rust/src/modules/courses/repository.rs
//! Courses Data Access Layer (Repository)

use sqlx::PgPool;
use uuid::Uuid;
use super::models::Course;

/// Fetch all courses ordered alphabetically
pub async fn find_all(pool: &PgPool) -> Result<Vec<Course>, sqlx::Error> {
    sqlx::query_as::<_, Course>(
        "SELECT id, name, created_at, updated_at FROM courses ORDER BY name ASC"
    )
    .fetch_all(pool)
    .await
}

/// Find a course by its unique ID
pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Course>, sqlx::Error> {
    sqlx::query_as::<_, Course>(
        "SELECT id, name, created_at, updated_at FROM courses WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

/// Find a course by exact name (case-insensitive)
pub async fn find_by_name(pool: &PgPool, name: &str) -> Result<Option<Course>, sqlx::Error> {
    sqlx::query_as::<_, Course>(
        "SELECT id, name, created_at, updated_at FROM courses WHERE LOWER(TRIM(name)) = LOWER(TRIM($1))"
    )
    .bind(name)
    .fetch_optional(pool)
    .await
}

/// Check if another course with the same name exists (excluding the specified id)
pub async fn find_duplicate(pool: &PgPool, name: &str, exclude_id: Uuid) -> Result<Option<Uuid>, sqlx::Error> {
    let row = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM courses WHERE LOWER(TRIM(name)) = LOWER(TRIM($1)) AND id != $2"
    )
    .bind(name)
    .bind(exclude_id)
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

/// Insert a new course record into the master table
pub async fn create(pool: &PgPool, name: &str) -> Result<Course, sqlx::Error> {
    sqlx::query_as::<_, Course>(
        "INSERT INTO courses (name) VALUES ($1) RETURNING id, name, created_at, updated_at"
    )
    .bind(name)
    .fetch_one(pool)
    .await
}

/// Update a course by its ID
pub async fn update(pool: &PgPool, id: Uuid, name: &str) -> Result<Option<Course>, sqlx::Error> {
    sqlx::query_as::<_, Course>(
        "UPDATE courses SET name = $1, updated_at = NOW() WHERE id = $2 RETURNING id, name, created_at, updated_at"
    )
    .bind(name)
    .bind(id)
    .fetch_optional(pool)
    .await
}

/// Delete a course by its ID
pub async fn delete(pool: &PgPool, id: Uuid) -> Result<Option<Course>, sqlx::Error> {
    sqlx::query_as::<_, Course>(
        "DELETE FROM courses WHERE id = $1 RETURNING id, name, created_at, updated_at"
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}
