// backend-rust/src/db.rs
pub mod schema;

use sqlx::postgres::PgPoolOptions;
use std::time::Duration;
use crate::config::Config;
use crate::errors::AppError;

pub type DbPool = sqlx::PgPool;

/// Initialize and configure the PostgreSQL connection pool
pub async fn create_pool(config: &Config) -> Result<DbPool, AppError> {
    let pool = PgPoolOptions::new()
        .max_connections(25)
        .min_connections(2)
        .idle_timeout(Duration::from_secs(120))
        .max_lifetime(Duration::from_secs(300))
        .acquire_timeout(Duration::from_secs(30))
        .connect(&config.database_url)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to connect to database: {}", e)))?;

    // Execute central schema blueprint initialization
    schema::init_schema(&pool).await?;

    Ok(pool)
}

/// Simple health check to verify database responsiveness
pub async fn check_health(pool: &DbPool) -> Result<(), AppError> {
    sqlx::query("SELECT 1")
        .execute(pool)
        .await
        .map_err(|e| AppError::Internal(format!("Database health check failed: {}", e)))?;
    Ok(())
}
