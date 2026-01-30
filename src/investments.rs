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

pub async fn fetch_price_with_source(
    client: &reqwest::Client,
    _ticker: &str, // Original ticker (e.g. BTC) - unused for fetching but good for logging
    api_ticker: &str,
    source: &str,
    itick_api_key: Option<&str>,
) -> Result<(Decimal, String), AppError> {
    match source {
        "ITICK" => {
            let api_key = itick_api_key.ok_or_else(|| {
                AppError::ValidationError("ITICK_API_KEY not configured".to_string())
            })?;
            // Parse api_ticker format: "REGION:CODE" (e.g., "US:AAPL") or just "CODE" (defaults to US)
            let (region, code) = if api_ticker.contains(':') {
                let parts: Vec<&str> = api_ticker.splitn(2, ':').collect();
                (parts[0], parts[1])
            } else {
                ("US", api_ticker)
            };
            fetch_price_itick(client, code, region, api_key).await
        }
        "YAHOO" => fetch_price_yahoo(client, api_ticker).await,
        "BINANCE" => fetch_price_binance(client, api_ticker)
            .await
            .map(|p| (p, "USD".to_string())), // Assuming USDT
        "COINGECKO" => fetch_price_coingecko(client, api_ticker)
            .await
            .map(|p| (p, "USD".to_string())),
        _ => {
            // Fallback or Error?
            // "Invalid Source"
            Err(AppError::ValidationError(format!(
                "Unknown price source: {}",
                source
            )))
        }
    }
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
    currency: Option<String>,
}

// Internal structs for CoinGecko API response parsing
// Response format: {"umbra-network": {"usd": 1.23}}
#[derive(Deserialize, Debug)]
struct CoinGeckoResponse(std::collections::HashMap<String, CoinGeckoPrice>);

#[derive(Deserialize, Debug)]
struct CoinGeckoPrice {
    usd: f64,
}

// Internal structs for iTick API response parsing
// Response format: {"code": 0, "msg": null, "data": {"s": "AAPL", "ld": 258.28, ...}}
#[derive(Deserialize, Debug)]
struct ITickResponse {
    code: i32,
    #[allow(dead_code)]
    msg: Option<String>,
    data: Option<ITickQuoteData>,
}

#[derive(Deserialize, Debug)]
struct ITickQuoteData {
    #[allow(dead_code)]
    s: String, // Symbol
    ld: f64, // Latest price (last done)
    #[allow(dead_code)]
    r: Option<String>, // Region
}

async fn fetch_price_itick(
    client: &reqwest::Client,
    code: &str,
    region: &str,
    api_key: &str,
) -> Result<(Decimal, String), AppError> {
    let url = format!(
        "https://api-free.itick.org/stock/quote?region={}&code={}",
        region, code
    );

    let resp = client
        .get(&url)
        .header("token", api_key)
        .header("accept", "application/json")
        .send()
        .await
        .map_err(|e| AppError::ValidationError(format!("iTick API connection failed: {}", e)))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(AppError::ValidationError(format!(
            "iTick API returned error {}: {}",
            status, text
        )));
    }

    let itick_data: ITickResponse = resp
        .json()
        .await
        .map_err(|e| AppError::ValidationError(format!("Failed to parse iTick response: {}", e)))?;

    if itick_data.code != 0 {
        return Err(AppError::ValidationError(format!(
            "iTick API returned error code: {}",
            itick_data.code
        )));
    }

    let data = itick_data.data.ok_or_else(|| {
        AppError::ValidationError(format!("No data found for {}:{}", region, code))
    })?;

    let price = Decimal::from_f64(data.ld)
        .ok_or_else(|| AppError::ValidationError("Failed to parse iTick price".to_string()))?;

    // Determine currency based on region
    let currency = match region.to_uppercase().as_str() {
        "US" => "USD",
        "HK" => "HKD",
        "SG" => "SGD",
        "ID" => "IDR",
        "CN" | "SH" | "SZ" => "CNY",
        "JP" => "JPY",
        "TW" => "TWD",
        "IN" => "INR",
        "TH" => "THB",
        "DE" => "EUR",
        "GB" => "GBP",
        "AU" => "AUD",
        "CA" => "CAD",
        _ => "USD", // Default to USD for unknown regions
    };

    Ok((price, currency.to_string()))
}

