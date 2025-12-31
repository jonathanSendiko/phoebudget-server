use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize, Serializer};
use uuid::Uuid;

fn round_currency<S>(x: &Decimal, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    // Round to 2 decimals, then convert to a string to preserve trailing zeros (e.g., "10.50")
    // Note: Sending as String is safest for frontend apps to avoid float precision issues.
    // If you prefer sending a Number, remove the .to_string() part.
    s.serialize_str(&x.round_dp(2).to_string())
}

fn round_currency_option<S>(x: &Option<Decimal>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match x {
        Some(d) => round_currency(d, s),
        None => s.serialize_none(),
    }
}

// --- Request DTOs ---

#[derive(Deserialize, Debug)]
pub struct CreateTransaction {
    pub amount: Decimal,
    #[serde(default)]
    pub description: Option<String>,
    pub category_id: i32,
    pub occurred_at: DateTime<Utc>,
    pub currency_code: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct CreatePortfolioItem {
    pub ticker: String,
    pub quantity: Decimal,
    pub avg_buy_price: Decimal,
}

#[derive(Deserialize)]
pub struct DateRangeParams {
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
}

// --- Response DTOs ---

#[derive(Serialize, Debug)]
pub struct Transaction {
    pub id: Uuid,
    #[serde(serialize_with = "round_currency")]
    pub amount: Decimal,
    pub description: Option<String>,
    pub category_id: Option<i32>,
    pub occurred_at: DateTime<Utc>,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, Debug)]
pub struct TransactionDetail {
    pub id: Uuid,
    #[serde(serialize_with = "round_currency")]
    pub amount: Decimal,
    pub description: Option<String>,
    pub category_id: Option<i32>,
    pub occurred_at: DateTime<Utc>,
    pub created_at: Option<DateTime<Utc>>,
    pub original_currency: Option<String>,
    #[serde(serialize_with = "round_currency_option", default)]
    pub original_amount: Option<Decimal>,
    pub exchange_rate: Option<Decimal>,
}

#[derive(Serialize)]
pub struct TransactionId {
    pub id: Uuid,
}

#[derive(Serialize, Debug)]
pub struct CategorySummary {
    pub category: String,
    #[serde(serialize_with = "round_currency")]
    pub total: Decimal,
}

#[derive(Serialize, Debug)]
pub struct FinancialHealth {
    #[serde(serialize_with = "round_currency")]
    pub cash_balance: Decimal,
    #[serde(serialize_with = "round_currency")]
    pub investment_balance: Decimal,
    #[serde(serialize_with = "round_currency")]
    pub total_net_worth: Decimal,
}

#[derive(Serialize, Debug)]
pub struct InvestmentSummary {
    pub ticker: String,
    pub name: String,
    #[serde(serialize_with = "round_currency")]
    pub quantity: Decimal,
    #[serde(serialize_with = "round_currency")]
    pub avg_buy_price: Decimal,
    #[serde(serialize_with = "round_currency")]
    pub current_price: Decimal,
    #[serde(serialize_with = "round_currency")]
    pub total_value: Decimal,
    #[serde(serialize_with = "round_currency")]
    pub change_pct: Decimal,
}

// --- Auth DTOs ---

#[derive(Deserialize, Debug)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub base_currency: String,
}

#[derive(Deserialize, Debug)]
pub struct UpdateTransaction {
    pub amount: Option<Decimal>,
    pub description: Option<String>,
    pub category_id: Option<i32>,
    pub occurred_at: Option<DateTime<Utc>>,
    pub original_currency: Option<String>,
    pub original_amount: Option<Decimal>,
    pub exchange_rate: Option<Decimal>,
}

#[derive(Deserialize, Debug)]
pub struct UpdateCurrency {
    pub base_currency: String,
}

#[derive(Deserialize, Debug)]
pub struct UpdateInvestment {
    pub quantity: Option<Decimal>,
    pub avg_buy_price: Option<Decimal>,
}

#[derive(Serialize, Debug)]
pub struct UserProfile {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub base_currency: String,
    pub joined_at: DateTime<Utc>,
}

#[derive(Deserialize, Debug)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Debug)]
pub struct AuthResponse {
    pub token: String,
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Asset {
    pub ticker: String,
    pub name: String,
    pub asset_type: String,
    pub api_ticker: Option<String>,
    pub source: Option<String>,
    #[serde(serialize_with = "round_currency_option")]
    pub current_price: Option<Decimal>,
}
