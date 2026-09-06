# Admin & System Management APIs

**Base Prefix**: `/api/admin`

Admin endpoints manage user lifecycle operations (activation, deactivation, deletion) and comprehensive system audit trail logs.

---

### Endpoints

#### 1. Toggle User Active Status
- **Method**: `PUT`
- **Route**: `/api/admin/users/:id/toggle`
- **Auth**: Admin only (`Bearer <accessToken>`)
- **Path Parameter**: `:id` (UUID of user)
- **Response** (`200 OK`):
```json
{
  "status": "success",
  "message": "User status toggled successfully",
  "data": {
    "id": "c1f72a44-...",
    "is_active": false
  }
}
```

---

#### 2. Delete User
- **Method**: `DELETE`
- **Route**: `/api/admin/users/:id`
- **Auth**: Admin only (`Bearer <accessToken>`)
- **Path Parameter**: `:id` (UUID of user)
- **Response** (`200 OK`):
```json
{
  "status": "success",
  "message": "User deleted successfully"
}
```

---

#### 3. Query Audit Logs
- **Method**: `GET`
- **Route**: `/api/admin/audit-logs`
- **Auth**: Admin only (`Bearer <accessToken>`)
- **Query Parameters**:
  - `page` (optional, default: 1)
  - `limit` (optional, default: 50)
  - `action` (optional, e.g. `CREATE`, `UPDATE`, `DELETE`)
  - `module` (optional, e.g. `inventory`, `finance`, `students`)
- **Response** (`200 OK`):
```json
{
  "status": "success",
  "data": [
    {
      "id": "f5e43a99-...",
      "user_id": "a0eebc99-...",
      "user_name": "Super Admin",
      "action": "CREATE_STUDENT",
      "module": "students",
      "details": "Created student profile for STU-2026-001",
      "ip_address": "127.0.0.1",
      "created_at": "2026-09-06T00:15:00Z"
    }
  ]
}
```

---

#### 4. Record Audit Log Entry
- **Method**: `POST`
- **Route**: `/api/admin/audit-logs`
- **Auth**: Required (`Bearer <accessToken>`)
- **Request Body**:
```json
{
  "action": "EXPORT_REPORT",
  "module": "finance",
  "details": "Exported Q3 fee collection ledger as CSV"
}
```
- **Response** (`201 Created`):
```json
{
  "status": "success",
  "message": "Audit log recorded"
}
```