async fn fetch_price_yahoo(
    client: &reqwest::Client,
    ticker: &str,
) -> Result<(Decimal, String), AppError> {
    let url = format!(
        "https://query1.finance.yahoo.com/v8/finance/chart/{}?interval=1d&range=1m",
        ticker
    );

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

    let price = Decimal::from_f64(result.meta.regular_market_price)
        .ok_or_else(|| AppError::ValidationError("Failed to parse price".to_string()))?;

    let currency = result.meta.currency.unwrap_or_else(|| "USD".to_string());

    Ok((price, currency))
}

async fn fetch_price_binance(client: &reqwest::Client, ticker: &str) -> Result<Decimal, AppError> {
    let url = format!(
        "https://api.binance.com/api/v3/ticker/price?symbol={}",
        ticker.to_uppercase()
    );

    let resp =
        client.get(&url).send().await.map_err(|e| {
            AppError::ValidationError(format!("Binance API connection failed: {}", e))
        })?;

    if !resp.status().is_success() {
        return Err(AppError::ValidationError(format!(
            "Binance API returned error for {}",
            ticker
        )));
    }

    let ticker_data: BinanceTickerResponse = resp.json().await.map_err(|e| {
        AppError::ValidationError(format!("Failed to parse Binance response: {}", e))
    })?;

    ticker_data.price.parse::<Decimal>().map_err(|_| {
        AppError::ValidationError(format!(
            "Failed to parse Binance price '{}'",
            ticker_data.price
        ))
    })
}

async fn fetch_price_coingecko(
    client: &reqwest::Client,
    ticker: &str,
) -> Result<Decimal, AppError> {
    let id = ticker.to_lowercase();

    let url = format!(
        "https://api.coingecko.com/api/v3/simple/price?ids={}&vs_currencies=usd",
        id
    );

    let resp = client.get(&url).send().await.map_err(|e| {
        AppError::ValidationError(format!("CoinGecko API connection failed: {}", e))
    })?;

    if !resp.status().is_success() {
        return Err(AppError::ValidationError(format!(
            "CoinGecko API returned error: {}",
            resp.status()
        )));
    }

    let data: CoinGeckoResponse = resp.json().await.map_err(|e| {
        AppError::ValidationError(format!("Failed to parse CoinGecko response: {}", e))
    })?;

    let price_item = data.0.get(&id).ok_or_else(|| {
        AppError::ValidationError(format!("CoinGecko: No price found for ID '{}'", id))
    })?;

    Decimal::from_f64(price_item.usd)
        .ok_or_else(|| AppError::ValidationError("Failed to parse CoinGecko price".to_string()))
}

// Internal structs for CoinGecko icon API response
#[derive(Deserialize, Debug)]
struct CoinGeckoIconResponse {
    image: CoinGeckoImage,
}

#[derive(Deserialize, Debug)]
struct CoinGeckoImage {
    large: String,
}

/// Fetch icon URL from CoinGecko API for a given coin ID
pub async fn fetch_coingecko_icon(
    client: &reqwest::Client,
    coin_id: &str,
) -> Result<Option<String>, AppError> {
    let id = coin_id.to_lowercase();
    let url = format!("https://api.coingecko.com/api/v3/coins/{}", id);

    let resp = client.get(&url).send().await.map_err(|e| {
        tracing::warn!("CoinGecko icon API connection failed for {}: {}", id, e);
        return AppError::ValidationError(format!("CoinGecko icon API connection failed: {}", e));
    })?;

    if !resp.status().is_success() {
        tracing::warn!(
            "CoinGecko icon API returned error {} for {}",
            resp.status(),
            id
        );
        // Return None instead of error - icon is optional
        return Ok(None);
    }

    let data: CoinGeckoIconResponse = match resp.json().await {
        Ok(d) => d,
        Err(e) => {
            tracing::warn!("Failed to parse CoinGecko icon response for {}: {}", id, e);
            return Ok(None);
        }
    };

    Ok(Some(data.image.large))
}

// Internal structs for Frankfurter API response parsing
#[derive(Deserialize, Debug)]
struct FrankfurterResponse {
    rates: std::collections::HashMap<String, f64>,
}

pub async fn fetch_exchange_rate(
    client: &reqwest::Client,
    from: &str,
    to: &str,
) -> Result<Decimal, AppError> {
    if from == to {
        return Ok(Decimal::new(1, 0));
    }

    let url = format!("https://api.frankfurter.app/latest?from={}&to={}", from, to);

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
