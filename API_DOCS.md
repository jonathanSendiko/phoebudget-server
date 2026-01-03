# Phoebudget Server API Documentation

Base URL: `/api/v1`

## General Information

All successful API responses are wrapped in a standard JSON envelope:

```json
{
  "success": true,
  "data": { ... },
  "message": "Optional success message" // or null
}
```

### Common Error Response

If a request fails, the API returns a standard error envelope:

```json
{
  "success": false,
  "errors": [
    {
      "code": "VAL-400", // e.g., AUTH-401, NOT-404, DB-500
      "message": "Detailed error description"
    }
  ]
}
```

Dates are formatted as ISO 8601 strings (e.g., `2023-10-27T10:00:00Z`).
Currency values are returned as strings to preserve precision (e.g., `"10.50"`).

---

## 1. Authentication

### Register
Create a new user account.

- **URL:** `/auth/register`
- **Method:** `POST`
- **Body:**
    ```json
    {
      "username": "johndoe",
      "email": "john@example.com",
      "password": "secretpassword",
      "base_currency": "USD"
    }
    ```
- **Response Data:**
    ```json
    {
      "token": "jwt_token_string",
      "refresh_token": "refresh_token_hex_string",
      "message": "User registered successfully"
    }
    ```

### Login
Authenticate a user and retrieve a JWT token.

- **URL:** `/auth/login`
- **Method:** `POST`
- **Body:**
    ```json
    {
      "email": "john@example.com",
      "password": "secretpassword"
    }
    ```
- **Response Data:**
    ```json
    {
      "token": "jwt_token_string",
      "refresh_token": "refresh_token_hex_string",
      "message": "Login successful"
    }
    ```

### Refresh Token
Obtain a new access token using a valid refresh token.

- **URL:** `/auth/refresh`
- **Method:** `POST`
- **Body:**
    ```json
    {
      "refresh_token": "valid_refresh_token_hex"
    }
    ```
- **Response Data:**
    ```json
    {
      "token": "new_jwt_token_string",
      "refresh_token": "new_refresh_token_hex_string",
      "message": "Token refreshed"
    }
    ```

### Get Profile
Retrieve the authenticated user's profile.

- **URL:** `/auth/profile`
- **Method:** `GET`
- **Headers:** `Authorization: Bearer <token>`
- **Response Data:**
    ```json
    {
      "id": "uuid-string",
      "username": "johndoe",
      "email": "john@example.com",
      "base_currency": "USD",
      "joined_at": "2023-01-01T12:00:00Z"
    }
    ```

---

## 2. Transactions

### Create Transaction
Record a new transaction.

- **URL:** `/transactions`
- **Method:** `POST`
- **Body:**
    ```json
    {
      "amount": "50.00",
      "description": "Grocery shopping", // Optional
      "category_id": 1,
      "occurred_at": "2023-10-27T14:30:00Z",
      "currency_code": "USD", // Optional, defaults to user base currency
      "pocket_id": "uuid-string" // Optional, defaults to Main pocket
    }
    ```
- **Response Data:**
    ```json
    {
      "id": "new-transaction-uuid"
    }
    ```

### Get Transactions
Retrieve a paginated list of transactions within a date range.

- **URL:** `/transactions`
- **Method:** `GET`
- **Query Parameters:**
    - `start_date`: ISO 8601 date string (optional)
    - `end_date`: ISO 8601 date string (optional)
    - `pocket_id`: UUID (optional) - Filter by specific pocket
    - `page`: Integer (optional, default: 1)
    - `limit`: Integer (optional, default: 10, max: 100)
    
    *Note: If no dates are provided, returns the most recent transactions.*
- **Response Data:**
    ```json
    {
      "transactions": [
        {
          "id": "uuid-string",
          "amount": "50.00",
          "description": "Grocery shopping",
          "category": {
            "id": 1,
            "name": "Food",
            "is_income": false,
            "icon": "restaurant"
          },
          "pocket": {
            "id": "uuid-string",
            "name": "Main",
            "icon": "account_balance_wallet"
          },
          "occurred_at": "2023-10-27T14:30:00Z",
          "created_at": "2023-10-27T14:30:05Z"
        }
      ],
      "total": 150,
      "page": 1,
      "limit": 50,
      "total_pages": 3
    }
    ```

