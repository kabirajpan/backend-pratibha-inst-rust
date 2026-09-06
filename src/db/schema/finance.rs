// backend-rust/src/db/schema/finance.rs
//! Finance Schema Blueprint: Fee Collections & General Expenses

pub const CREATE_FEE_COLLECTIONS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS fee_collections (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    student_id VARCHAR(50) NOT NULL,
    fee_type VARCHAR(50) NOT NULL,
    room VARCHAR(100) DEFAULT '—',
    bus_route VARCHAR(100) DEFAULT '—',
    bus_no VARCHAR(50) DEFAULT '—',
    receipt_book_no VARCHAR(50) DEFAULT '—',
    receipt_no VARCHAR(100) UNIQUE NOT NULL,
    receipt_date DATE NOT NULL,
    payment_date DATE NOT NULL,
    amount FLOAT8 NOT NULL DEFAULT 0.00,
    utr_no VARCHAR(100) DEFAULT '—',
    payment_mode VARCHAR(50) NOT NULL DEFAULT 'Online',
    due_fees FLOAT8 NOT NULL DEFAULT 0.00,
    remarks TEXT DEFAULT '—',
    discount FLOAT8 NOT NULL DEFAULT 0.00,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_fee_collections_student_id ON fee_collections (student_id);
CREATE INDEX IF NOT EXISTS idx_fee_collections_fee_type ON fee_collections (fee_type);
CREATE INDEX IF NOT EXISTS idx_fee_collections_receipt_no ON fee_collections (receipt_no);
CREATE INDEX IF NOT EXISTS idx_fee_collections_payment_date ON fee_collections (payment_date);
"#;

pub const CREATE_EXPENSES_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS expenses (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ref_no VARCHAR(100) UNIQUE NOT NULL,
    description TEXT NOT NULL,
    amount FLOAT8 NOT NULL DEFAULT 0.00,
    category VARCHAR(100) NOT NULL DEFAULT 'general',
    date DATE NOT NULL DEFAULT CURRENT_DATE,
    payment_mode VARCHAR(50) NOT NULL DEFAULT 'Online',
    remarks TEXT,
    utr VARCHAR(100) DEFAULT '—',
    receipt VARCHAR(100) DEFAULT '—',
    party_name VARCHAR(150) DEFAULT '—',
    spent_by VARCHAR(150) DEFAULT 'Staff User',
    voucher_no VARCHAR(100) DEFAULT '—',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_expenses_category ON expenses (category);
CREATE INDEX IF NOT EXISTS idx_expenses_date ON expenses (date);
"#;
