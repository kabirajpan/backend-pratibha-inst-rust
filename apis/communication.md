# Communication & Notifications APIs

**Base Prefixes**:
- WhatsApp Gateway: `/api/whatsapp`
- In-App User Notifications: `/api/notifications`

Covers external WhatsApp parent/student message dispatching and in-app system notifications (bell icon alerts and inbox).

---

### 1. WhatsApp Gateway

#### Check Gateway Connectivity Status
- **Method**: `GET`
- **Route**: `/api/whatsapp/status`
- **Auth**: Required (`admin`)
- **Response** (`200 OK`):
```json
{
  "status": "success",
  "data": {
    "connected": true,
    "provider": "whatsapp-cloud-api",
    "phone_number": "+91XXXXXXXXXX"
  }
}
```

#### Dispatch Test / Alert Message
- **Method**: `POST`
- **Route**: `/api/whatsapp/send`
- **Auth**: Required (`admin`)
- **Request Body**:
```json
{
  "phone": "+919876543210",
  "message": "Dear Parent, this is a test notification from Pratibha Institute portal."
}
```
- **Response** (`200 OK`):
```json
{
  "status": "success",
  "message": "WhatsApp message queued for delivery",
  "message_id": "wamid.HBgLM..."
}
```

---

### 2. In-App User Notifications & Inbox

#### List Notifications
- **Method**: `GET`
- **Route**: `/api/notifications`
- **Auth**: Required (`Bearer <accessToken>`)
- **Query Parameters**: `is_read` (optional: `true` / `false`), `limit`
- **Response** (`200 OK`):
```json
{
  "status": "success",
  "data": [
    {
      "id": "notif-01-...",
      "title": "Low Stock Alert",
      "message": "Item 'A4 Printing Paper' has reached minimum threshold (10 reams left)",
      "type": "warning",
      "is_read": false,
      "link": "/dashboard/inventory",
      "created_at": "2026-09-06T00:30:00Z"
    }
  ]
}
```

#### Mark Notification as Read
- **Method**: `PUT`
- **Route**: `/api/notifications/:id/read`
- **Auth**: Required

#### Mark All Notifications as Read
- **Method**: `PUT`
- **Route**: `/api/notifications/read-all`
- **Auth**: Required
