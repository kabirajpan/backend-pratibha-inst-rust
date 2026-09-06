# Announcements & Broadcast APIs

**Base Prefix**: `/api/announcements`

Handles circulars, department notices, emergency alerts, target audience filtering (students, faculty, parents, all), and broadcast permission governance.

---

### Endpoints

#### 1. List Announcements
- **Method**: `GET`
- **Route**: `/api/announcements`
- **Auth**: Required
- **Query Parameters**:
  - `target_role` (optional: `all`, `students`, `staff`, `wardens`)
  - `category` (optional: `academic`, `administrative`, `hostel`, `sports`)
  - `page`, `limit`
- **Response** (`200 OK`):
```json
{
  "status": "success",
  "data": [
    {
      "id": "anc-001-...",
      "title": "Annual Sports Meet 2026 Registration",
      "content": "Registrations are now open for track and field events at the sports office.",
      "category": "sports",
      "target_role": "students",
      "is_pinned": true,
      "author_name": "Sports Dept",
      "expires_at": "2026-09-20T00:00:00Z",
      "created_at": "2026-09-05T00:00:00Z"
    }
  ]
}
```

---

#### 2. Create Announcement
- **Method**: `POST`
- **Route**: `/api/announcements`
- **Auth**: Required (`admin`, `staff` with broadcast permission)
- **Request Body**:
```json
{
  "title": "Semester Examination Fee Submission Deadline",
  "content": "All students must clear pending dues before Sept 15 to collect admit cards.",
  "category": "academic",
  "target_role": "students",
  "is_pinned": false,
  "expires_at": "2026-09-15T23:59:59Z"
}
```
- **Response** (`201 Created`): Created announcement object.

---

#### 3. Update Announcement
- **Method**: `PUT`
- **Route**: `/api/announcements/:id`
- **Auth**: Required (`admin` or original author)

---

#### 4. Delete Announcement
- **Method**: `DELETE`
- **Route**: `/api/announcements/:id`
- **Auth**: Required (`admin`)

---

#### 5. Broadcast Permissions
- **Method**: `GET` / `PUT`
- **Route**: `/api/announcements/permissions`
- **Auth**: Admin only
- **Description**: Configure which staff roles or specific staff members have rights to broadcast notices across departments.
