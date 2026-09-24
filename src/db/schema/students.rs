// backend-rust/src/db/schema/students.rs
//! Students Master Registry Schema Blueprint

pub const CREATE_STUDENTS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS students (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    student_id VARCHAR(50) UNIQUE NOT NULL,
    name VARCHAR(150) NOT NULL,
    class_name VARCHAR(100) NOT NULL,
    course_name VARCHAR(150),
    dob DATE NOT NULL,
    gender VARCHAR(20) DEFAULT 'Male',
    phone VARCHAR(30),
    email VARCHAR(255),
    father_name VARCHAR(150),
    mother_name VARCHAR(150),
    parent_phone VARCHAR(30),
    aadhar_no VARCHAR(50),
    admission_no VARCHAR(50),
    blood_group VARCHAR(20),
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    tuition_fee FLOAT8 DEFAULT 0.00,
    tuition_duration INT DEFAULT 1,
    tuition_start_date DATE,
    tuition_end_date DATE,
    transport_fee FLOAT8 DEFAULT 0.00,
    transport_duration INT DEFAULT 1,
    transport_start_date DATE,
    transport_end_date DATE,
    hostel_fee FLOAT8 DEFAULT 0.00,
    hostel_duration INT DEFAULT 1,
    hostel_start_date DATE,
    hostel_end_date DATE,
    reg_date DATE DEFAULT CURRENT_DATE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_students_student_id ON students (student_id);
CREATE INDEX IF NOT EXISTS idx_students_class_name ON students (class_name);
CREATE INDEX IF NOT EXISTS idx_students_course_name ON students (course_name);
CREATE INDEX IF NOT EXISTS idx_students_status ON students (status);
"#;
