# Students Directory & Admissions APIs

**Base Prefix**: `/api/students`

Manages core student demographic profiles, academic enrollment info, parent contacts, status transitions, and bulk import pipelines.

---

### Endpoints

#### 1. List Students (Filtered & Paginated)
- **Method**: `GET`
- **Route**: `/api/students`
- **Auth**: Required (`Bearer <accessToken>`)
- **Query Parameters**:
  - `page` (optional, default: 1)
  - `limit` (optional, default: 20)
  - `search` (optional, searches name, roll_no, student_id)
  - `class_name` (optional)
  - `course_name` (optional)
  - `section` (optional)
  - `status` (optional: `active`, `inactive`, `graduated`, `suspended`)
- **Response** (`200 OK`):
```json
{
  "status": "success",
  "data": [
    {
      "id": "e929b936-...",
      "student_id": "STU-2026-001",
      "student_name": "Aarav Sharma",
      "class_name": "10th",
      "course_name": "Senior Secondary",
      "section": "A",
      "roll_no": "1001",
      "gender": "Male",
      "dob": "2010-05-15",
      "phone": "+919876543210",
      "email": "aarav.sharma@example.com",
      "parent_name": "Rajesh Sharma",
      "parent_phone": "+919876500000",
      "address": "Civil Lines, Prayagraj",
      "status": "active",
      "created_at": "2026-09-06T00:00:00Z"
    }
  ],
  "total": 450,
  "page": 1,
  "limit": 20
}
```

---

#### 2. Get Single Student
- **Method**: `GET`
- **Route**: `/api/students/:id`
- **Auth**: Required
- **Path Parameter**: `:id` (UUID)
- **Response** (`200 OK`): Single student JSON object.

---

#### 3. Create Student Admission
- **Method**: `POST`
- **Route**: `/api/students`
- **Auth**: Required (`admin`, `staff`)
- **Request Body**:
```json
{
  "student_id": "STU-2026-002",
  "student_name": "Diya Patel",
  "class_name": "11th",
  "course_name": "Senior Secondary Science",
  "section": "Science-B",
  "roll_no": "1102",
  "gender": "Female",
  "dob": "2009-08-20",
  "phone": "+919876543211",
  "email": "diya.patel@example.com",
  "parent_name": "Sunil Patel",
  "parent_phone": "+919876500001",
  "address": "Sector 4, Lucknow",
  "status": "active"
}
```
- **Response** (`201 Created`): Created student object.

---

#### 4. Update Student Profile
- **Method**: `PUT`
- **Route**: `/api/students/:id`
- **Auth**: Required
- **Path Parameter**: `:id` (UUID)
- **Request Body**: Partial or full student fields (`class_name`, `course_name`, `section`, etc.).
- **Response** (`200 OK`): Updated student object.

---

#### 5. Delete / Archive Student
- **Method**: `DELETE`
- **Route**: `/api/students/:id`
- **Auth**: Admin only
- **Path Parameter**: `:id` (UUID)
- **Response** (`200 OK`):
```json
{
  "status": "success",
  "message": "Student deleted successfully"
}
```

---

#### 6. Bulk Import Students
- **Method**: `POST`
- **Route**: `/api/students/import`
- **Auth**: Required (`admin`, `staff`)
- **Request Body**: Array of student objects:
```json
[
  {
    "student_id": "STU-2026-003",
    "student_name": "Rohan Verma",
    "class_name": "12th",
    "course_name": "Commerce & Accountancy",
    "section": "Commerce",
    "roll_no": "1205",
    "phone": "+919876543299",
    "parent_name": "Amit Verma",
    "parent_phone": "+919876500099"
  }
]
```
- **Response** (`200 OK`):
```json
{
  "status": "success",
  "imported_count": 1,
  "failed_count": 0
}
```
