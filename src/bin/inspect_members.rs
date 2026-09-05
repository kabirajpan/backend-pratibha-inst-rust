use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");
    let pool = PgPoolOptions::new().max_connections(2).connect(&db_url).await?;

    let rows: Vec<(String, String, String, Option<String>, String, String)> = sqlx::query_as(
        "SELECT id::text, student_id, name, class, status, created_at::text FROM library_members ORDER BY created_at ASC"
    )
    .fetch_all(&pool)
    .await?;

    println!("Total members in DB: {}", rows.len());
    for (id, sid, name, class_val, status, created_at) in rows {
        println!("ID: {}, Student ID: {}, Name: {}, Class: {:?}, Status: {}, Created: {}", 
            id, sid, name, class_val, status, created_at);
    }

    Ok(())
}
