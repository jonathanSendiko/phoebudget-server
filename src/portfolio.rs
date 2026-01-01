//! Portfolio calculation functions
//!
//! Pure calculation functions extracted for testability.

#![allow(dead_code)] // Functions are used in tests and for potential future refactoring

use rust_decimal::Decimal;

use crate::schemas::{InvestmentSummary, PortfolioJoinedRow, PortfolioResponse};

/// Calculate investment summary for a single portfolio item
pub fn calculate_investment_summary(
    item: PortfolioJoinedRow,
    exchange_rate: Decimal,
    base_currency: &str,
) -> InvestmentSummary {
    let asset_currency = item.currency.unwrap_or_else(|| "USD".to_string());

    let current_price_native = item.current_price;
    let current_price_converted = item.current_price * exchange_rate;
    let avg_buy_converted = item.avg_buy_price * exchange_rate;

    let total_value_native = item.quantity * current_price_native;
    let total_value_converted = item.quantity * current_price_converted;

    // Calculate Change % (based on native prices - ratio is same in any currency)
    let change_pct = if item.avg_buy_price > Decimal::ZERO {
        ((current_price_native - item.avg_buy_price) / item.avg_buy_price) * Decimal::from(100)
    } else {
        Decimal::ZERO
    };

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

/// Calculate portfolio totals from a list of investment summaries
pub fn calculate_portfolio_totals(
    summaries: &[InvestmentSummary],
    exchange_rates: &std::collections::HashMap<String, Decimal>,
    base_currency: &str,
) -> (Decimal, Decimal) {
    let mut total_cost = Decimal::ZERO;
    let mut absolute_change = Decimal::ZERO;

    for summary in summaries {
        let rate = if summary.asset_currency == base_currency {
            Decimal::ONE
        } else {
            *exchange_rates
                .get(&summary.asset_currency)
                .unwrap_or(&Decimal::ONE)
        };

        let cost_converted = summary.quantity * summary.avg_buy_price * rate;
        let value_converted = summary.quantity * summary.current_price * rate;
        let change = value_converted - cost_converted;

        total_cost += cost_converted;
        absolute_change += change;
    }

    (total_cost, absolute_change)
}

/// Build a complete portfolio response from items, exchange rates, and base currency
pub fn build_portfolio_response(
    items: Vec<PortfolioJoinedRow>,
    exchange_rates: &std::collections::HashMap<String, Decimal>,
    base_currency: &str,
) -> PortfolioResponse {
    let mut summary_list = Vec::new();
    let mut total_cost = Decimal::ZERO;
    let mut absolute_change = Decimal::ZERO;

    for item in items {
        let asset_currency = item.currency.clone().unwrap_or_else(|| "USD".to_string());
        let rate = if asset_currency == base_currency {
            Decimal::ONE
        } else {
            *exchange_rates.get(&asset_currency).unwrap_or(&Decimal::ONE)
        };

        // Calculate cost and change in base currency
        let cost_converted = item.quantity * item.avg_buy_price * rate;
        let value_converted = item.quantity * item.current_price * rate;
        let change = value_converted - cost_converted;

        total_cost += cost_converted;
        absolute_change += change;

        let summary = calculate_investment_summary(item, rate, base_currency);
        summary_list.push(summary);
    }

    PortfolioResponse {
        investments: summary_list,
        total_cost,
        absolute_change,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    use std::collections::HashMap;

    fn make_test_item(
        ticker: &str,
        quantity: Decimal,
        avg_buy_price: Decimal,
        current_price: Decimal,
        currency: Option<&str>,
    ) -> PortfolioJoinedRow {
        PortfolioJoinedRow {
            ticker: ticker.to_string(),
            name: format!("{} Inc", ticker),
            quantity,
            avg_buy_price,
            current_price,
            source: Some("YAHOO".to_string()),
            api_ticker: Some(ticker.to_string()),
            currency: currency.map(|s| s.to_string()),
            icon_url: None,
        }
    }

    // ==================== Investment Summary Calculation Tests ====================

    #[test]
    fn test_calculate_investment_summary_same_currency() {
        // Given: Stock in USD, user base currency is USD
        let item = make_test_item("AAPL", dec!(10), dec!(150.00), dec!(180.00), Some("USD"));
        let rate = Decimal::ONE; // No conversion needed

        // When
        let summary = calculate_investment_summary(item, rate, "USD");

        // Then
        assert_eq!(summary.ticker, "AAPL");
        assert_eq!(summary.quantity, dec!(10));
        assert_eq!(summary.avg_buy_price, dec!(150.00));
        assert_eq!(summary.avg_buy_price_converted, dec!(150.00)); // Same as native
        assert_eq!(summary.current_price, dec!(180.00));
        assert_eq!(summary.current_price_converted, dec!(180.00)); // Same as native
        assert_eq!(summary.total_value, dec!(1800.00)); // 10 * 180
        assert_eq!(summary.total_value_converted, dec!(1800.00));
        assert_eq!(summary.change_pct, dec!(20)); // (180-150)/150 * 100 = 20%
        assert_eq!(summary.currency, "USD");
        assert_eq!(summary.asset_currency, "USD");
    }

    #[test]
    fn test_calculate_investment_summary_with_currency_conversion() {
        // Given: Stock in USD, user base currency is SGD, rate = 1.35
        let item = make_test_item("AAPL", dec!(10), dec!(150.00), dec!(180.00), Some("USD"));
        let rate = dec!(1.35); // 1 USD = 1.35 SGD

        // When
        let summary = calculate_investment_summary(item, rate, "SGD");

        // Then
        assert_eq!(summary.avg_buy_price, dec!(150.00)); // Native stays the same
        assert_eq!(summary.avg_buy_price_converted, dec!(202.50)); // 150 * 1.35
        assert_eq!(summary.current_price, dec!(180.00)); // Native stays the same
        assert_eq!(summary.current_price_converted, dec!(243.00)); // 180 * 1.35
        assert_eq!(summary.total_value, dec!(1800.00)); // Native: 10 * 180
        assert_eq!(summary.total_value_converted, dec!(2430.00)); // 1800 * 1.35
        assert_eq!(summary.change_pct, dec!(20)); // % is calculated in native, so same
        assert_eq!(summary.currency, "SGD"); // Base currency
        assert_eq!(summary.asset_currency, "USD");
    }

    #[test]
    fn test_calculate_investment_summary_negative_change() {
        // Given: Stock went down from 100 to 80
        let item = make_test_item("TSLA", dec!(5), dec!(100.00), dec!(80.00), Some("USD"));
        let rate = Decimal::ONE;

        // When
        let summary = calculate_investment_summary(item, rate, "USD");

        // Then
        assert_eq!(summary.change_pct, dec!(-20)); // (80-100)/100 * 100 = -20%
        assert_eq!(summary.total_value, dec!(400.00)); // 5 * 80
    }

    #[test]
    fn test_calculate_investment_summary_zero_avg_buy_price() {
        // Given: Edge case where avg_buy_price is 0 (maybe free shares?)
        let item = make_test_item("FREE", dec!(100), dec!(0), dec!(10.00), Some("USD"));
        let rate = Decimal::ONE;

        // When
        let summary = calculate_investment_summary(item, rate, "USD");

        // Then: Should not divide by zero, change_pct should be 0
        assert_eq!(summary.change_pct, Decimal::ZERO);
        assert_eq!(summary.total_value, dec!(1000.00));
    }

    #[test]
    fn test_calculate_investment_summary_default_currency() {
        // Given: No currency specified (should default to USD)
        let item = make_test_item("BTC", dec!(1), dec!(40000.00), dec!(45000.00), None);
        let rate = Decimal::ONE;

        // When
        let summary = calculate_investment_summary(item, rate, "USD");

        // Then
        assert_eq!(summary.asset_currency, "USD"); // Default
    }

    // ==================== Portfolio Totals Calculation Tests ====================

    #[test]
    fn test_calculate_portfolio_totals_single_item() {
        // Given: Single stock bought at $100, now worth $120
        let summaries = vec![InvestmentSummary {
            ticker: "AAPL".to_string(),
            name: "Apple".to_string(),
            quantity: dec!(10),
            avg_buy_price: dec!(100.00),
            avg_buy_price_converted: dec!(100.00),
            current_price: dec!(120.00),
            current_price_converted: dec!(120.00),
            total_value: dec!(1200.00),
            total_value_converted: dec!(1200.00),
            change_pct: dec!(20),
            currency: "USD".to_string(),
            asset_currency: "USD".to_string(),
            icon_url: None,
        }];

        let rates = HashMap::new();

        // When
        let (total_cost, absolute_change) = calculate_portfolio_totals(&summaries, &rates, "USD");

        // Then
        assert_eq!(total_cost, dec!(1000.00)); // 10 * 100
        assert_eq!(absolute_change, dec!(200.00)); // (10 * 120) - (10 * 100) = 200
    }

    #[test]
    fn test_calculate_portfolio_totals_multiple_items() {
        // Given: Two stocks
        let summaries = vec![
            InvestmentSummary {
                ticker: "AAPL".to_string(),
                name: "Apple".to_string(),
                quantity: dec!(10),
                avg_buy_price: dec!(100.00),
                avg_buy_price_converted: dec!(100.00),
                current_price: dec!(120.00),
                current_price_converted: dec!(120.00),
                total_value: dec!(1200.00),
                total_value_converted: dec!(1200.00),
                change_pct: dec!(20),
                currency: "USD".to_string(),
                asset_currency: "USD".to_string(),
                icon_url: None,
            },
            InvestmentSummary {
                ticker: "GOOGL".to_string(),
                name: "Google".to_string(),
                quantity: dec!(5),
                avg_buy_price: dec!(200.00),
                avg_buy_price_converted: dec!(200.00),
                current_price: dec!(180.00), // Lost value
                current_price_converted: dec!(180.00),
                total_value: dec!(900.00),
                total_value_converted: dec!(900.00),
                change_pct: dec!(-10),
                currency: "USD".to_string(),
                asset_currency: "USD".to_string(),
                icon_url: None,
            },
        ];

        let rates = HashMap::new();

        // When
        let (total_cost, absolute_change) = calculate_portfolio_totals(&summaries, &rates, "USD");

        // Then
        // AAPL: cost = 1000, value = 1200, change = +200
        // GOOGL: cost = 1000, value = 900, change = -100
        assert_eq!(total_cost, dec!(2000.00));
        assert_eq!(absolute_change, dec!(100.00)); // 200 - 100 = 100
    }

    // ==================== Full Portfolio Response Tests ====================

    #[test]
    fn test_build_portfolio_response_empty() {
        let items = vec![];
        let rates = HashMap::new();

        let response = build_portfolio_response(items, &rates, "USD");

        assert!(response.investments.is_empty());
        assert_eq!(response.total_cost, Decimal::ZERO);
        assert_eq!(response.absolute_change, Decimal::ZERO);
    }

    #[test]
    fn test_build_portfolio_response_with_items() {
        let items = vec![
            make_test_item("AAPL", dec!(10), dec!(100.00), dec!(120.00), Some("USD")),
            make_test_item("GOOGL", dec!(5), dec!(200.00), dec!(180.00), Some("USD")),
        ];
        let rates = HashMap::new();

        let response = build_portfolio_response(items, &rates, "USD");

        assert_eq!(response.investments.len(), 2);
        assert_eq!(response.total_cost, dec!(2000.00));
        assert_eq!(response.absolute_change, dec!(100.00));
    }

    #[test]
    fn test_build_portfolio_response_with_currency_conversion() {
        // Given: USD stock, user base is SGD, rate = 1.35
        let items = vec![make_test_item(
            "AAPL",
            dec!(10),
            dec!(100.00),
            dec!(120.00),
            Some("USD"),
        )];
        let mut rates = HashMap::new();
        rates.insert("USD".to_string(), dec!(1.35));

        let response = build_portfolio_response(items, &rates, "SGD");

        // Cost in SGD: 10 * 100 * 1.35 = 1350
        // Value in SGD: 10 * 120 * 1.35 = 1620
        // Change: 1620 - 1350 = 270
        assert_eq!(response.total_cost, dec!(1350.00));
        assert_eq!(response.absolute_change, dec!(270.00));

        // Check individual investment
        let inv = &response.investments[0];
        assert_eq!(inv.avg_buy_price_converted, dec!(135.00)); // 100 * 1.35
        assert_eq!(inv.current_price_converted, dec!(162.00)); // 120 * 1.35
        assert_eq!(inv.total_value_converted, dec!(1620.00)); // 10 * 162
    }
}
