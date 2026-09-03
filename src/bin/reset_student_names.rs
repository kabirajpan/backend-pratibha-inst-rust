use sqlx::postgres::PgPoolOptions;
use dotenvy::dotenv;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new().max_connections(5).connect(&db_url).await?;

    let clean_students = vec![
        ("STU-2026-001", "Aarav Sharma", "class 10", "Science"),
        ("STU-2026-002", "Priya Patel", "class 9", "Math"),
        ("STU-2026-003", "Rohan Verma", "class 10", "History"),
        ("STU-2026-004", "Ananya Sen", "class 9", "Science"),
        ("STU-2026-005", "Devansh Joshi", "class 10", "Math"),
        ("STU-2026-006", "Isha Nair", "class 9", "History"),
        ("STU-2026-007", "Karan Singh", "class 10", "Science"),
        ("STU-2026-008", "Sneha Rao", "class 9", "Math"),
        ("STU-2026-009", "Aditya Gupta", "class 10", "History"),
        ("STU-2026-010", "Meera Das", "class 9", "Science"),
    ];

    for (id, name, cls, crs) in clean_students {
        let _ = sqlx::query("UPDATE students SET name = $1, class_name = $2, course_name = $3 WHERE TRIM(student_id) = $4 OR student_id ILIKE $4")
            .bind(name)
            .bind(cls)
            .bind(crs)
            .bind(id)
            .execute(&pool)
            .await;
    }

    println!("Successfully reset all student profiles (name, class, course) to clean 5/10 fresh values!");
    Ok(())
}
