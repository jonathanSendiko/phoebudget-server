#![cfg(test)]

use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::HashMap;

use crate::schemas::{InvestmentSummary, PortfolioJoinedRow};

use super::{build_portfolio_response, calculate_investment_summary, calculate_portfolio_totals};

// ============================================================================
// Test Helpers
// ============================================================================

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

/// Create a test investment summary.
fn make_test_summary(
    ticker: &str,
    quantity: Decimal,
    avg_buy_price: Decimal,
    current_price: Decimal,
    asset_currency: &str,
) -> InvestmentSummary {
    InvestmentSummary {
        ticker: ticker.to_string(),
        name: format!("{} Inc", ticker),
        quantity,
        avg_buy_price,
        avg_buy_price_converted: avg_buy_price,
        current_price,
        current_price_converted: current_price,
        total_value: quantity * current_price,
        total_value_converted: quantity * current_price,
        change_pct: if avg_buy_price > Decimal::ZERO {
            ((current_price - avg_buy_price) / avg_buy_price) * dec!(100)
        } else {
            Decimal::ZERO
        },
        currency: "USD".to_string(),
        asset_currency: asset_currency.to_string(),
        icon_url: None,
    }
}

// ============================================================================
// Investment Summary Calculation Tests
// ============================================================================

mod investment_summary {
    use super::*;

    #[test]
    fn same_currency_no_conversion() {
        // Given: Stock in USD, user base currency is USD
        let item = make_test_item("AAPL", dec!(10), dec!(150.00), dec!(180.00), Some("USD"));

        // When
        let summary = calculate_investment_summary(item, Decimal::ONE, "USD");

        // Then: Native and converted values should be identical
        assert_eq!(summary.ticker, "AAPL");
        assert_eq!(summary.quantity, dec!(10));
        assert_eq!(summary.avg_buy_price, dec!(150.00));
        assert_eq!(summary.avg_buy_price_converted, dec!(150.00));
        assert_eq!(summary.current_price, dec!(180.00));
        assert_eq!(summary.current_price_converted, dec!(180.00));
        assert_eq!(summary.total_value, dec!(1800.00));
        assert_eq!(summary.total_value_converted, dec!(1800.00));
        assert_eq!(summary.change_pct, dec!(20)); // (180-150)/150 * 100
        assert_eq!(summary.currency, "USD");
        assert_eq!(summary.asset_currency, "USD");
    }

    #[test]
    fn with_currency_conversion() {
        // Given: Stock in USD, user base currency is SGD
        let item = make_test_item("AAPL", dec!(10), dec!(150.00), dec!(180.00), Some("USD"));
        let rate = dec!(1.35); // 1 USD = 1.35 SGD

        // When
        let summary = calculate_investment_summary(item, rate, "SGD");

        // Then: Native values unchanged, converted values multiplied by rate
        assert_eq!(summary.avg_buy_price, dec!(150.00));
        assert_eq!(summary.avg_buy_price_converted, dec!(202.50)); // 150 * 1.35
        assert_eq!(summary.current_price, dec!(180.00));
        assert_eq!(summary.current_price_converted, dec!(243.00)); // 180 * 1.35
        assert_eq!(summary.total_value, dec!(1800.00));
        assert_eq!(summary.total_value_converted, dec!(2430.00)); // 1800 * 1.35
        assert_eq!(summary.change_pct, dec!(20)); // Percentage unchanged
        assert_eq!(summary.currency, "SGD");
        assert_eq!(summary.asset_currency, "USD");
    }

    #[test]
    fn negative_change() {
        // Given: Stock went down 20%
        let item = make_test_item("TSLA", dec!(5), dec!(100.00), dec!(80.00), Some("USD"));

        // When
        let summary = calculate_investment_summary(item, Decimal::ONE, "USD");

        // Then
        assert_eq!(summary.change_pct, dec!(-20));
        assert_eq!(summary.total_value, dec!(400.00));
    }

