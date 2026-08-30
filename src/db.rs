use sqlx::postgres::PgPoolOptions;
use std::time::Duration;
use crate::config::Config;
use crate::errors::AppError;

pub type DbPool = sqlx::PgPool;

pub async fn create_pool(config: &Config) -> Result<DbPool, AppError> {
    let pool = PgPoolOptions::new()
        .max_connections(15)
        .min_connections(0)
        .idle_timeout(Duration::from_secs(120))
        .max_lifetime(Duration::from_secs(300))
        .acquire_timeout(Duration::from_secs(30))
        .connect(&config.database_url)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to connect to database: {}", e)))?;

    Ok(pool)
}
