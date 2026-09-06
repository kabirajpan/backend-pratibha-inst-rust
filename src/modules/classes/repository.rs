// backend-rust/src/modules/classes/repository.rs
//! Classes Data Access Layer (Repository)

use sqlx::PgPool;
use uuid::Uuid;
use super::models::Class;

/// Fetch all classes ordered alphabetically
pub async fn find_all(pool: &PgPool) -> Result<Vec<Class>, sqlx::Error> {
    sqlx::query_as::<_, Class>(
        "SELECT id, name, created_at, updated_at FROM classes ORDER BY name ASC"
    )
    .fetch_all(pool)
    .await
}

/// Find a class by its unique ID
pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Class>, sqlx::Error> {
    sqlx::query_as::<_, Class>(
        "SELECT id, name, created_at, updated_at FROM classes WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

/// Find an existing class by exact name (case-insensitive)
pub async fn find_by_name(pool: &PgPool, name: &str) -> Result<Option<Class>, sqlx::Error> {
    sqlx::query_as::<_, Class>(
        "SELECT id, name, created_at, updated_at FROM classes WHERE LOWER(TRIM(name)) = LOWER(TRIM($1))"
    )
    .bind(name)
    .fetch_optional(pool)
    .await
}

/// Check if another class with the same name exists (excluding the specified id)
pub async fn find_duplicate(pool: &PgPool, name: &str, exclude_id: Uuid) -> Result<Option<Uuid>, sqlx::Error> {
    let row = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM classes WHERE LOWER(TRIM(name)) = LOWER(TRIM($1)) AND id != $2"
    )
    .bind(name)
    .bind(exclude_id)
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

/// Insert a new class record into the master table
pub async fn create(pool: &PgPool, name: &str) -> Result<Class, sqlx::Error> {
    sqlx::query_as::<_, Class>(
        "INSERT INTO classes (name) VALUES ($1) RETURNING id, name, created_at, updated_at"
    )
    .bind(name)
    .fetch_one(pool)
    .await
}

/// Update a class by its ID
pub async fn update(pool: &PgPool, id: Uuid, name: &str) -> Result<Option<Class>, sqlx::Error> {
    sqlx::query_as::<_, Class>(
        "UPDATE classes SET name = $1, updated_at = NOW() WHERE id = $2 RETURNING id, name, created_at, updated_at"
    )
    .bind(name)
    .bind(id)
    .fetch_optional(pool)
    .await
}

/// Delete a class by its ID
pub async fn delete(pool: &PgPool, id: Uuid) -> Result<Option<Class>, sqlx::Error> {
    sqlx::query_as::<_, Class>(
        "DELETE FROM classes WHERE id = $1 RETURNING id, name, created_at, updated_at"
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}
