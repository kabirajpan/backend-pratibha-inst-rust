use sqlx::postgres::PgPoolOptions;
use std::time::Duration;
use crate::config::Config;
use crate::errors::AppError;

pub type DbPool = sqlx::PgPool;

pub async fn create_pool(config: &Config) -> Result<DbPool, AppError> {
    let pool = PgPoolOptions::new()
        .max_connections(15)
        .min_connections(8) // Keep baseline warm connections to avoid handshake latencies on active requests
        .idle_timeout(Duration::from_secs(600))
        .acquire_timeout(Duration::from_secs(30))
        .connect(&config.database_url)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to connect to database: {}", e)))?;

    Ok(pool)
}
