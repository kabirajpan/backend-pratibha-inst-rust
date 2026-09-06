// backend-rust/src/db/schema/transport.rs
//! Transport Schema Blueprint: Fleet Vehicles & Student Transport Allotments

pub const CREATE_VEHICLES_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS vehicles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    reg_no VARCHAR(50) UNIQUE NOT NULL,
    type VARCHAR(100) NOT NULL DEFAULT 'Bus',
    capacity INT NOT NULL DEFAULT 40,
    driver VARCHAR(100) NOT NULL DEFAULT 'Unassigned',
    route VARCHAR(150) NOT NULL DEFAULT 'Route 1',
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    remarks TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_vehicles_reg_no ON vehicles (reg_no);
"#;

pub const CREATE_TRANSPORT_STUDENTS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS transport_students (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    student_id VARCHAR(50) UNIQUE NOT NULL,
    vehicle_no VARCHAR(50),
    route VARCHAR(150),
    pickup_point VARCHAR(150),
    fee_amount FLOAT8 NOT NULL DEFAULT 0.00,
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    remarks TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_transport_students_student_id ON transport_students (student_id);
CREATE INDEX IF NOT EXISTS idx_transport_students_vehicle_no ON transport_students (vehicle_no);
"#;