### Get Transaction Details
Retrieve detailed information for a specific transaction.

- **URL:** `/transactions/{id}`
- **Method:** `GET`
- **Response Data:**
    ```json
    {
      "id": "uuid-string",
      "amount": "50.00",
      "description": "Grocery shopping",
      "category_id": 1,
      "occurred_at": "2023-10-27T14:30:00Z",
      "created_at": "2023-10-27T14:30:05Z",
      "original_currency": "EUR",
      "original_amount": "45.00",
      "exchange_rate": "1.11"
    }
    ```

### Update Transaction
Update an existing transaction.

- **URL:** `/transactions/{id}`
- **Method:** `PUT`
- **Body:** (All fields optional)
    ```json
    {
      "amount": "60.00",
      "description": "Updated description",
      "category_id": 2,
      "occurred_at": "2023-10-28T10:00:00Z"
    }
    ```
- **Response Data:** `String` ("Transaction updated")

### Delete Transaction
Remove a transaction.
**Note:** This endpoint performs a "soft delete". The transaction is marked as deleted but is not removed from the database immediately. It can be restored using the `/restore` endpoint.

- **URL:** `/transactions/{id}`
- **Method:** `DELETE`
- **Response Data:** `String` ("Transaction deleted")

### Restore Transaction
Restore a previously deleted transaction.

- **URL:** `/transactions/{id}/restore`
- **Method:** `POST`
- **Response Data:** `String` ("Transaction restored")

---

## 3. Portfolio

### Get Portfolio
Retrieve a summary of all investments.

- **URL:** `/portfolio`
- **Method:** `GET`
- **Response Data:**
    ```json
    {
      "investments": [
        {
          "ticker": "AAPL",
          "name": "Apple Inc.",
          "quantity": "10.00",
          "avg_buy_price": "150.00",
          "current_price": "175.00",
          "total_value": "1750.00",
          "change_pct": "16.67",
          "icon_url": "https://path/to/icon.png" // or null
        }
      ],
      "total_cost": "1500.00",
      "absolute_change": "250.00"
    }
    ```

### Add Investment
Add a new asset to the portfolio.

- **URL:** `/portfolio`
- **Method:** `POST`
- **Body:**
    ```json
    {
      "ticker": "AAPL",
      "quantity": "5",
      "avg_buy_price": "150.00"
    }
    ```
- **Response Data:** `String` ("Added AAPL to portfolio")

### Refresh Portfolio
Trigger a price update for all assets in the portfolio (fetches latest prices).

- **URL:** `/portfolio/refresh`
- **Method:** `POST`
- **Response Data:** `String` ("Updated X assets")

### Update Investment
Update details of a specific holding.

- **URL:** `/portfolio/{ticker}`
- **Method:** `PUT`
- **Body:** (All fields optional)
    ```json
    {
      "quantity": "15",
      "avg_buy_price": "155.00"
    }
    ```
- **Response Data:** `String` ("Investment updated")

### Remove Investment
Remove an asset from the portfolio.

- **URL:** `/portfolio/{ticker}`
- **Method:** `DELETE`
- **Response Data:** `String` ("Investment removed")

### Get All Assets
Retrieve a list of all defined assets (e.g. stocks, indices, crypto) available in the system.

- **URL:** `/assets`
- **Method:** `GET`
- **Response Data:**
    ```json
    [
      {
        "ticker": "BTC",
        "name": "Bitcoin",
        "asset_type": "Crypto",
        "current_price": "45000.00",
        "icon_url": "https://assets.coingecko.com/.../bitcoin.png" // or null
      }
    ]
    ```

---

## 4. Analysis

### Category Analysis
Get spending and income breakdown by category.

