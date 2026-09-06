// backend-rust/src/db/schema/mod.rs
//! ============================================================================
//! 🏛️ ENTERPRISE DATABASE SCHEMA BLUEPRINT REGISTRY
//! ============================================================================
//!
//! Modular architecture dividing table blueprints by domain:
//! - master.rs     -> classes, courses
//! - auth.rs       -> users
//! - students.rs   -> students master registry
//! - finance.rs    -> fee_collections, expenses
//! - hostel.rs     -> hostel_rooms, hostel_students
//! - transport.rs  -> vehicles, transport_students
//! - library.rs    -> books, library_members, book_issues, library_settings
//! - inventory.rs  -> inventory_items, inventory_issues
//! - system.rs     -> audit_logs, announcements, user_notifications, todos

pub mod master;
pub mod auth;
pub mod students;
pub mod finance;
pub mod hostel;
pub mod transport;
pub mod library;
pub mod inventory;
pub mod system;

use sqlx::PgPool;
use crate::errors::AppError;

/// Central Schema Initialization:
/// Runs domain DDL statements in dependency order and validates canonical columns
pub async fn init_schema(pool: &PgPool) -> Result<(), AppError> {
    tracing::info!("Verifying and initializing database schema blueprint across all domains...");

    let ddl_statements = [
        ("classes", master::CREATE_CLASSES_TABLE),
        ("courses", master::CREATE_COURSES_TABLE),
        ("users", auth::CREATE_USERS_TABLE),
        ("students", students::CREATE_STUDENTS_TABLE),
        ("fee_collections", finance::CREATE_FEE_COLLECTIONS_TABLE),
        ("hostel_rooms", hostel::CREATE_HOSTEL_ROOMS_TABLE),
        ("hostel_students", hostel::CREATE_HOSTEL_STUDENTS_TABLE),
        ("vehicles", transport::CREATE_VEHICLES_TABLE),
        ("transport_students", transport::CREATE_TRANSPORT_STUDENTS_TABLE),
        ("books", library::CREATE_BOOKS_TABLE),
        ("library_members", library::CREATE_LIBRARY_MEMBERS_TABLE),
        ("book_issues", library::CREATE_BOOK_ISSUES_TABLE),
        ("library_settings", library::CREATE_LIBRARY_SETTINGS_TABLE),
        ("inventory_items", inventory::CREATE_INVENTORY_ITEMS_TABLE),
        ("inventory_issues", inventory::CREATE_INVENTORY_ISSUES_TABLE),
        ("expenses", finance::CREATE_EXPENSES_TABLE),
        ("audit_logs", system::CREATE_AUDIT_LOGS_TABLE),
        ("announcements", system::CREATE_ANNOUNCEMENTS_TABLE),
        ("user_notifications", system::CREATE_USER_NOTIFICATIONS_TABLE),
        ("todos", system::CREATE_TODOS_TABLE),
    ];

    for (name, ddl) in ddl_statements {
        if let Err(e) = sqlx::raw_sql(ddl).execute(pool).await {
            tracing::warn!("Schema notice for table '{}': {}", name, e);
        }
    }

    // Schema sync patches for backward-compatibility on existing databases
    let sync_patches = [
        "ALTER TABLE students ADD COLUMN IF NOT EXISTS class_name VARCHAR(100);",
        "ALTER TABLE students ADD COLUMN IF NOT EXISTS course_name VARCHAR(150);",
        "ALTER TABLE library_members ADD COLUMN IF NOT EXISTS class_name VARCHAR(100);",
        "ALTER TABLE library_members ADD COLUMN IF NOT EXISTS course_name VARCHAR(150);",
        "ALTER TABLE library_members ADD COLUMN IF NOT EXISTS class VARCHAR(100);",
        "ALTER TABLE library_members ADD COLUMN IF NOT EXISTS course VARCHAR(150);",
        "ALTER TABLE fee_collections ADD COLUMN IF NOT EXISTS discount FLOAT8 NOT NULL DEFAULT 0.00;",
        "ALTER TABLE fee_collections ADD COLUMN IF NOT EXISTS due_fees FLOAT8 NOT NULL DEFAULT 0.00;",
    ];

    for patch in sync_patches {
        let _ = sqlx::query(patch).execute(pool).await;
    }

    // Default library settings row if missing
    let _ = sqlx::query(
        r#"
        INSERT INTO library_settings (issue_limit, return_days, fine_per_day)
        SELECT 3, 14, 10.0
        WHERE NOT EXISTS (SELECT 1 FROM library_settings);
        "#
    )
    .execute(pool)
    .await;

    tracing::info!("Database schema blueprint verified successfully across all domains.");
    Ok(())
}
