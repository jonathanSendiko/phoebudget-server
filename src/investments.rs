use crate::error::AppError;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive; // Required for from_f64
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct BinanceTickerResponse {
    #[allow(dead_code)]
    symbol: String,
    price: String,
}

pub async fn fetch_price(ticker: &str) -> Result<Decimal, AppError> {
    // Strategy: Try Yahoo Finance first. If it fails, try Binance.

    // 1. Try Yahoo Finance
    match fetch_price_yahoo(ticker).await {
        Ok(price) => return Ok(price),
        Err(e) => {
            tracing::warn!(
                "Yahoo Finance failed for {}: {:?}. Attempting Binance...",
                ticker,
                e
            );
        }
    }

    // 2. Try Binance
    // Binance tickers are strictly uppercase and usually just the pair (e.g. BTCUSDT, ETHBTC).
    // If the user provided "BTC-USD" (Yahoo style), Binance might not like it.
    // But if they provided "ASTERUSDT" (Crypto style), Yahoo failed, so we try Binance.
    // We try the ticker as-is.
    fetch_price_binance(ticker).await
}

// Internal structs for Yahoo API response parsing
#[derive(Deserialize, Debug)]
struct YahooResponse {
    chart: YahooChart,
}

#[derive(Deserialize, Debug)]
struct YahooChart {
    result: Option<Vec<YahooResult>>,
    error: Option<YahooErrorDetails>,
}

#[derive(Deserialize, Debug)]
struct YahooResult {
    meta: YahooMeta,
}

#[derive(Deserialize, Debug)]
struct YahooErrorDetails {
    code: String,
    description: String,
}

#[derive(Deserialize, Debug)]
struct YahooMeta {
    #[serde(rename = "regularMarketPrice")]
    regular_market_price: f64,
}

async fn fetch_price_yahoo(ticker: &str) -> Result<Decimal, AppError> {
    let url = format!(
        "https://query1.finance.yahoo.com/v8/finance/chart/{}?interval=1d&range=1m",
        ticker
    );

    // use a standard browser user-agent to avoid 429/403
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .build()
        .map_err(|e| AppError::ValidationError(format!("Failed to build HTTP client: {}", e)))?;

    let resp =
        client.get(&url).send().await.map_err(|e| {
            AppError::ValidationError(format!("Yahoo API connection failed: {}", e))
        })?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(AppError::ValidationError(format!(
            "Yahoo API returned error {}: {}",
            status, text
        )));
    }

    let yahoo_data: YahooResponse = resp
        .json()
        .await
        .map_err(|e| AppError::ValidationError(format!("Failed to parse Yahoo response: {}", e)))?;

    if let Some(err) = yahoo_data.chart.error {
        return Err(AppError::ValidationError(format!(
            "Yahoo API returned explicit error: {} - {}",
            err.code, err.description
        )));
    }

    let result = yahoo_data
        .chart
        .result
        .and_then(|r| r.into_iter().next())
        .ok_or_else(|| AppError::ValidationError(format!("No data found for {}", ticker)))?;

    Decimal::from_f64(result.meta.regular_market_price)
        .ok_or_else(|| AppError::ValidationError("Failed to parse price".to_string()))
}

async fn fetch_price_binance(ticker: &str) -> Result<Decimal, AppError> {
    let url = format!(
        "https://api.binance.com/api/v3/ticker/price?symbol={}",
        ticker.to_uppercase()
    );

    let resp = reqwest::get(&url)
        .await
        .map_err(|e| AppError::ValidationError(format!("Binance API connection failed: {}", e)))?;

    if !resp.status().is_success() {
        return Err(AppError::ValidationError(format!(
            "Binance API returned error for {}",
            ticker
        )));
    }

    let ticker_data: BinanceTickerResponse = resp.json().await.map_err(|e| {
        AppError::ValidationError(format!("Failed to parse Binance response: {}", e))
    })?;

    // Binance returns price as "0.69300000" (String)
    ticker_data.price.parse::<Decimal>().map_err(|_| {
        AppError::ValidationError(format!(
            "Failed to parse Binance price '{}'",
            ticker_data.price
        ))
    })
}

// Internal structs for Frankfurter API response parsing
#[derive(Deserialize, Debug)]
struct FrankfurterResponse {
    rates: std::collections::HashMap<String, f64>,
}

pub async fn fetch_exchange_rate(from: &str, to: &str) -> Result<Decimal, AppError> {
    if from == to {
        return Ok(Decimal::new(1, 0));
    }

    let url = format!("https://api.frankfurter.app/latest?from={}&to={}", from, to);

    let client = reqwest::Client::new();
    let resp = client.get(&url).send().await.map_err(|e| {
        AppError::ValidationError(format!("Frankfurter API connection failed: {}", e))
    })?;

    if !resp.status().is_success() {
        return Err(AppError::ValidationError(format!(
            "Frankfurter API returned error: {}",
            resp.status()
        )));
    }

    let data: FrankfurterResponse = resp.json().await.map_err(|e| {
        AppError::ValidationError(format!("Failed to parse Frankfurter response: {}", e))
    })?;

    let rate = data.rates.get(to).ok_or_else(|| {
        AppError::ValidationError(format!("No rate found for {} -> {}", from, to))
    })?;

    Decimal::from_f64(*rate)
        .ok_or_else(|| AppError::ValidationError("Failed to parse exchange rate".to_string()))
}
