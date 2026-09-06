# Finance & Fee Accounting APIs

**Base Prefix**: `/api/finance`

Handles fee invoicing, tuition/transport/hostel fee payments, payment mode reconciliation (UPI, Cash, Bank Transfer), and institute operational expenses.

---

### 1. Student Fee Records

#### List Fee Records
- **Method**: `GET`
- **Route**: `/api/finance/fees/:id_or_type` (e.g. `/api/finance/fees/all`, `/api/finance/fees/Tuition`, `/api/finance/fees/Hostel`, `/api/finance/fees/Transport`)
- **Auth**: Required (`admin`, `accountant`, `staff`)
- **Query Parameters**:
  - `student_id` (optional)
  - `class_name` (optional)
  - `course_name` (optional)
  - `status` (optional: `paid`, `partial`, `pending`, `overdue`)
  - `page`, `limit`
- **Response** (`200 OK`):
```json
{
  "status": "success",
  "data": [
    {
      "id": "fee-101-...",
      "student_id": "STU-2026-001",
      "student_name": "Aarav Sharma",
      "fee_type": "Tuition Fee - Term 1",
      "amount": 25000.00,
      "paid_amount": 25000.00,
      "due_date": "2026-09-10",
      "payment_date": "2026-09-02",
      "payment_mode": "UPI",
      "transaction_id": "UPI/2348729384",
      "status": "paid",
      "created_at": "2026-09-01T00:00:00Z"
    }
  ]
}
```

#### Record Fee Payment / Create Invoice
- **Method**: `POST`
- **Route**: `/api/finance/fees`
- **Request Body**:
```json
{
  "student_id": "STU-2026-002",
  "student_name": "Diya Patel",
  "fee_type": "Hostel Fee - Sept",
  "amount": 4500.00,
  "paid_amount": 4500.00,
  "due_date": "2026-09-15",
  "payment_date": "2026-09-05",
  "payment_mode": "Cash",
  "receipt_no": "REC-2026-094",
  "status": "paid"
}
```

#### Update Fee Record
- **Method**: `PUT`
- **Route**: `/api/finance/fees/:id`

#### Delete Fee Record
- **Method**: `DELETE`
- **Route**: `/api/finance/fees/:id`

---

### 2. Operational Expenses Ledger

#### List Expenses
- **Method**: `GET`
- **Route**: `/api/finance/expenses`
- **Query**: `category`, `date_from`, `date_to`, `page`, `limit`

#### Add Expense Record
- **Method**: `POST`
- **Route**: `/api/finance/expenses`
- **Request Body**:
```json
{
  "category": "Utilities",
  "title": "Campus High-Speed Fiber Internet",
  "amount": 8499.00,
  "expense_date": "2026-09-04",
  "payment_mode": "Bank Transfer",
  "reference_no": "TXN-90214",
  "description": "Monthly Airtel Broadband lease for computer labs"
}
```

#### Update / Delete Expense
- **Method**: `PUT` / `DELETE`
- **Route**: `/api/finance/expenses/:id`
