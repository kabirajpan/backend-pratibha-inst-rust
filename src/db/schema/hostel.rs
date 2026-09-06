// backend-rust/src/db/schema/hostel.rs
//! Hostel Schema Blueprint: Rooms & Student Allotments

pub const CREATE_HOSTEL_ROOMS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS hostel_rooms (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    room_no VARCHAR(50) UNIQUE NOT NULL,
    floor VARCHAR(50) NOT NULL DEFAULT 'Ground Floor',
    capacity INT NOT NULL DEFAULT 2,
    occupied INT NOT NULL DEFAULT 0,
    fee_per_month FLOAT8 NOT NULL DEFAULT 0.00,
    status VARCHAR(50) NOT NULL DEFAULT 'available',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_hostel_rooms_room_no ON hostel_rooms (room_no);
"#;

pub const CREATE_HOSTEL_STUDENTS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS hostel_students (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    student_id VARCHAR(50) UNIQUE NOT NULL,
    room_no VARCHAR(50) NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    join_date DATE NOT NULL DEFAULT CURRENT_DATE,
    leave_date DATE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_hostel_students_student_id ON hostel_students (student_id);
CREATE INDEX IF NOT EXISTS idx_hostel_students_room_no ON hostel_students (room_no);
"#;
