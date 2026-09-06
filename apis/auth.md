# Authentication & User APIs

**Base Prefix**: `/api/auth`

Authentication endpoints handle user onboarding, session generation with JWT tokens, profile retrieval, password management, and staff directory access.

---

### Endpoints

#### 1. Register New User / Staff
- **Method**: `POST`
- **Route**: `/api/auth/register`
- **Auth**: Public or Admin
- **Request Body**:
```json
{
  "name": "Jane Doe",
  "email": "jane.doe@pratibha.edu",
  "password": "SecurePassword123!",
  "role": "staff"
}
```
> Allowed roles: `admin`, `staff`, `accountant`, `librarian`, `warden`, `transport_manager`
- **Response** (`201 Created`):
```json
{
  "status": "success",
  "data": {
    "user": {
      "id": "c1f72a44-...",
      "name": "Jane Doe",
      "email": "jane.doe@pratibha.edu",
      "role": "staff",
      "created_at": "2026-09-06T00:00:00Z"
    },
    "accessToken": "eyJhbGciOi..."
  }
}
```

---

#### 2. User Login
- **Method**: `POST`
- **Route**: `/api/auth/login`
- **Auth**: Public
- **Request Body**:
```json
{
  "email": "admin@pratibha.edu",
  "password": "AdminPassword123!"
}
```
- **Response** (`200 OK`):
```json
{
  "status": "success",
  "data": {
    "user": {
      "id": "a0eebc99-...",
      "name": "Super Admin",
      "email": "admin@pratibha.edu",
      "role": "admin"
    },
    "accessToken": "eyJhbGciOi..."
  }
}
```

---

#### 3. Refresh Access Token
- **Method**: `POST`
- **Route**: `/api/auth/refresh`
- **Auth**: Bearer Refresh Token or Cookie
- **Response** (`200 OK`):
```json
{
  "status": "success",
  "data": {
    "accessToken": "eyJhbGciOi..."
  }
}
```

---

#### 4. Get Current User Profile
- **Method**: `GET`
- **Route**: `/api/auth/me`
- **Auth**: Required (`Bearer <accessToken>`)
- **Response** (`200 OK`):
```json
{
  "status": "success",
  "data": {
    "id": "a0eebc99-...",
    "name": "Super Admin",
    "email": "admin@pratibha.edu",
    "role": "admin",
    "is_active": true
  }
}
```

---

#### 5. Change Password
- **Method**: `POST`
- **Route**: `/api/auth/change-password`
- **Auth**: Required (`Bearer <accessToken>`)
- **Request Body**:
```json
{
  "old_password": "CurrentPassword123!",
  "new_password": "NewSecurePassword456!"
}
```
- **Response** (`200 OK`):
```json
{
  "status": "success",
  "message": "Password changed successfully"
}
```

---

#### 6. User Logout
- **Method**: `POST`
- **Route**: `/api/auth/logout`
- **Auth**: Required
- **Response** (`200 OK`):
```json
{
  "status": "success",
  "message": "Logged out successfully"
}
```

---

#### 7. List Staff Members
- **Method**: `GET`
- **Route**: `/api/auth/staff`
- **Auth**: Required (`admin` / `staff`)
- **Response** (`200 OK`):
```json
{
  "status": "success",
  "data": [
    {
      "id": "c1f72a44-...",
      "name": "Jane Doe",
      "email": "jane.doe@pratibha.edu",
      "role": "staff",
      "is_active": true,
      "created_at": "2026-09-06T00:00:00Z"
    }
  ]
}
```
