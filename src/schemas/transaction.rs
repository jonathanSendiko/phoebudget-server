use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::common::{round_currency, round_currency_option};
use super::pocket::PocketSummary;

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

#[derive(Deserialize)]
pub struct TransactionQueryParams {
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub pocket_id: Option<Uuid>,
    pub search: Option<String>,
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

#[derive(Deserialize, Debug)]
pub struct TransferRequest {
    pub source_pocket_id: Uuid,
    pub destination_pocket_id: Uuid,
    pub amount: Decimal,
    pub description: Option<String>,
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
    pub pocket: Option<PocketSummary>,
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

#[derive(Serialize, Debug, sqlx::FromRow)]
pub struct Category {
    pub id: i32,
    pub name: String,
    pub is_income: bool,
    pub icon: String,
    #[serde(default)]
    pub exclude_from_analysis: bool,
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
