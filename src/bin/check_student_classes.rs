use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");
    let pool = PgPoolOptions::new().max_connections(2).connect(&db_url).await?;

    let rows: Vec<(String, String, Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT student_id, name, class_name, course_name FROM students ORDER BY student_id"
    )
    .fetch_all(&pool)
    .await?;

    println!("{:<12} | {:<30} | {:<12} | {:<15}", "STUDENT_ID", "NAME", "CLASS", "COURSE");
    println!("{}", "-".repeat(75));
    for (sid, name, class, course) in &rows {
        println!("{:<12} | {:<30} | {:<12} | {:<15}",
            sid,
            name,
            class.as_deref().unwrap_or("—"),
            course.as_deref().unwrap_or("—")
        );
    }
    println!("\nTotal: {} students", rows.len());
    Ok(())
}
