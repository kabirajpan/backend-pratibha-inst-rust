use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");
    let pool = PgPoolOptions::new().max_connections(2).connect(&db_url).await?;

    let admin_user: Option<(String, String)> = sqlx::query_as("SELECT email, role::text FROM users LIMIT 1")
        .fetch_optional(&pool)
        .await?;

    if let Some((email, role)) = admin_user {
        println!("User found - Email: {}, Role: {}", email, role);
    } else {
        println!("No users found in users table.");
    }

    Ok(())
}
