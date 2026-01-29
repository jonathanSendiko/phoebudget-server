# Financial Goals & Recurring Subscriptions API Documentation

This document outlines the API endpoints for the newly implemented Financial Goals and Recurring Subscriptions features.

## Base URL
`/api/v1`

---

## 1. Financial Goals
Manage savings goals linked to pockets.

### 1.1 Create Goal
Create a new financial goal.

**Endpoint:** `POST /goals`

**Request Body:**
| Field | Type | Mandatory | Description |
|---|---|---|---|
| `name` | String | Yes | Name of the goal (e.g., "New Car") |
| `target_amount` | Decimal | Yes | Target saving amount (e.g., 20000.00) |
| `pocket_id` | UUID | Yes | ID of the pocket where funds are stored |
| `description` | String | No | Optional description |
| `current_amount` | Decimal | No | Current saved amount (default: 0) |
| `icon` | String | No | Icon identifier (default: "savings") |

**Example Request:**
```json
{
  "name": "Japan Trip",
  "target_amount": 5000.00,
  "current_amount": 1000.00,
  "pocket_id": "123e4567-e89b-12d3-a456-426614174000",
  "icon": "flight"
}
```

**Response (200 OK):**
```json
{
  "status": "success",
  "data": {
    "id": "goal-uuid-here"
  }
}
```

### 1.2 Get Goals
List all financial goals for the authenticated user.

**Endpoint:** `GET /goals`

**Response (200 OK):**
Array of goal summaries.
```json
{
  "status": "success",
  "data": [
    {
      "id": "goal-uuid",
      "name": "Japan Trip",
      "icon": "flight",
      "target_amount": "5000.00",
      "current_amount": "1500.00",
      "percentage": 30.0
    }
  ]
}
```

### 1.3 Get Goal Details
Get detailed information about a specific goal.

**Endpoint:** `GET /goals/{id}`

**Response (200 OK):**
```json
{
  "status": "success",
  "data": {
    "id": "goal-uuid",
    "name": "Japan Trip",
    "description": "Saving for summer vacation",
    "icon": "flight",
    "target_amount": "5000.00",
    "current_amount": "1500.00",
    "percentage": 30.0,
    "pocket": {
      "id": "pocket-uuid",
      "name": "Travel Fund",
      "icon": "travel"
    },
    "created_at": "2026-01-29T10:00:00Z"
  }
}
```

### 1.4 Update Goal
Update an existing goal.

**Endpoint:** `PUT /goals/{id}`

**Request Body:**
All fields are optional. Only provided fields will be updated.

| Field | Type | Mandatory | Description |
|---|---|---|---|
| `name` | String | No | New name |
| `description` | String | No | New description |
| `target_amount` | Decimal | No | New target amount |
| `icon` | String | No | New icon |

**Example Request:**
```json
{
  "target_amount": 6000.00
}
```

### Create Goal Entry (Add/Remove Funds)
`POST /api/v1/goals/{id}/entries`

Adds or removes funds from a goal. The goal's `current_amount` is automatically updated.

**Request Body:**
| Field | Type | Required | Description |
|---|---|---|---|
| `amount` | Decimal | Yes | Amount to add (positive) or remove (negative) |
| `description` | String | No | Optional note |
| `date` | DateTime | No | Date of entry (default is now) |

**Example Request:**
```json
{
  "amount": 500.00,
  "description": "Monthly saving"
}
```

### Get Goal Entries
`GET /api/v1/goals/{id}/entries`

Returns a list of entries for a specific goal.

**Response (200 OK):**
```json
{
  "status": "success",
  "data": null
}
```

### 1.5 Delete Goal
Delete a financial goal.

**Endpoint:** `DELETE /goals/{id}`

**Response (200 OK):**
```json
{
  "status": "success",
  "data": null
}
```

---

## 2. Recurring Subscriptions
Manage monthly or annual subscriptions with automatic transaction creation.

### 2.1 Create Subscription
Create a new recurring subscription.

**Endpoint:** `POST /subscriptions`

**Request Body:**
| Field | Type | Mandatory | Description |
|---|---|---|---|
| `name` | String | Yes | Name (e.g., "Netflix") |
| `amount` | Decimal | Yes | Cost per cycle |
| `basis` | String | Yes | Frequency: "monthly" or "annually" |
| `billing_day` | Integer | Yes | Day of month to charge (1-31) |
| `pocket_id` | UUID | Yes | Source pocket for payment |
| `billing_month` | Integer | No | Required if basis is "annually" (1-12) |
| `category_id` | Integer | No | Category for generated transactions |
| `description` | String | No | Optional description |

**Example Request (Monthly):**
```json
{
  "name": "Netflix",
  "amount": 15.99,
  "basis": "monthly",
  "billing_day": 15,
  "pocket_id": "pocket-uuid"
}
```

**Example Request (Annual):**
```json
{
  "name": "Amazon Prime",
  "amount": 139.00,
  "basis": "annually",
  "billing_day": 1,
  "billing_month": 1,
  "pocket_id": "pocket-uuid"
}
```

**Response (200 OK):**
```json
{
  "status": "success",
  "data": {
    "id": "sub-uuid"
  }
}
```

### 2.2 Get Subscriptions
List all active and inactive subscriptions.

**Endpoint:** `GET /subscriptions`

**Response (200 OK):**
```json
{
  "status": "success",
  "data": [
    {
      "id": "sub-uuid",
      "name": "Netflix",
      "amount": "15.99",
      "basis": "monthly",
      "next_charge_date": "2026-02-15",
      "is_active": true,
      "icon": "movie"
    }
  ]
}
```

### 2.3 Get Subscription Details
Get details of a specific subscription.

**Endpoint:** `GET /subscriptions/{id}`

**Response (200 OK):**
```json
{
  "status": "success",
  "data": {
    "id": "sub-uuid",
    "name": "Netflix",
    "description": "Standard Plan",
    "amount": "15.99",
    "basis": "monthly",
    "billing_day": 15,
    "billing_month": null,
    "next_charge_date": "2026-02-15",
    "is_active": true,
    "pocket": {
      "id": "pocket-uuid",
      "name": "Entertainment",
      "icon": "movie"
    },
    "category": {
      "id": 5,
      "name": "Subscriptions",
      "icon": "subscriptions"
    },
    "created_at": "2026-01-29T10:00:00Z"
  }
}
```

### 2.4 Update Subscription
Update an existing subscription.

**Endpoint:** `PUT /subscriptions/{id}`

**Request Body:**
All fields are optional.

| Field | Type | Mandatory | Description |
|---|---|---|---|
| `name` | String | No | New name |
| `amount` | Decimal | No | New amount |
| `billing_day` | Integer | No | New billing day |
| `is_active` | Boolean | No | Pause/Resume subscription |
| `pocket_id` | UUID | No | Change source pocket |
| ... | ... | ... | Other fields similar to Create |

**Example Request:**
```json
{
  "amount": 17.99,
  "is_active": false
}
```

**Response (200 OK):**
```json
{
  "status": "success",
  "data": null
}
```

### 2.5 Delete Subscription
Delete a subscription.

**Endpoint:** `DELETE /subscriptions/{id}`

**Response (200 OK):**
```json
{
  "status": "success",
  "data": null
}
```
