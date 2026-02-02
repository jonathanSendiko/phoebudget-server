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

#[derive(Deserialize, Debug)]
pub struct UpdateCurrency {
    pub base_currency: String,
}
