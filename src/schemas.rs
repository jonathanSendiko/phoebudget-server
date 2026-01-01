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
    pub pocket_id: Option<Uuid>,
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
pub struct TransactionQueryParams {
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub pocket_id: Option<Uuid>,
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_limit")]
    pub limit: i64,
}

fn default_page() -> i64 {
    1
}

fn default_limit() -> i64 {
    10
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
    pub category: Option<Category>,
    pub pocket: Option<PocketSummary>,
    pub occurred_at: DateTime<Utc>,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, Debug)]
pub struct PaginatedTransactions {
    pub transactions: Vec<Transaction>,
    pub total: i64,
    pub page: i64,
    pub limit: i64,
    pub total_pages: i64,
}

#[derive(Serialize, Debug)]
pub struct TransactionDetail {
    pub id: Uuid,
    #[serde(serialize_with = "round_currency")]
    pub amount: Decimal,
    pub description: Option<String>,
    pub category: Option<Category>,
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

// --- Pocket DTOs ---

#[derive(Deserialize, Debug)]
pub struct CreatePocket {
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct UpdatePocket {
    pub name: Option<String>,
    pub description: Option<String>,
    pub icon: Option<String>,
}

#[derive(Serialize, Debug, Clone)]
pub struct Pocket {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub icon: String,
    pub is_default: bool,
    pub created_at: Option<DateTime<Utc>>,
}

/// Lightweight pocket info for embedding in transactions
#[derive(Serialize, Debug, Clone)]
pub struct PocketSummary {
    pub id: Uuid,
    pub name: String,
    pub icon: String,
}

#[derive(Serialize)]
pub struct PocketId {
    pub id: Uuid,
}

#[derive(Serialize, Debug)]
pub struct CategorySummary {
    pub category: String,
    #[serde(serialize_with = "round_currency")]
    pub total: Decimal,
    pub is_income: bool,
    pub icon: String,
}

#[derive(Serialize, Debug)]
pub struct SpendingAnalysisResponse {
    #[serde(serialize_with = "round_currency")]
    pub total_income: Decimal,
    #[serde(serialize_with = "round_currency")]
    pub total_spent: Decimal,
    #[serde(serialize_with = "round_currency")]
    pub net_income: Decimal,
    pub categories: Vec<CategorySummary>,
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
    pub avg_buy_price: Decimal, // Native
    #[serde(serialize_with = "round_currency")]
    pub avg_buy_price_converted: Decimal, // Base
    #[serde(serialize_with = "round_currency")]
    pub current_price: Decimal, // Native
    #[serde(serialize_with = "round_currency")]
    pub current_price_converted: Decimal, // Base
    #[serde(serialize_with = "round_currency")]
    pub total_value: Decimal, // Native
    #[serde(serialize_with = "round_currency")]
    pub total_value_converted: Decimal, // Base
    #[serde(serialize_with = "round_currency")]
    pub change_pct: Decimal,
    pub currency: String,       // Base Currency
    pub asset_currency: String, // Native Currency
    pub icon_url: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct PortfolioResponse {
    pub investments: Vec<InvestmentSummary>,
    #[serde(serialize_with = "round_currency")]
    pub total_cost: Decimal,
    #[serde(serialize_with = "round_currency")]
    pub absolute_change: Decimal,
    #[serde(serialize_with = "round_currency")]
    pub total_value: Decimal,
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
    pub currency: Option<String>,
    pub icon_url: Option<String>,
}

#[derive(Serialize, Debug, sqlx::FromRow)]
pub struct Category {
    pub id: i32,
    pub name: String,
    pub is_income: bool,
    pub icon: String,
}

/// Internal struct for portfolio data joined with asset info (from repository)
#[derive(Debug)]
pub struct PortfolioJoinedRow {
    pub ticker: String,
    pub name: String,
    pub quantity: Decimal,
    pub avg_buy_price: Decimal,
    pub current_price: Decimal,
    pub source: Option<String>,
    pub api_ticker: Option<String>,
    pub currency: Option<String>,
    pub icon_url: Option<String>,
}
