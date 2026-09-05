use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::env;
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    role: String,
    exp: usize,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");
    let jwt_secret = env::var("JWT_ACCESS_SECRET").expect("JWT_ACCESS_SECRET must be set");
    let pool = PgPoolOptions::new().max_connections(2).connect(&db_url).await?;

    let user: (String, String) = sqlx::query_as("SELECT id::text, role FROM users LIMIT 1")
        .fetch_one(&pool)
        .await?;

    let claims = Claims {
        sub: user.0,
        role: user.1,
        exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp() as usize,
    };

    let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(jwt_secret.as_bytes()))?;
    println!("Generated JWT Token successfully.");

    let client = reqwest::Client::new();
    
    // 1. Fetch all students (no params)
    let res1: serde_json::Value = client.get("http://localhost:5000/api/students")
        .bearer_auth(&token)
        .send()
        .await?
        .json()
        .await?;
    println!("1. GET /api/students (No params): data count = {}", res1["data"].as_array().map(|a| a.len()).unwrap_or(0));

    // 2. Fetch with class_name=all and course_name=all
    let res2: serde_json::Value = client.get("http://localhost:5000/api/students?class_name=all&course_name=all")
        .bearer_auth(&token)
        .send()
        .await?
        .json()
        .await?;
    println!("2. GET /api/students?class_name=all&course_name=all: data count = {}", res2["data"].as_array().map(|a| a.len()).unwrap_or(0));

    // 3. Fetch with empty string class_name=&course_name=
    let res3: serde_json::Value = client.get("http://localhost:5000/api/students?class_name=&course_name=")
        .bearer_auth(&token)
        .send()
        .await?
        .json()
        .await?;
    println!("3. GET /api/students?class_name=&course_name=: data count = {}", res3["data"].as_array().map(|a| a.len()).unwrap_or(0));

    Ok(())
}
