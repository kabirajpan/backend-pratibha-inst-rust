# Unified Module Todos APIs

**Base Prefix**: `/api/todos`

Provides contextual task and action item management scoped per institute module (`library`, `hostel`, `transport`, `inventory`, `finance`, `general`).

---

### Endpoints

#### 1. List Module Todos
- **Method**: `GET`
- **Route**: `/api/todos/:module`
- **Auth**: Required (`Bearer <accessToken>`)
- **Path Parameter**: `:module` (e.g. `library`, `inventory`, `hostel`, `transport`, `finance`, `general`)
- **Response** (`200 OK`):
```json
{
  "status": "success",
  "data": [
    {
      "id": "todo-001-...",
      "module": "inventory",
      "title": "Restock A4 paper reams",
      "is_completed": false,
      "priority": "high",
      "due_date": "2026-09-10",
      "created_at": "2026-09-05T00:00:00Z"
    }
  ]
}
```

---

#### 2. Create Todo
- **Method**: `POST`
- **Route**: `/api/todos/:module`
- **Auth**: Required
- **Path Parameter**: `:module`
- **Request Body**:
```json
{
  "title": "Inspect Room B-204 water supply",
  "priority": "medium",
  "due_date": "2026-09-08"
}
```
- **Response** (`201 Created`): Created todo object.

---

#### 3. Update Todo Status / Content
- **Method**: `PUT`
- **Route**: `/api/todos/:id`
- **Auth**: Required
- **Path Parameter**: `:id` (UUID)
- **Request Body**:
```json
{
  "is_completed": true,
  "title": "Restock A4 paper reams (Delivered)"
}
```
- **Response** (`200 OK`): Updated todo object.

---

#### 4. Delete Todo
- **Method**: `DELETE`
- **Route**: `/api/todos/:id`
- **Auth**: Required
- **Response** (`200 OK`):
```json
{
  "status": "success",
  "message": "Todo deleted successfully"
}
```
