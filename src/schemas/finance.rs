use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::common::round_currency;

#[derive(Serialize, Debug)]
pub struct FinancialHealth {
    #[serde(serialize_with = "round_currency")]
    pub cash_balance: Decimal,
    #[serde(serialize_with = "round_currency")]
    pub investment_balance: Decimal,
    #[serde(serialize_with = "round_currency")]
    pub total_net_worth: Decimal,
}

#[derive(Serialize, Debug, Clone)]
pub struct NetWorthHistoryPoint {
    pub month: String,
    #[serde(serialize_with = "round_currency")]
    pub total_income: Decimal,
    #[serde(serialize_with = "round_currency")]
    pub total_spent: Decimal,
    #[serde(serialize_with = "round_currency")]
    pub net_change: Decimal,
    #[serde(serialize_with = "round_currency")]
    pub net_worth_end: Decimal,
}

#[derive(Serialize, Debug)]
pub struct NetWorthHistoryResponse {
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    #[serde(serialize_with = "round_currency")]
    pub opening_balance: Decimal,
    pub points: Vec<NetWorthHistoryPoint>,
}

#[derive(Debug, Clone)]
pub struct MonthlyCashFlowRow {
    pub month: DateTime<Utc>,
    pub total_income: Decimal,
    pub total_spent: Decimal,
}

#[derive(Deserialize, Debug)]
pub struct UpdateCurrency {
    pub base_currency: String,
}
