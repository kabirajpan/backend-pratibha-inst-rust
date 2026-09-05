// src/bin/cleanup_test_members.rs
// Run with: cargo run --bin cleanup_test_members

use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");
    let pool = PgPoolOptions::new().max_connections(2).connect(&db_url).await?;

    println!("Clearing all library members from database...");

    let res = sqlx::query("TRUNCATE TABLE library_members CASCADE;")
        .execute(&pool)
        .await?;

    println!("Cleared all members cleanly.");
    Ok(())
}
