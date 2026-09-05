// src/bin/clear_issue_records.rs
// Run with: cargo run --bin clear_issue_records
use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");
    let pool = PgPoolOptions::new().max_connections(2).connect(&db_url).await?;

    let issues_before: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM book_issues").fetch_one(&pool).await?;
    let lib_fees_before: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM fee_collections WHERE fee_type = 'library'").fetch_one(&pool).await?;
    println!("Book issues count before clear: {}", issues_before.0);
    println!("Library fee collections before clear: {}", lib_fees_before.0);

    sqlx::query("TRUNCATE TABLE book_issues CASCADE").execute(&pool).await?;
    sqlx::query("DELETE FROM fee_collections WHERE fee_type = 'library'").execute(&pool).await?;

    let issues_after: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM book_issues").fetch_one(&pool).await?;
    let lib_fees_after: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM fee_collections WHERE fee_type = 'library'").fetch_one(&pool).await?;
    println!("Book issues count after clear: {}", issues_after.0);
    println!("Library fee collections after clear: {}", lib_fees_after.0);

    println!("SUCCESS: All book issue records and library fee collections cleared from database.");
    Ok(())
}
