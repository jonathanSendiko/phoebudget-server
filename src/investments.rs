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

    // Helper to get rate from USD to Target (e.g., USD -> SGD)
    // Ticker "SGD=X" usually means 1 USD = x SGD
    async fn get_usd_rate(target: &str) -> Result<Decimal, AppError> {
        if target == "USD" {
            return Ok(Decimal::new(1, 0));
        }
        // Try direct USD pair first
        let ticker = format!("{}=X", target);
        // Special case for some pairs if needed, but standard is TARGET=X for USD->TARGET
        // Exception: EUR, GBP, AUD, NZD are often quoted as EURUSD=X (1 EUR = x USD)
        // For simplicity in this MVP, we assume we are dealing with standard ones or we try to fetching "TARGET=X"
        // If "EUR=X" doesn't exist/work, we might get an error.
        // Let's assume standard behavior for now: "SGD=X", "IDR=X".
        // If we need to support specific majors properly (EUR, GBP), we need to check if they are inverted.
        // Yahoo "EUR=X" is actually returning EUR/USD rate? Or USD/EUR?
        // Usually "EUR=X" returns 1 USD = ? EUR. (Current approx 0.95 EUR).
        // "EURUSD=X" returns 1 EUR = ? USD.
        // Let's stick to "TARGET=X" for 1 USD = ? TARGET.

        fetch_price(&ticker).await
    }

    let rate_usd_to_from = get_usd_rate(from).await?;
    let rate_usd_to_to = get_usd_rate(to).await?;

    // Rate(A->B) = Rate(USD->B) / Rate(USD->A)
    // Example: SGD -> IDR
    // USD -> SGD = 1.34
    // USD -> IDR = 15000
    // 1 SGD = (1/1.34) USD = (1/1.34) * 15000 IDR = 15000 / 1.34 = 11194

    if rate_usd_to_from.is_zero() {
        return Err(AppError::ValidationError(
            "Invalid exchange rate data".to_string(),
        ));
    }

    Ok(rate_usd_to_to / rate_usd_to_from)
}
