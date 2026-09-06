use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");
    let pool = PgPoolOptions::new().max_connections(2).connect(&db_url).await?;

    // Tuition fees joined with student register
    let rows: Vec<(String, String, Option<String>, Option<String>, Option<String>, Option<String>)> = sqlx::query_as(
        r#"
        SELECT 
            f.student_id,
            COALESCE(f.student_name, '—') AS fee_student_name,
            f.class_name   AS fee_class,
            f.course_name  AS fee_course,
            s.class_name   AS reg_class,
            s.course_name  AS reg_course
        FROM tuition_fees f
        LEFT JOIN students s ON LOWER(TRIM(f.student_id)) = LOWER(TRIM(s.student_id))
        ORDER BY f.student_id
        "#
    )
    .fetch_all(&pool)
    .await?;

    println!("{:<12} | {:<14} | {:<14} | {:<14} | {:<14} | MATCH?",
        "STUDENT_ID", "FEE CLASS", "REG CLASS", "FEE COURSE", "REG COURSE");
    println!("{}", "-".repeat(90));

    let mut all_match = true;
    for (sid, _name, fee_class, fee_course, reg_class, reg_course) in &rows {
        let fc = fee_class.as_deref().unwrap_or("—");
        let rc = reg_class.as_deref().unwrap_or("NOT IN REG");
        let fco = fee_course.as_deref().unwrap_or("—");
        let rco = reg_course.as_deref().unwrap_or("NOT IN REG");
        let class_ok = fc.trim().to_lowercase() == rc.trim().to_lowercase();
        let course_ok = fco.trim().to_lowercase() == rco.trim().to_lowercase();
        let matched = class_ok && course_ok;
        if !matched { all_match = false; }
        println!("{:<12} | {:<14} | {:<14} | {:<14} | {:<14} | {}",
            sid, fc, rc, fco, rco, if matched { "✅" } else { "❌ MISMATCH" });
    }

    println!("\nTotal fee records: {}", rows.len());
    if all_match { println!("✅ ALL MATCH"); } else { println!("❌ MISMATCHES FOUND"); }
    Ok(())
}