- **URL:** `/analysis/category`
- **Method:** `GET`
- **Query Parameters:**
    - `start_date`
    - `end_date`
- **Response Data:**
    ```json
    {
      "total_income": "5000.00",
      "total_spent": "200.00",
      "net_income": "4800.00",
      "categories": [
        {
          "category": "Salary",
          "total": "5000.00",
          "is_income": true,
          "icon": "attach_money"
        },
        {
          "category": "Food",
          "total": "150.00",
          "is_income": false,
          "icon": "restaurant"
        }
      ]
    }
    ```

### Financial Health
Get overall financial status (Cash + Investments).

- **URL:** `/analysis/net-worth`
- **Method:** `GET`
- **Response Data:**
    ```json
    {
      "cash_balance": "5000.00",
      "investment_balance": "10000.00",
      "total_net_worth": "15000.00"
    }
    ```

---

## 5. Settings

### Update Base Currency
Change the user's preferred display currency.

- **URL:** `/settings/currency`
- **Method:** `PUT`
- **Body:**
    ```json
    {
      "base_currency": "EUR"
    }
    ```
- **Response Data:** `String` ("Base currency updated")

### Get Available Currencies
List all supported currencies.

- **URL:** `/settings/currencies`
- **Method:** `GET`
- **Response Data:** `["USD", "EUR", "GBP", "JPY", ...]`

### Get Categories
List all available transaction categories with their type and icon.

- **URL:** `/categories`
- **Method:** `GET`
- **Response Data:**
    ```json
    [
      {
        "id": 1,
        "name": "Food",
        "is_income": false,
        "icon": "restaurant"
      },
      {
        "id": 3,
        "name": "Salary",
        "is_income": true,
        "icon": "attach_money"
      }
    ]
    ```

---

## 6. Pockets

Pockets allow users to allocate money into different categories (e.g., Savings, Emergency Fund).
Every user has a default "Main" pocket that cannot be deleted.

### Create Pocket
Create a new pocket.

- **URL:** `/pockets`
- **Method:** `POST`
- **Body:**
    ```json
    {
      "name": "Savings",
      "description": "Emergency fund", // Optional
      "icon": "savings" // Optional, defaults to "account_balance_wallet"
    }
    ```
- **Response Data:**
    ```json
    {
      "id": "new-pocket-uuid"
    }
    ```

### Get Pockets
Retrieve all pockets for the authenticated user.

- **URL:** `/pockets`
- **Method:** `GET`
- **Response Data:**
    ```json
    [
      {
        "id": "uuid-string",
        "name": "Main",
        "description": null,
        "icon": "account_balance_wallet",
        "is_default": true,
        "created_at": "2023-01-01T12:00:00Z"
      },
      {
        "id": "uuid-string",
        "name": "Savings",
        "description": "Emergency fund",
        "icon": "savings",
        "is_default": false,
        "created_at": "2023-10-27T14:30:00Z"
      }
    ]
    ```

### Get Pocket
Retrieve details of a specific pocket.

- **URL:** `/pockets/{id}`
- **Method:** `GET`
- **Response Data:** Same as single pocket object above.

### Update Pocket
Update an existing pocket.

- **URL:** `/pockets/{id}`
- **Method:** `PUT`
- **Body:** (All fields optional)
    ```json
    {
      "name": "New Name",
      "description": "Updated description",
      "icon": "new_icon"
    }
    ```
- **Response Data:** `String` ("Pocket updated")

### Delete Pocket
Remove a pocket. Note: The default "Main" pocket cannot be deleted.

- **URL:** `/pockets/{id}`
- **Method:** `DELETE`
- **Response Data:** `String` ("Pocket deleted")

### Transfer Funds
Transfer money from one pocket to another.

- **URL:** `/pockets/transfer`
- **Method:** `POST`
- **Body:**
    ```json
    {
      "source_pocket_id": "uuid-string",
      "destination_pocket_id": "uuid-string",
      "amount": "50.00",
      "description": "Savings allocation" // Optional
    }
    ```
- **Response Data:** `String` ("Transfer successful")

