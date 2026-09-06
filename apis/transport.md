# Transport Fleet & Logistics APIs

**Base Prefix**: `/api/transport`

Endpoints for managing institute buses/vans, fleet maintenance/fuel operational expenses, student commuter route assignments, and bulk Excel imports.

---

### 1. Vehicle Fleet Management

#### List Vehicles
- **Method**: `GET`
- **Route**: `/api/transport/vehicles`
- **Auth**: Required
- **Response** (`200 OK`):
```json
{
  "status": "success",
  "data": [
    {
      "id": "v101-...",
      "vehicle_number": "UP-70-AB-1234",
      "model": "Tata Starbus Ultra",
      "driver_name": "Ramesh Yadav",
      "driver_phone": "+919876540001",
      "capacity": 42,
      "route": "Civil Lines to Main Campus",
      "status": "active"
    }
  ]
}
```

#### Add Vehicle
- **Method**: `POST`
- **Route**: `/api/transport/vehicles`
- **Auth**: Required (`admin`, `transport_manager`)
- **Request Body**:
```json
{
  "vehicle_number": "UP-70-CD-5678",
  "model": "Force Traveller 26",
  "driver_name": "Suresh Kumar",
  "driver_phone": "+919876540002",
  "capacity": 26,
  "route": "Teliyarganj to Main Campus",
  "status": "active"
}
```
- **Response** (`201 Created`): Created vehicle record.

#### Update / Delete Vehicle
- **Method**: `PUT` / `DELETE`
- **Route**: `/api/transport/vehicles/:id`

---

### 2. Vehicle Operational Expenses

#### List Expenses
- **Method**: `GET`
- **Route**: `/api/transport/expenses`
- **Query**: `vehicle_id` (optional), `date_from`, `date_to`

#### Add Vehicle Expense
- **Method**: `POST`
- **Route**: `/api/transport/expenses`
- **Request Body**:
```json
{
  "vehicle_id": "v101-...",
  "expense_type": "Fuel",
  "amount": 4200.00,
  "description": "Diesel 45L refuel at Indian Oil bunk",
  "expense_date": "2026-09-05"
}
```

---

### 3. Student Commuter Allocations

#### List Commuter Students
- **Method**: `GET`
- **Route**: `/api/transport/students`

#### Assign Student to Route / Vehicle
- **Method**: `POST`
- **Route**: `/api/transport/students`
- **Request Body**:
```json
{
  "student_id": "STU-2026-001",
  "student_name": "Aarav Sharma",
  "vehicle_id": "v101-...",
  "pickup_point": "Civil Lines Crossing",
  "monthly_fee": 1200.00
}
```

#### Bulk Import Transport Students
- **Method**: `POST`
- **Route**: `/api/transport/students/import`
- **Request Body**: Array of student transport assignment objects.
