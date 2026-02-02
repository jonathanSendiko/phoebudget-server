use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

/// Subscription limits based on plan tier
#[derive(Serialize, Debug, Clone)]
pub struct SubscriptionLimits {
    pub max_investments: Option<i32>,
    pub max_pockets: Option<i32>,
    pub history_days: Option<i32>,
    pub multi_currency: bool,
    pub pocket_transfers: bool,
    pub advanced_analytics: bool,
    pub data_export: bool,
}

/// Response for GET /auth/subscription
#[derive(Serialize, Debug)]
pub struct SubscriptionResponse {
    pub plan: String,
    pub status: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub limits: SubscriptionLimits,
}

/// Internal row type for subscription repository
#[derive(Debug, sqlx::FromRow)]
pub struct SubscriptionRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub plan: String,
    pub status: String,
    pub started_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub payment_provider: Option<String>,
    pub external_subscription_id: Option<String>,
}
