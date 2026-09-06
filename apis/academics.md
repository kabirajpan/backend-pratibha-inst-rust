# Classes & Courses (Academics) APIs

**Base Prefixes**:
- `/api/classes` (and `/api/library/classes` compatibility alias)
- `/api/courses` (and `/api/library/courses` compatibility alias)

Provides academic reference entities used across admissions, fee assignments, and library card issuance.

---

### 1. Classes Endpoints

#### List All Classes
- **Method**: `GET`
- **Route**: `/api/classes` or `/api/library/classes`
- **Auth**: Required
- **Response** (`200 OK`):
```json
{
  "status": "success",
  "data": [
    {
      "id": "b3e34b12-...",
      "class_name": "10th",
      "section": "A",
      "capacity": 45,
      "created_at": "2026-09-06T00:00:00Z"
    }
  ]
}
```

#### Create Class
- **Method**: `POST`
- **Route**: `/api/classes`
- **Auth**: Required (`admin`, `staff`)
- **Request Body**:
```json
{
  "class_name": "11th",
  "section": "Science-A",
  "capacity": 50
}
```
- **Response** (`201 Created`): Created class object.

#### Delete Class
- **Method**: `DELETE`
- **Route**: `/api/classes/:id`
- **Auth**: Admin only
- **Response** (`200 OK`):
```json
{
  "status": "success",
  "message": "Class deleted successfully"
}
```

---

### 2. Courses & Programs Endpoints

#### List All Courses
- **Method**: `GET`
- **Route**: `/api/courses` or `/api/library/courses`
- **Auth**: Required
- **Response** (`200 OK`):
```json
{
  "status": "success",
  "data": [
    {
      "id": "c7a19d88-...",
      "course_name": "Senior Secondary Science",
      "course_code": "SCI-101",
      "description": "Physics, Chemistry, Math & Biology curriculum",
      "duration_months": 24,
      "created_at": "2026-09-06T00:00:00Z"
    }
  ]
}
```

#### Create Course
- **Method**: `POST`
- **Route**: `/api/courses`
- **Auth**: Required (`admin`)
- **Request Body**:
```json
{
  "course_name": "Commerce & Accountancy",
  "course_code": "COMM-201",
  "description": "General commerce, accounting, and economics",
  "duration_months": 24
}
```
- **Response** (`201 Created`): Created course object.

#### Delete Course
- **Method**: `DELETE`
- **Route**: `/api/courses/:id`
- **Auth**: Admin only
- **Response** (`200 OK`):
```json
{
  "status": "success",
  "message": "Course deleted successfully"
}
```
