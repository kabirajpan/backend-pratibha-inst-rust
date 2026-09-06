# Inventory & Asset Management APIs

**Base Prefix**: `/api/inventory`

Endpoints for managing campus inventory assets, consumable stocks, supplier categories, low-stock threshold monitoring, and staff/department checkout transactions.

---

### 1. Stock Statistics & Alerts

#### Inventory Dashboard Stats
- **Method**: `GET`
- **Route**: `/api/inventory/stats`
- **Auth**: Required
- **Response** (`200 OK`):
```json
{
  "status": "success",
  "data": {
    "total_items": 340,
    "low_stock_items": 12,
    "categories_count": 8,
    "total_issued_items": 54
  }
}
```

#### Low Stock Warning List
- **Method**: `GET`
- **Route**: `/api/inventory/low-stock`
- **Auth**: Required
- **Response** (`200 OK`): List of items where `quantity <= min_stock_threshold`.

---

### 2. Item Categories

#### List Categories
- **Method**: `GET`
- **Route**: `/api/inventory/categories`

#### Create Category
- **Method**: `POST`
- **Route**: `/api/inventory/categories`
- **Request Body**:
```json
{
  "name": "Laboratory Equipment",
  "description": "Physics & Chemistry glassware and measuring instruments"
}
```

#### Delete Category
- **Method**: `DELETE`
- **Route**: `/api/inventory/categories/:id`

---

### 3. Inventory Items & Consumables

#### List Items
- **Method**: `GET`
- **Route**: `/api/inventory/items`
- **Query Parameters**: `category_id`, `search`, `page`, `limit`

#### Create Item
- **Method**: `POST`
- **Route**: `/api/inventory/items`
- **Request Body**:
```json
{
  "name": "A4 Printing Paper (Ream)",
  "category_id": "cat-01-...",
  "quantity": 100,
  "unit": "reams",
  "min_stock_threshold": 15,
  "unit_price": 280.00,
  "location": "Admin Store Room - Rack 3"
}
```

#### Update Item
- **Method**: `PUT`
- **Route**: `/api/inventory/items/:id`

#### Delete Item
- **Method**: `DELETE`
- **Route**: `/api/inventory/items/:id`

#### Bulk Import Items
- **Method**: `POST`
- **Route**: `/api/inventory/items/import`
- **Request Body**: Array of inventory items.

---

### 4. Item Issue & Returns

#### List Issued Transactions
- **Method**: `GET`
- **Route**: `/api/inventory/issues`
- **Query**: `status` (`issued`, `returned`), `issued_to`

#### Issue Item to Staff / Department
- **Method**: `POST`
- **Route**: `/api/inventory/issues`
- **Request Body**:
```json
{
  "item_id": "item-99-...",
  "issued_to": "Dr. R. K. Singh",
  "department": "Physics Dept",
  "quantity": 2,
  "purpose": "Optics Lab Experiment Session"
}
```

#### Return Issued Item
- **Method**: `POST`
- **Route**: `/api/inventory/issues/:id/return`
- **Request Body**:
```json
{
  "return_quantity": 2,
  "condition": "good",
  "notes": "Returned intact after lab session"
}
```
