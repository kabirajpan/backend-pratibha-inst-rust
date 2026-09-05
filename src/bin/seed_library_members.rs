// src/bin/seed_library_members.rs
// Run with: cargo run --bin seed_library_members

use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");
    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&db_url)
        .await?;

    println!("Fetching available classes from master table...");

    let test_members = vec![
        (
            "B2026001",
            "Aarav Sharma",
            "class 10",
            "History",
            "9810087654",
            "active",
        ),
        (
            "B2026002",
            "Mansi Mandavi",
            "class 10",
            "Math",
            "9823456789",
            "active",
        ),
        (
            "B2026003",
            "Ritu Sen",
            "class 8",
            "Science",
            "9834567890",
            "active",
        ),
        (
            "B2026004",
            "Komal Preet",
            "class 9",
            "History",
            "9845678901",
            "active",
        ),
        (
            "B2026005",
            "Priya Patel",
            "class 8",
            "Math",
            "9856789012",
            "active",
        ),
        (
            "B2026006",
            "Rohan Verma",
            "class 10",
            "Science",
            "9867890123",
            "active",
        ),
        (
            "B2026007",
            "Ananya Roy",
            "class 8",
            "History",
            "9878901234",
            "active",
        ),
        (
            "B2026008",
            "Harsh Vardhan",
            "class 9",
            "Math",
            "9889012345",
            "active",
        ),
        (
            "B2026009",
            "Sneha Reddy",
            "class 9",
            "Science",
            "9890123456",
            "active",
        ),
        (
            "B2026010",
            "Devansh Joshi",
            "class 10",
            "History",
            "9901234567",
            "active",
        ),
    ];

    let mut seeded = 0;
    for (student_id, name, class_name, course_name, phone, status) in test_members.into_iter() {
        let res = sqlx::query(
            r#"
            INSERT INTO library_members (id, student_id, name, class, course, phone, status, created_at, updated_at)
            VALUES (gen_random_uuid(), $1, $2, $3, $4, $5, $6, NOW(), NOW())
            ON CONFLICT (student_id) DO UPDATE
            SET name = EXCLUDED.name, class = EXCLUDED.class, course = EXCLUDED.course, phone = EXCLUDED.phone, status = EXCLUDED.status, updated_at = NOW()
            "#
        )
        .bind(student_id)
        .bind(name)
        .bind(class_name)
        .bind(course_name)
        .bind(phone)
        .bind(status)
        .execute(&pool)
        .await;

        match res {
            Ok(_) => seeded += 1,
            Err(e) => eprintln!("Error seeding member {}: {:?}", student_id, e),
        }
    }

    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM library_members")
        .fetch_one(&pool)
        .await?;

    println!(
        "SUCCESS: Seeded/updated {} members. Total members in DB: {}",
        seeded, count.0
    );
    Ok(())
}
