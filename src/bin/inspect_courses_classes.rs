use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");
    let pool = PgPoolOptions::new().max_connections(2).connect(&db_url).await?;

    let courses: Vec<(String,)> = sqlx::query_as("SELECT name FROM courses ORDER BY name")
        .fetch_all(&pool)
        .await?;

    println!("=== EXISTING COURSES IN DB ===");
    for (name,) in &courses {
        println!("- {}", name);
    }

    let classes: Vec<(String,)> = sqlx::query_as("SELECT name FROM classes ORDER BY name")
        .fetch_all(&pool)
        .await?;

    println!("\n=== EXISTING CLASSES IN DB ===");
    for (name,) in &classes {
        println!("- {}", name);
    }

    let rooms: Vec<(String,)> = sqlx::query_as("SELECT room_no FROM hostel_rooms ORDER BY room_no")
        .fetch_all(&pool)
        .await
        .unwrap_or_default();

    println!("\n=== EXISTING HOSTEL ROOMS IN DB ===");
    for (room_no,) in &rooms {
        println!("- {}", room_no);
    }

    let vehicles: Vec<(String,)> = sqlx::query_as("SELECT reg_no FROM vehicles ORDER BY reg_no")
        .fetch_all(&pool)
        .await
        .unwrap_or_default();

    println!("\n=== EXISTING VEHICLES IN DB ===");
    for (reg_no,) in &vehicles {
        println!("- {}", reg_no);
    }

    Ok(())
}
