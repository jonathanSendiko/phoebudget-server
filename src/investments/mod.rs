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
        "STOOQ" => fetch_price_stooq(client, api_ticker)
            .await
            .map(|p| (p, "USD".to_string())),
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

async fn fetch_price_stooq(
    client: &reqwest::Client,
    api_ticker: &str,
) -> Result<Decimal, AppError> {
    // api_ticker examples: "voo.us", "spy.us", "aapl.us"
    let symbol = api_ticker.trim().to_lowercase();
    let url = format!(
        "https://stooq.com/q/l/?s={}&f=sd2t2ohlcv&h&e=csv",
        symbol
    );

    let resp = client
        .get(&url)
        .header(
            reqwest::header::USER_AGENT,
            "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0 Safari/537.36",
        )
        .send()
        .await
        .map_err(|e| AppError::ValidationError(format!("Stooq API connection failed: {}", e)))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(AppError::ValidationError(format!(
            "Stooq API returned error {}: {}",
            status, text
        )));
    }

    let text = resp
        .text()
        .await
        .map_err(|e| AppError::ValidationError(format!("Failed to read Stooq response: {}", e)))?;

    // CSV format:
    // Symbol,Date,Time,Open,High,Low,Close,Volume
    // VOO.US,YYYY-MM-DD,HH:MM:SS,....,Close,Volume
    let mut lines = text.lines();
    let _header = lines.next();
    let row = lines
        .next()
        .ok_or_else(|| AppError::ValidationError("No data found on Stooq".to_string()))?;

    let parts: Vec<&str> = row.split(',').collect();
    if parts.len() < 8 {
        return Err(AppError::ValidationError(format!(
            "Unexpected Stooq CSV row: {}",
            row
        )));
    }

    let close_str = parts[6];
    if close_str == "N/A" || close_str.is_empty() {
        return Err(AppError::ValidationError(format!(
            "No Stooq close price for {}",
            api_ticker
        )));
    }

    let close_f: f64 = close_str.parse().map_err(|_| {
        AppError::ValidationError(format!("Failed to parse Stooq close price: {}", close_str))
    })?;

    Decimal::from_f64(close_f)
        .ok_or_else(|| AppError::ValidationError("Failed to parse Stooq price".to_string()))
}

async fn fetch_price_yahoo(
    client: &reqwest::Client,
    ticker: &str,
) -> Result<(Decimal, String), AppError> {

    let url = format!(
        "https://query1.finance.yahoo.com/v8/finance/chart/{}?interval=1d&range=1m",
        ticker
    );

    let resp = client
        .get(&url)
        // Yahoo is picky; set a normal UA to avoid occasional 429/blocks.
        .header(
            reqwest::header::USER_AGENT,
            "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0 Safari/537.36",
        )
        .send()
        .await
        .map_err(|e| AppError::ValidationError(format!("Yahoo API connection failed: {}", e)))?;

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

// Internal structs for ExchangeRate-API (open.er-api.com) response parsing
// Response format: {"result":"success","base_code":"USD","rates":{...}}
#[derive(Deserialize, Debug)]
struct ErApiResponse {
    result: Option<String>,
    #[allow(dead_code)]
    base_code: Option<String>,
    rates: Option<std::collections::HashMap<String, f64>>,
    #[allow(dead_code)]
    error_type: Option<String>,
}

pub async fn fetch_exchange_rate(
    client: &reqwest::Client,
    from: &str,
    to: &str,
) -> Result<Decimal, AppError> {
    // Support common aliases used by some users/UIs.
    // Most FX providers use ISO 4217 code TWD (not NTD).
    let from = if from.eq_ignore_ascii_case("NTD") { "TWD" } else { from };
    let to = if to.eq_ignore_ascii_case("NTD") { "TWD" } else { to };

    if from == to {
        return Ok(Decimal::new(1, 0));
    }

    // ExchangeRate-API supports a wide set of currencies including TWD/IDR/SGD.
    let url = format!("https://open.er-api.com/v6/latest/{}", from);

    let resp = client.get(&url).send().await.map_err(|e| {
        AppError::ValidationError(format!("FX API connection failed: {}", e))
    })?;

    if !resp.status().is_success() {
        return Err(AppError::ValidationError(format!(
            "FX API returned error: {}",
            resp.status()
        )));
    }

    let data: ErApiResponse = resp.json().await.map_err(|e| {
        AppError::ValidationError(format!("Failed to parse FX API response: {}", e))
    })?;

    if data.result.as_deref() != Some("success") {
        return Err(AppError::ValidationError(format!(
            "FX API returned non-success result: {:?} ({:?})",
            data.result, data.error_type
        )));
    }

    let rates = data
        .rates
        .ok_or_else(|| AppError::ValidationError("FX API missing rates".to_string()))?;

    let rate = rates.get(to).ok_or_else(|| {
        AppError::ValidationError(format!("No rate found for {} -> {}", from, to))
    })?;

    Decimal::from_f64(*rate)
        .ok_or_else(|| AppError::ValidationError("Failed to parse exchange rate".to_string()))
}
