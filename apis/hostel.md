# Hostel Management APIs

**Base Prefix**: `/api/hostel`

Endpoints for managing residential facility blocks, rooms, bed capacity, and student resident assignments.

---

### Endpoints

#### 1. List Hostel Rooms
- **Method**: `GET`
- **Route**: `/api/hostel/rooms`
- **Auth**: Required (`Bearer <accessToken>`)
- **Query Parameters**:
  - `block` (optional, e.g. `Block A`, `Block B`)
  - `room_type` (optional: `single`, `double`, `triple`, `dormitory`)
- **Response** (`200 OK`):
```json
{
  "status": "success",
  "data": [
    {
      "id": "h1a2b3c4-...",
      "room_number": "A-101",
      "block": "Block A",
      "floor": 1,
      "capacity": 2,
      "occupied": 1,
      "monthly_rent": 4500.00,
      "status": "available",
      "created_at": "2026-09-06T00:00:00Z"
    }
  ]
}
```

---

#### 2. Create Hostel Room
- **Method**: `POST`
- **Route**: `/api/hostel/rooms`
- **Auth**: Required (`admin`, `warden`)
- **Request Body**:
```json
{
  "room_number": "B-204",
  "block": "Block B",
  "floor": 2,
  "capacity": 3,
  "monthly_rent": 3800.00,
  "status": "available"
}
```
- **Response** (`201 Created`): Created room object.

---

#### 3. Update Hostel Room
- **Method**: `PUT`
- **Route**: `/api/hostel/rooms/:id`
- **Auth**: Required
- **Request Body**: Fields to update (`capacity`, `monthly_rent`, `status`).
- **Response** (`200 OK`): Updated room object.

---

#### 4. Delete Hostel Room
- **Method**: `DELETE`
- **Route**: `/api/hostel/rooms/:id`
- **Auth**: Admin only
- **Response** (`200 OK`):
```json
{
  "status": "success",
  "message": "Room deleted successfully"
}
```

---

#### 5. List Hostel Residents (Students)
- **Method**: `GET`
- **Route**: `/api/hostel/students`
- **Auth**: Required
- **Query Parameters**:
  - `room_id` (optional)
  - `status` (optional: `active`, `vacated`)
- **Response** (`200 OK`):
```json
{
  "status": "success",
  "data": [
    {
      "id": "s8c7d6e5-...",
      "student_id": "STU-2026-001",
      "student_name": "Aarav Sharma",
      "room_id": "h1a2b3c4-...",
      "room_number": "A-101",
      "bed_number": "Bed 1",
      "admission_date": "2026-08-01",
      "guardian_phone": "+919876500000",
      "status": "active"
    }
  ]
}
```

---

#### 6. Allocate Room to Student
- **Method**: `POST`
- **Route**: `/api/hostel/students`
- **Auth**: Required (`admin`, `warden`)
- **Request Body**:
```json
{
  "student_id": "STU-2026-002",
  "student_name": "Diya Patel",
  "room_id": "h1a2b3c4-...",
  "bed_number": "Bed 2",
  "admission_date": "2026-09-01",
  "guardian_phone": "+919876500001"
}
```
- **Response** (`201 Created`): Created resident object.

---

#### 7. Update Student Hostel Allocation
- **Method**: `PUT`
- **Route**: `/api/hostel/students/:id`
- **Auth**: Required
- **Request Body**: Fields to modify (e.g., room change or status to `vacated`).
- **Response** (`200 OK`): Updated resident object.

---

#### 8. Remove Student from Hostel
- **Method**: `DELETE`
- **Route**: `/api/hostel/students/:id`
- **Auth**: Required (`admin`, `warden`)
- **Response** (`200 OK`):
```json
{
  "status": "success",
  "message": "Student removed from hostel room"
}
```
