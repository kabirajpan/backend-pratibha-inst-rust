// backend-rust/src/db/schema/master.rs
//! Master Registries Schema Blueprint: Classes & Courses

pub const CREATE_CLASSES_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS classes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) UNIQUE NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_classes_lower_name ON classes (LOWER(TRIM(name)));
"#;

pub const CREATE_COURSES_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS courses (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(150) UNIQUE NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_courses_lower_name ON courses (LOWER(TRIM(name)));
"#;
