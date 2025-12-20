use crate::error::AppError;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive; // Required for from_f64
use yahoo_finance_api::YahooConnector;

pub async fn fetch_price(ticker: &str) -> Result<Decimal, AppError> {
    // 1. Initialize the provider
    let provider = YahooConnector::new().map_err(|e| {
        AppError::ValidationError(format!("Failed to initialize Yahoo Connector: {}", e))
    })?;

    // 2. Request data: "1d" range (1 day), "1m" interval (1 minute)
    // This gives us the most recent trading data.
    let response = provider
        .get_quote_range(ticker, "1d", "1m")
        .await
        .map_err(|e| AppError::ValidationError(format!("Yahoo API Error for {}: {}", ticker, e)))?;

    // 3. Extract the last available quote
    let quote = response
        .last_quote()
        .map_err(|_| AppError::ValidationError(format!("No price data found for {}", ticker)))?;

    // 4. Convert f64 (Float) to Decimal (Money Safe)
    // Yahoo returns floats, but our DB uses Decimals.
    Decimal::from_f64(quote.close)
        .ok_or_else(|| AppError::ValidationError("Failed to parse price".to_string()))
}

pub async fn fetch_exchange_rate(from: &str, to: &str) -> Result<Decimal, AppError> {
    if from == to {
        return Ok(Decimal::new(1, 0)); // 1.0
    }

    // Yahoo Ticker for pair: e.g. "SGD=X" generally means USD -> SGD.
    // "EURUSD=X" means EUR -> USD.
    // Standard convention on Yahoo for USD base is "CURRENCY=X" (e.g. converted FROM USD).
    // If we want USD -> SGD, ticker is "SGD=X".
    // If we want SGD -> USD, we might need to invert or find specific ticker.
    // For this MVP, let's assume valid pairs for "USD" -> "Target".
    let ticker = format!("{}=X", to);

    let provider = YahooConnector::new().map_err(|e| {
        AppError::ValidationError(format!("Failed to initialize Yahoo Connector: {}", e))
    })?;

    let response = provider
        .get_quote_range(&ticker, "1d", "1m")
        .await
        .map_err(|e| AppError::ValidationError(format!("Yahoo API Error for {}: {}", ticker, e)))?;

    let quote = response
        .last_quote()
        .map_err(|_| AppError::ValidationError(format!("No exchange rate found for {}", ticker)))?;

    Decimal::from_f64(quote.close)
        .ok_or_else(|| AppError::ValidationError("Failed to parse exchange rate".to_string()))
}
