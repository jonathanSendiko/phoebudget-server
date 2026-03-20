use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::common::{round_currency, round_currency_option};

#[derive(Deserialize, Debug, Clone)]
pub struct CreatePortfolioItem {
    pub ticker: String,
    pub quantity: Decimal,
    pub avg_buy_price: Decimal,
}

#[derive(Deserialize, Debug)]
pub struct UpdateInvestment {
    pub quantity: Option<Decimal>,
    pub avg_buy_price: Option<Decimal>,
}

#[derive(Serialize, Debug, Clone)]
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
    #[serde(serialize_with = "round_currency")]
    pub absolute_change: Decimal, // Base
    #[serde(serialize_with = "round_currency")]
    pub portfolio_pct: Decimal, // Percentage of total portfolio
    pub currency: String,       // Base Currency
    pub base_currency: String,  // Base Currency
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
