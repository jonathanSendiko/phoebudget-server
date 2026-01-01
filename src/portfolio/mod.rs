mod tests;

use rust_decimal::Decimal;
use std::collections::HashMap;

use crate::schemas::{InvestmentSummary, PortfolioJoinedRow, PortfolioResponse};

pub fn calculate_change_percent(current_price: Decimal, base_price: Decimal) -> Decimal {
    if base_price > Decimal::ZERO {
        ((current_price - base_price) / base_price) * Decimal::from(100)
    } else {
        Decimal::ZERO
    }
}

pub fn calculate_investment_summary(
    item: PortfolioJoinedRow,
    exchange_rate: Decimal,
    base_currency: &str,
) -> InvestmentSummary {
    let asset_currency = item.currency.unwrap_or_else(|| "USD".to_string());

    let current_price_native = item.current_price;
    let total_value_native = item.quantity * current_price_native;

    let current_price_converted = item.current_price * exchange_rate;
    let avg_buy_converted = item.avg_buy_price * exchange_rate;
    let total_value_converted = item.quantity * current_price_converted;

    let change_pct = calculate_change_percent(current_price_native, item.avg_buy_price);

    InvestmentSummary {
        ticker: item.ticker,
        name: item.name,
        quantity: item.quantity,
        avg_buy_price: item.avg_buy_price,
        avg_buy_price_converted: avg_buy_converted,
        current_price: current_price_native,
        current_price_converted,
        total_value: total_value_native,
        total_value_converted,
        change_pct,
        currency: base_currency.to_string(),
        asset_currency,
        icon_url: item.icon_url,
    }
}

fn get_exchange_rate(
    from_currency: &str,
    to_currency: &str,
    rates: &HashMap<String, Decimal>,
) -> Decimal {
    if from_currency == to_currency {
        Decimal::ONE
    } else {
        *rates.get(from_currency).unwrap_or(&Decimal::ONE)
    }
}

pub fn build_portfolio_response(
    items: Vec<PortfolioJoinedRow>,
    exchange_rates: &HashMap<String, Decimal>,
    base_currency: &str,
) -> PortfolioResponse {
    let mut summary_list = Vec::with_capacity(items.len());
    let mut total_cost = Decimal::ZERO;
    let mut absolute_change = Decimal::ZERO;

    for item in items {
        let asset_currency = item.currency.clone().unwrap_or_else(|| "USD".to_string());
        let rate = get_exchange_rate(&asset_currency, base_currency, exchange_rates);

        // Accumulate totals in base currency
        let cost_converted = item.quantity * item.avg_buy_price * rate;
        let value_converted = item.quantity * item.current_price * rate;

        total_cost += cost_converted;
        absolute_change += value_converted - cost_converted;

        let summary = calculate_investment_summary(item, rate, base_currency);
        summary_list.push(summary);
    }

    PortfolioResponse {
        investments: summary_list,
        total_cost,
        absolute_change,
    }
}

// Helper for tests only - not used in production but tests portfolio totals calculation
#[cfg(test)]
pub fn calculate_portfolio_totals(
    summaries: &[InvestmentSummary],
    exchange_rates: &HashMap<String, Decimal>,
    base_currency: &str,
) -> (Decimal, Decimal) {
    let mut total_cost = Decimal::ZERO;
    let mut absolute_change = Decimal::ZERO;

    for summary in summaries {
        let rate = get_exchange_rate(&summary.asset_currency, base_currency, exchange_rates);

        let cost_converted = summary.quantity * summary.avg_buy_price * rate;
        let value_converted = summary.quantity * summary.current_price * rate;

        total_cost += cost_converted;
        absolute_change += value_converted - cost_converted;
    }

    (total_cost, absolute_change)
}
