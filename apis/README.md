# Backend API Documentation Directory

This directory contains the complete reference documentation for all HTTP endpoints exposed by the Pratibha Institute backend service (`backend-rust`), running on port `5000` (default) under the `/api` prefix.

## Modular API Specifications

1. [01. Authentication & Users (`auth.md`)](./auth.md)
   - Staff/admin registration, login, token refresh, current user profile, password updates, staff directory.
2. [02. Admin & System Management (`admin.md`)](./admin.md)
   - User account status toggles, user deletion, system audit log query & logging.
3. [03. Students Directory (`students.md`)](./students.md)
   - Student profiles CRUD, class/section queries, bulk Excel/CSV import.
4. [04. Classes & Courses (`academics.md`)](./academics.md)
   - Academic master records: Class listings & Course/Subject catalog.
5. [05. Hostel Management (`hostel.md`)](./hostel.md)
   - Room allocations, occupancy tracking, hostel resident admissions & maintenance.
6. [06. Transport & Fleet Logistics (`transport.md`)](./transport.md)
   - Bus/van vehicle fleet records, vehicle operational expenses, bus-rider student rosters, bulk imports.
7. [07. Inventory & Assets (`inventory.md`)](./inventory.md)
   - Stock statistics, categories, asset items, low-stock warnings, item issuance and returns.
8. [08. Library Management (`library.md`)](./library.md)
   - Catalog stats, book records, library members, book issuing/returning, fine calculations, rules & settings.
9. [09. Finance & Fee Accounting (`finance.md`)](./finance.md)
   - Student fee payment tracking, collection summaries, institute expense ledger.
10. [10. Unified Module Todos (`todos.md`)](./todos.md)
    - Cross-module task tracking (library, hostel, transport, inventory, finance, general).
11. [11. Announcements & Broadcasts (`announcements.md`)](./announcements.md)
    - Departmental and institute-wide announcements, role-based targets, broadcast permissions.
12. [12. WhatsApp Gateway & Notifications (`communication.md`)](./communication.md)
    - Gateway status, test message dispatch, user notification bell & inbox endpoints.

---

## Global Standards & Headers

- **Base URL**: `http://localhost:5000/api`
- **Content-Type**: `application/json` (except multi-part file imports)
- **Authentication**: 
  - Standard JWT Bearer token passed in the `Authorization` header:
    ```http
    Authorization: Bearer <accessToken>
    ```
  - Also compatible with HTTP-only cookie extraction (`accessToken`).
