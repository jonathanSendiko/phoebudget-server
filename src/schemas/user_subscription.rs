use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::common::round_currency;
use super::pocket::PocketSummary;
use super::transaction::Category;

// --- User Subscriptions DTOs ---

#[derive(Deserialize, Debug)]
pub struct CreateUserSubscription {
    pub name: String,
    pub description: Option<String>,
    pub amount: Decimal,
    pub basis: String, // 'monthly' or 'annually'
    pub billing_day: i32,
    pub billing_month: Option<i32>,
    pub category_id: Option<i32>,
    pub pocket_id: Uuid,
}

#[derive(Deserialize, Debug)]
pub struct UpdateUserSubscription {
    pub name: Option<String>,
    pub description: Option<String>,
    pub amount: Option<Decimal>,
    pub basis: Option<String>,
    pub billing_day: Option<i32>,
    pub billing_month: Option<i32>,
    pub category_id: Option<i32>,
    pub is_active: Option<bool>,
    // pocket_id cannot be changed easily as it affects future transactions? allowing it.
    pub pocket_id: Option<Uuid>,
}

#[derive(Serialize, Debug)]
pub struct UserSubscriptionSummary {
    pub id: Uuid,
    pub name: String,
    #[serde(serialize_with = "round_currency")]
    pub amount: Decimal,
    pub basis: String,
    pub next_charge_date: NaiveDate,
    pub is_active: bool,
    pub icon: String, // From category
}

#[derive(Serialize, Debug)]
pub struct UserSubscriptionDetail {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    #[serde(serialize_with = "round_currency")]
    pub amount: Decimal,
    pub basis: String,
    pub billing_day: i32,
    pub billing_month: Option<i32>,
    pub next_charge_date: NaiveDate,
    pub is_active: bool,
    pub pocket: PocketSummary,
    pub category: Option<Category>,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Serialize)]
pub struct UserSubscriptionId {
    pub id: Uuid,
}

#[derive(Debug, sqlx::FromRow)]
pub struct UserSubscriptionRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub pocket_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub amount: Decimal,
    pub basis: String,
    pub billing_day: i32,
    pub billing_month: Option<i32>,
    pub category_id: Option<i32>,
    pub is_active: Option<bool>,
    pub next_charge_date: NaiveDate,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}
