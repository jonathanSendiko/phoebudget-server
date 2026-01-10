# Arbor Premium Tier Specification

> **Document Purpose**: This specification defines the premium feature gating requirements for both the Flutter client and Rust backend API. Use this as the source of truth for implementing subscription logic.

---

## 1. Overview

Arbor will implement a **freemium model** with tiered feature access. The backend API must enforce these limits and expose subscription status to the client.

### Target Markets
- **Indonesia (IDR)**
- **Singapore (SGD)**  
- **United States (USD)**

---

## 2. Feature Access Matrix

| Feature | Free Tier | Premium Tier | Enforcement Layer |
|---------|-----------|--------------|-------------------|
| Transactions (base currency) | ✅ Unlimited | ✅ Unlimited | - |
| Multi-currency transactions | ❌ No | ✅ Yes | API (reject foreign currency) |
| Investments tracked | 3 max | Unlimited | API (count check) |
| Pockets | 2 max | Unlimited | API (count check) |
| Pocket transfers | ❌ No | ✅ Yes | API (reject transfer endpoint) |
| Spending analysis (charts) | ❌ No (summary only) | ✅ Full | API (return limited data) |
| Custom date range analysis | ❌ No | ✅ Yes | API (ignore date params) |
| Transaction history | 90 days | Unlimited | API (filter by date) |
| Data export (CSV/PDF) | ❌ No | ✅ Yes | API (new endpoint, premium only) |

---

## 3. API Requirements

### 3.1 User Subscription Schema

Add to `users` table or create `subscriptions` table:

```sql
CREATE TABLE subscriptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    plan VARCHAR(20) NOT NULL DEFAULT 'free', -- 'free', 'premium', 'lifetime'
    status VARCHAR(20) NOT NULL DEFAULT 'active', -- 'active', 'cancelled', 'expired'
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ, -- NULL for lifetime
    payment_provider VARCHAR(50), -- 'stripe', 'google_play', 'app_store', 'manual'
    external_subscription_id VARCHAR(255), -- Provider's subscription ID
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id)
);
```

### 3.2 Subscription Status Endpoint

**`GET /auth/subscription`**

Response:
```json
{
  "data": {
    "plan": "premium",
    "status": "active",
    "expires_at": "2027-01-09T00:00:00Z",
    "limits": {
      "max_investments": null,
      "max_pockets": null,
      "history_days": null,
      "multi_currency": true,
      "pocket_transfers": true,
      "advanced_analytics": true,
      "data_export": true
    }
  }
}
```

For free users:
```json
{
  "data": {
    "plan": "free",
    "status": "active",
    "expires_at": null,
    "limits": {
      "max_investments": 3,
      "max_pockets": 2,
      "history_days": 90,
      "multi_currency": false,
      "pocket_transfers": false,
      "advanced_analytics": false,
      "data_export": false
    }
  }
}
```

### 3.3 Feature Gating Logic

#### Investments (`POST /portfolio`)

```rust
// Before creating investment
let current_count = investment_repo.count_by_user(user_id).await?;
let limits = subscription_service.get_limits(user_id).await?;

if let Some(max) = limits.max_investments {
    if current_count >= max {
        return Err(ApiError::SubscriptionLimit {
            feature: "investments",
            limit: max,
            current: current_count,
        });
    }
}
```

Error Response (HTTP 403):
```json
{
  "error": {
    "code": "SUBSCRIPTION_LIMIT",
    "message": "You have reached the maximum of 3 investments on the free plan",
    "feature": "investments",
    "limit": 3,
    "upgrade_url": "arbor://upgrade"
  }
}
```

#### Pockets (`POST /pockets`)

Same pattern as investments, limit = 2 for free users.

#### Pocket Transfers (`POST /pockets/transfer`)

```rust
if !limits.pocket_transfers {
    return Err(ApiError::PremiumRequired {
        feature: "pocket_transfers",
        message: "Pocket transfers require a premium subscription",
    });
}
```

#### Multi-Currency Transactions (`POST /transactions`)

```rust
if payload.currency_code.is_some() && payload.currency_code != user.base_currency {
    if !limits.multi_currency {
        return Err(ApiError::PremiumRequired {
            feature: "multi_currency",
            message: "Multi-currency transactions require a premium subscription",
        });
    }
}
```

#### Transaction History (`GET /transactions`)

