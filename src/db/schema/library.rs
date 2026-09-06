// backend-rust/src/db/schema/library.rs
//! Library Schema Blueprint: Books, Members, Circulation Issues, Settings

pub const CREATE_BOOKS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS books (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title VARCHAR(255) NOT NULL,
    author VARCHAR(255) NOT NULL DEFAULT 'Unknown Author',
    category VARCHAR(100) NOT NULL DEFAULT 'General',
    type VARCHAR(50) NOT NULL DEFAULT 'book',
    acc_no VARCHAR(50),
    sl_no VARCHAR(50),
    isbn VARCHAR(50),
    publisher VARCHAR(150),
    year INT,
    volume VARCHAR(50),
    number_val VARCHAR(50),
    month VARCHAR(50),
    total_copies INT NOT NULL DEFAULT 1,
    available_copies INT NOT NULL DEFAULT 1,
    shelf_location VARCHAR(100) DEFAULT 'Shelf 1',
    rack_no VARCHAR(50) DEFAULT 'R-1',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_books_title ON books (title);
CREATE INDEX IF NOT EXISTS idx_books_acc_no ON books (acc_no);
"#;

pub const CREATE_LIBRARY_MEMBERS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS library_members (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    student_id VARCHAR(50) UNIQUE NOT NULL,
    name VARCHAR(150) NOT NULL,
    class_name VARCHAR(100) DEFAULT '—',
    course_name VARCHAR(150) DEFAULT '—',
    class VARCHAR(100),
    course VARCHAR(150),
    phone VARCHAR(30) DEFAULT '—',
    total_issued INT NOT NULL DEFAULT 0,
    currently_issued INT NOT NULL DEFAULT 0,
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    joined_date DATE NOT NULL DEFAULT CURRENT_DATE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_library_members_student_id ON library_members (student_id);
"#;

pub const CREATE_BOOK_ISSUES_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS book_issues (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    issue_no VARCHAR(100) UNIQUE NOT NULL,
    book_id UUID NOT NULL REFERENCES books(id) ON DELETE CASCADE,
    member_id UUID NOT NULL REFERENCES library_members(id) ON DELETE CASCADE,
    issue_date DATE NOT NULL DEFAULT CURRENT_DATE,
    due_date DATE NOT NULL,
    return_date DATE,
    status VARCHAR(50) NOT NULL DEFAULT 'issued',
    fine_amount FLOAT8 NOT NULL DEFAULT 0.00,
    fine_paid BOOLEAN NOT NULL DEFAULT false,
    remarks TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_book_issues_issue_no ON book_issues (issue_no);
CREATE INDEX IF NOT EXISTS idx_book_issues_status ON book_issues (status);
"#;

pub const CREATE_LIBRARY_SETTINGS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS library_settings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    issue_limit INT NOT NULL DEFAULT 3,
    return_days INT NOT NULL DEFAULT 14,
    fine_per_day FLOAT8 NOT NULL DEFAULT 10.0,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
"#;