    #[test]
    fn zero_avg_buy_price_no_divide_by_zero() {
        // Given: Edge case with zero cost basis (free shares)
        let item = make_test_item("FREE", dec!(100), dec!(0), dec!(10.00), Some("USD"));

        // When
        let summary = calculate_investment_summary(item, Decimal::ONE, "USD");

        // Then: Should not panic, change_pct should be 0
        assert_eq!(summary.change_pct, Decimal::ZERO);
        assert_eq!(summary.total_value, dec!(1000.00));
    }

    #[test]
    fn default_currency_when_none() {
        // Given: No currency specified
        let item = make_test_item("BTC", dec!(1), dec!(40000.00), dec!(45000.00), None);

        // When
        let summary = calculate_investment_summary(item, Decimal::ONE, "USD");

        // Then: Should default to USD
        assert_eq!(summary.asset_currency, "USD");
    }
}

// ============================================================================
// Portfolio Totals Calculation Tests
// ============================================================================

mod portfolio_totals {
    use super::*;

    #[test]
    fn single_item() {
        // Given: One stock, cost $1000, now worth $1200
        let summaries = vec![make_test_summary(
            "AAPL",
            dec!(10),
            dec!(100.00),
            dec!(120.00),
            "USD",
        )];

        // When
        let (total_cost, absolute_change) =
            calculate_portfolio_totals(&summaries, &HashMap::new(), "USD");

        // Then
        assert_eq!(total_cost, dec!(1000.00));
        assert_eq!(absolute_change, dec!(200.00));
    }

    #[test]
    fn multiple_items_mixed_performance() {
        // Given: Two stocks - one up, one down
        let summaries = vec![
            make_test_summary("AAPL", dec!(10), dec!(100.00), dec!(120.00), "USD"), // +$200
            make_test_summary("GOOGL", dec!(5), dec!(200.00), dec!(180.00), "USD"), // -$100
        ];

        // When
        let (total_cost, absolute_change) =
            calculate_portfolio_totals(&summaries, &HashMap::new(), "USD");

        // Then
        assert_eq!(total_cost, dec!(2000.00)); // 1000 + 1000
        assert_eq!(absolute_change, dec!(100.00)); // 200 - 100
    }
}

// ============================================================================
// Full Portfolio Response Tests
// ============================================================================

mod portfolio_response {
    use super::*;

    #[test]
    fn empty_portfolio() {
        let response = build_portfolio_response(vec![], &HashMap::new(), "USD");

        assert!(response.investments.is_empty());
        assert_eq!(response.total_cost, Decimal::ZERO);
        assert_eq!(response.absolute_change, Decimal::ZERO);
    }

    #[test]
    fn with_multiple_items() {
        let items = vec![
            make_test_item("AAPL", dec!(10), dec!(100.00), dec!(120.00), Some("USD")),
            make_test_item("GOOGL", dec!(5), dec!(200.00), dec!(180.00), Some("USD")),
        ];

        let response = build_portfolio_response(items, &HashMap::new(), "USD");

        assert_eq!(response.investments.len(), 2);
        assert_eq!(response.total_cost, dec!(2000.00));
        assert_eq!(response.absolute_change, dec!(100.00));
    }

    #[test]
    fn with_currency_conversion() {
        // Given: USD stock, user base is SGD
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

        // Cost: 10 * 100 * 1.35 = 1350 SGD
        // Value: 10 * 120 * 1.35 = 1620 SGD
        // Change: 1620 - 1350 = 270 SGD
        assert_eq!(response.total_cost, dec!(1350.00));
        assert_eq!(response.absolute_change, dec!(270.00));

        // Check converted values in summary
        let inv = &response.investments[0];
        assert_eq!(inv.avg_buy_price_converted, dec!(135.00));
        assert_eq!(inv.current_price_converted, dec!(162.00));
        assert_eq!(inv.total_value_converted, dec!(1620.00));
    }
}