```rust
if let Some(history_days) = limits.history_days {
    let cutoff = Utc::now() - Duration::days(history_days as i64);
    // Modify query to filter: WHERE occurred_at >= cutoff
}
```

#### Spending Analysis (`GET /analysis/category`)

For free users, return limited response:
```json
{
  "data": {
    "total_income": 5000.00,
    "total_spent": 3500.00,
    "net_income": 1500.00,
    "categories": null,
    "premium_required": true
  }
}
```

Ignore `start_date` and `end_date` params for free users (always return current month).

#### Data Export (`GET /export/transactions`, `GET /export/portfolio`)

New endpoints, return 403 for free users.

---

## 4. Pricing Configuration

Store in config or database for easy updates:

```rust
pub struct PricingConfig {
    pub monthly: RegionalPricing,
    pub annual: RegionalPricing,
    pub lifetime: RegionalPricing,
}

pub struct RegionalPricing {
    pub idr: i64,  // Indonesian Rupiah (no decimals)
    pub sgd: f64,  // Singapore Dollars
    pub usd: f64,  // US Dollars
}
```

### Recommended Prices

| Plan | Indonesia (IDR) | Singapore (SGD) | United States (USD) |
|------|-----------------|-----------------|---------------------|
| **Monthly** | Rp 29,000 | SGD 4.99 | $4.99 |
| **Annual** | Rp 249,000 | SGD 49.99 | $49.99 |
| **Lifetime** | Rp 599,000 | SGD 119.00 | $99.00 |

### Pricing Endpoint (Optional)

**`GET /settings/pricing`**

```json
{
  "data": {
    "currency": "IDR",
    "plans": [
      {
        "id": "monthly",
        "name": "Premium Monthly",
        "price": 29000,
        "period": "month",
        "trial_days": 7
      },
      {
        "id": "annual",
        "name": "Premium Annual",
        "price": 249000,
        "period": "year",
        "savings_percent": 28,
        "trial_days": 7
      },
      {
        "id": "lifetime",
        "name": "Premium Lifetime",
        "price": 599000,
        "period": "lifetime",
        "trial_days": 0
      }
    ]
  }
}
```

---

## 5. Webhook Handlers (Payment Integration)

### Required Endpoints

| Endpoint | Purpose |
|----------|---------|
| `POST /webhooks/stripe` | Handle Stripe subscription events |
| `POST /webhooks/google-play` | Handle Google Play Billing events |
| `POST /webhooks/app-store` | Handle App Store subscription events |

### Key Events to Handle

- `subscription.created` → Set plan to premium, set expires_at
- `subscription.renewed` → Update expires_at
- `subscription.cancelled` → Set status to cancelled, keep premium until expires_at
- `subscription.expired` → Set plan to free, status to expired
- `payment.failed` → Optionally set status to past_due

---

## 6. Grace Period & Upgrade UX

### Grace Period
- When subscription expires, allow **3 days grace period** before downgrading.
- During grace period, show banner but maintain access.

### Downgrade Behavior
When user downgrades to free:
1. **Do NOT delete excess data** (investments, pockets beyond limit)
2. Make excess items **read-only** (can view/delete, cannot edit/add new)
3. Show clear messaging about which items are locked

---

## 7. Client-Side Caching

The Flutter client should:
1. Cache subscription status locally (with TTL of 1 hour)
2. Refresh on app foreground
3. Optimistically show premium UI during trial
4. Listen to deep links for upgrade completion (`arbor://upgrade-complete`)

---

## 8. Error Codes Reference

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `SUBSCRIPTION_LIMIT` | 403 | User exceeded free tier limit for countable feature |
| `PREMIUM_REQUIRED` | 403 | Feature requires premium, user is on free |
| `SUBSCRIPTION_EXPIRED` | 403 | User's subscription has expired |
| `TRIAL_EXPIRED` | 403 | User's trial has ended, needs to subscribe |

---

## 9. Testing Considerations

### Test Users
Create test users with different subscription states:
- `test+free@arbor.app` → Free plan
- `test+premium@arbor.app` → Premium plan
- `test+expired@arbor.app` → Expired premium
- `test+trial@arbor.app` → Active trial

### Bypass Flag (Dev Only)
Environment variable `BYPASS_SUBSCRIPTION_LIMITS=true` to disable all checks in development.

---

## Changelog

| Date | Author | Changes |
|------|--------|---------|
| 2026-01-09 | Initial | Created specification document |

