// src/bin/clear_students.rs
// Run with: cargo run --bin clear_students
use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new().max_connections(2).connect(&db_url).await?;

    let count_before: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM students").fetch_one(&pool).await?;
    println!("Students count before clear: {}", count_before.0);

    sqlx::query("TRUNCATE TABLE students CASCADE").execute(&pool).await?;

    let count_after: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM students").fetch_one(&pool).await?;
    println!("Students count after clear: {}", count_after.0);

    println!("SUCCESS: Student register DB cleared.");
    Ok(())
}
