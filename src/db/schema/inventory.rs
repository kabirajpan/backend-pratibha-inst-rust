// backend-rust/src/db/schema/inventory.rs
//! Inventory Schema Blueprint: Items Catalog & Issue Transactions

pub const CREATE_INVENTORY_ITEMS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS inventory_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(150) NOT NULL,
    category VARCHAR(100) NOT NULL DEFAULT 'General',
    quantity INT NOT NULL DEFAULT 0,
    unit VARCHAR(50) NOT NULL DEFAULT 'pcs',
    unit_price FLOAT8 NOT NULL DEFAULT 0.00,
    supplier VARCHAR(150) DEFAULT 'General Supplier',
    min_stock INT NOT NULL DEFAULT 5,
    location VARCHAR(100) DEFAULT 'Store Room',
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_inventory_items_name ON inventory_items (name);
"#;

pub const CREATE_INVENTORY_ISSUES_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS inventory_issues (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    item_id UUID NOT NULL REFERENCES inventory_items(id) ON DELETE CASCADE,
    issued_to VARCHAR(150) NOT NULL,
    quantity INT NOT NULL DEFAULT 1,
    purpose TEXT DEFAULT 'Departmental Use',
    issue_date DATE NOT NULL DEFAULT CURRENT_DATE,
    return_date DATE,
    status VARCHAR(50) NOT NULL DEFAULT 'issued',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
"#;
