use sqlx::postgres::PgPoolOptions;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");
    let pool = PgPoolOptions::new().connect(&db_url).await?;

    let res = sqlx::query("DELETE FROM fee_collections WHERE fee_type = 'hostel'")
        .execute(&pool)
        .await?;

    println!("Successfully deleted {} hostel fee records from fee_collections.", res.rows_affected());
    Ok(())
}
