# Library Management APIs

**Base Prefix**: `/api/library`

Comprehensive endpoints powering library cataloging, barcode/accession numbering, member registration, book checkouts & returns, fine calculations, and institutional loan policies.

---

### 1. Library Overview & Statistics

#### Dashboard Stats
- **Method**: `GET`
- **Route**: `/api/library/stats`
- **Response** (`200 OK`):
```json
{
  "status": "success",
  "data": {
    "total_books": 12500,
    "available_books": 11820,
    "issued_books": 680,
    "total_members": 850,
    "overdue_issues": 34,
    "unpaid_fines": 1450.00
  }
}
```

#### Recent Library Activity
- **Method**: `GET`
- **Route**: `/api/library/activity`
- **Response** (`200 OK`): Recent issue/return transaction stream.

---

### 2. Books Catalog

#### List Books
- **Method**: `GET`
- **Route**: `/api/library/books`
- **Query Parameters**: `search`, `category`, `author`, `status`, `page`, `limit`

#### Get Single Book
- **Method**: `GET`
- **Route**: `/api/library/books/:id`

#### Add Book
- **Method**: `POST`
- **Route**: `/api/library/books`
- **Request Body**:
```json
{
  "title": "Concepts of Physics - Vol 1",
  "author": "H. C. Verma",
  "isbn": "978-8177091878",
  "category": "Physics",
  "publisher": "Bharati Bhawan",
  "edition": "2024",
  "copies_total": 10,
  "shelf_location": "Bay 3, Shelf B"
}
```

#### Update / Delete Book
- **Method**: `PUT` / `DELETE`
- **Route**: `/api/library/books/:id`

#### Bulk Import Books
- **Method**: `POST`
- **Route**: `/api/library/books/import`
- **Request Body**: Array of book records.

---

### 3. Library Members

#### List Members
- **Method**: `GET`
- **Route**: `/api/library/members`
- **Query Parameters**: `search`, `member_type` (`student`, `staff`), `status`

#### Register Member
- **Method**: `POST`
- **Route**: `/api/library/members`
- **Request Body**:
```json
{
  "member_id": "LIB-STU-001",
  "name": "Aarav Sharma",
  "member_type": "student",
  "reference_id": "STU-2026-001",
  "email": "aarav.sharma@example.com",
  "phone": "+919876543210",
  "max_allowed_books": 4
}
```

#### Update Member
- **Method**: `PUT`
- **Route**: `/api/library/members/:id`

#### Bulk Import Members
- **Method**: `POST`
- **Route**: `/api/library/members/import`

---

### 4. Book Issues, Returns & Fines

#### List Book Issues
- **Method**: `GET`
- **Route**: `/api/library/issues`
- **Query**: `status` (`issued`, `returned`, `overdue`), `member_id`

#### Issue Book
- **Method**: `POST`
- **Route**: `/api/library/issues`
- **Request Body**:
```json
{
  "book_id": "book-123-...",
  "member_id": "member-456-...",
  "due_date": "2026-09-20"
}
```

#### Return Book
- **Method**: `POST`
- **Route**: `/api/library/issues/:id/return`
- **Request Body**:
```json
{
  "return_date": "2026-09-18",
  "fine_amount": 0.00,
  "condition": "good"
}
```

#### Update Issue Fine
- **Method**: `PUT`
- **Route**: `/api/library/issues/:id/fine`
- **Request Body**:
```json
{
  "fine_amount": 50.00,
  "fine_status": "paid"
}
```

---

### 5. Library Policy Settings

#### Get Settings
- **Method**: `GET`
- **Route**: `/api/library/settings`

#### Update Settings
- **Method**: `PUT`
- **Route**: `/api/library/settings`
- **Request Body**:
```json
{
  "max_issue_days": 14,
  "max_books_per_student": 4,
  "max_books_per_faculty": 10,
  "fine_per_day": 5.00
}
```
