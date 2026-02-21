# Net Worth Growth (Monthly) API

## Purpose
Provide a monthly net worth history series (cash-only) for graphing growth over time.

## Endpoint
`GET /analysis/net-worth/history`

### Query Parameters
- `start_date` (required): `DateTime<Utc>`
- `end_date` (required): `DateTime<Utc>`

### Notes
- Uses UTC month boundaries.
- Categories with `exclude_from_analysis = true` are excluded.
- Months with no activity are included with zero totals.

## Response
```json
{
  "success": true,
  "message": null,
  "data": {
    "start_date": "2025-01-01T00:00:00Z",
    "end_date": "2025-03-31T23:59:59Z",
    "opening_balance": "100.00",
    "points": [
      {
        "month": "2025-01",
        "total_income": "50.00",
        "total_spent": "20.00",
        "net_change": "30.00",
        "net_worth_end": "130.00"
      },
      {
        "month": "2025-02",
        "total_income": "10.00",
        "total_spent": "40.00",
        "net_change": "-30.00",
        "net_worth_end": "100.00"
      },
      {
        "month": "2025-03",
        "total_income": "0.00",
        "total_spent": "0.00",
        "net_change": "0.00",
        "net_worth_end": "100.00"
      }
    ]
  }
}
```

## Response Schema
- `start_date`: `DateTime<Utc>`
- `end_date`: `DateTime<Utc>`
- `opening_balance`: `Decimal`
- `points[]`:
  - `month`: `YYYY-MM`
  - `total_income`: `Decimal`
  - `total_spent`: `Decimal`
  - `net_change`: `Decimal` (`total_income - total_spent`)
  - `net_worth_end`: `Decimal` (opening balance + cumulative net change)
