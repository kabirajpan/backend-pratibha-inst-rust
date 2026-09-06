use sqlx::postgres::PgPoolOptions;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");
    let pool = PgPoolOptions::new().connect(&db_url).await?;

    let t_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM transport_expenses")
        .fetch_one(&pool)
        .await?;

    let e_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM expenses")
        .fetch_one(&pool)
        .await?;

    println!("Before cleanup: transport_expenses = {}, expenses = {}", t_count.0, e_count.0);

    let res_t = sqlx::query("DELETE FROM transport_expenses")
        .execute(&pool)
        .await?;

    let res_e = sqlx::query("DELETE FROM expenses")
        .execute(&pool)
        .await?;

    println!("Deleted {} records from transport_expenses", res_t.rows_affected());
    println!("Deleted {} records from expenses", res_e.rows_affected());

    let t_after: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM transport_expenses")
        .fetch_one(&pool)
        .await?;

    let e_after: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM expenses")
        .fetch_one(&pool)
        .await?;

    println!("After cleanup: transport_expenses = {}, expenses = {}", t_after.0, e_after.0);

    Ok(())
}
