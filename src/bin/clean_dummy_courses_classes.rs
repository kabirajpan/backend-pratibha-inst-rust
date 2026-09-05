use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");
    let pool = PgPoolOptions::new().max_connections(2).connect(&db_url).await?;

    println!("Cleaning dummy courses from database...");
    let res_courses = sqlx::query("DELETE FROM courses WHERE name IN ('History', 'Math', 'Science')")
        .execute(&pool)
        .await?;
    println!("Deleted {} dummy courses ('History', 'Math', 'Science').", res_courses.rows_affected());

    let courses: Vec<(String,)> = sqlx::query_as("SELECT name FROM courses ORDER BY name")
        .fetch_all(&pool)
        .await?;

    println!("\n=== REMAINING OFFICIAL COURSES IN DB ===");
    for (name,) in &courses {
        println!("- {}", name);
    }

    let classes: Vec<(String,)> = sqlx::query_as("SELECT name FROM classes ORDER BY name")
        .fetch_all(&pool)
        .await?;

    println!("\n=== REMAINING OFFICIAL CLASSES IN DB ===");
    for (name,) in &classes {
        println!("- {}", name);
    }

    Ok(())
}
