use sqlx::postgres::PgPoolOptions;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");
    let pool = PgPoolOptions::new().connect(&db_url).await?;

    println!("Connected to database...");

    // 1. Create tables if not exist & truncate classes, courses, students, hostel/transport/library/fee tables
    let _ = sqlx::query(
        "CREATE TABLE IF NOT EXISTS courses (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            name VARCHAR UNIQUE NOT NULL,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            updated_at TIMESTAMPTZ DEFAULT NOW()
        );"
    )
    .execute(&pool)
    .await;

    sqlx::query("TRUNCATE TABLE classes, courses, students, hostel_students, transport_students, library_members, fee_collections CASCADE;")
        .execute(&pool)
        .await?;
    println!("Cleared classes, courses, students, facility allotments, and fee records.");

    // No hardcoded dummy courses or classes seeded — all courses and classes must be created explicitly by administrators
    println!("Database tables reset.");

    // 2. Delete ONLY student user accounts, keeping Admin/Staff credentials safe
    let user_res = sqlx::query("DELETE FROM users WHERE role::text = 'student'")
        .execute(&pool)
        .await?;
    println!("Cleaned {} student accounts from users table (Admin & Staff credentials preserved).", user_res.rows_affected());

    // 3. Verify counts
    let count_students: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM students").fetch_one(&pool).await?;
    let count_classes: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM classes").fetch_one(&pool).await?;
    let count_courses: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM courses").fetch_one(&pool).await?;
    let count_users: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users").fetch_one(&pool).await?;

    println!("\n--- DATABASE CLEANUP STATUS ---");
    println!("Total Students: {}", count_students.0);
    println!("Total Classes: {}", count_classes.0);
    println!("Total Courses: {}", count_courses.0);
    println!("Preserved User Credentials (Admins/Staff): {}", count_users.0);
    println!("--------------------------------\n");

    Ok(())
}
