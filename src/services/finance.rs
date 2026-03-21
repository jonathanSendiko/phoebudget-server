use async_trait::async_trait;
use chrono::{DateTime, Datelike, Months, Utc};
use rust_decimal::Decimal;
use std::collections::HashMap;
use uuid::Uuid;

use crate::error::AppError;
use crate::investments;
use crate::repository::{PortfolioRepository, SettingsRepository, TransactionRepository};
use crate::schemas::{
    Asset, CreatePortfolioItem, FinancialHealth, MonthlyCashFlowRow, NetWorthHistoryPoint,
    NetWorthHistoryResponse, UpdateInvestment,
};

#[async_trait]
pub trait FinancePortfolioRepo: Send + Sync {
    async fn get_tickers(&self, user_id: Uuid) -> Result<Vec<String>, AppError>;
    async fn get_all_joined(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<crate::schemas::PortfolioJoinedRow>, AppError>;
    async fn get_asset(&self, ticker: &str) -> Result<Option<Asset>, AppError>;
    async fn update_asset_price(
        &self,
        ticker: &str,
        price: Decimal,
        currency: &str,
    ) -> Result<(), AppError>;
    async fn update_asset_icon(&self, ticker: &str, icon_url: &str) -> Result<(), AppError>;
    async fn add_item(&self, user_id: Uuid, item: CreatePortfolioItem) -> Result<(), AppError>;
    async fn delete(&self, user_id: Uuid, ticker: &str) -> Result<u64, AppError>;
    async fn update(
        &self,
        user_id: Uuid,
        ticker: &str,
        quantity: Option<Decimal>,
        avg_buy_price: Option<Decimal>,
    ) -> Result<(), AppError>;
}

#[async_trait]
pub trait FinanceTransactionRepo: Send + Sync {
    async fn get_net_cash(&self, user_id: Uuid) -> Result<Decimal, AppError>;
    async fn get_net_cash_before(
        &self,
        user_id: Uuid,
        start_date: DateTime<Utc>,
    ) -> Result<Decimal, AppError>;
    async fn get_monthly_cash_flow(
        &self,
        user_id: Uuid,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<Vec<MonthlyCashFlowRow>, AppError>;
}

#[async_trait]
pub trait FinanceSettingsRepo: Send + Sync {
    async fn get_base_currency(&self, user_id: Uuid) -> Result<String, AppError>;
    async fn validate_currency(&self, code: &str) -> Result<bool, AppError>;
    async fn set_base_currency(&self, user_id: Uuid, currency: &str) -> Result<(), AppError>;
}

#[async_trait]
pub trait PriceProvider: Send + Sync {
    async fn fetch_price(
        &self,
        ticker: &str,
        api_ticker: &str,
        source: &str,
        itick_api_key: Option<&str>,
    ) -> Result<(Decimal, String), AppError>;
    async fn fetch_icon(&self, api_ticker: &str) -> Result<Option<String>, AppError>;
}

#[async_trait]
pub trait ExchangeRateProvider: Send + Sync {
    async fn fetch_rate(&self, from: &str, to: &str) -> Result<Decimal, AppError>;
}

pub struct HttpPriceProvider {
    client: reqwest::Client,
}

impl HttpPriceProvider {
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl PriceProvider for HttpPriceProvider {
    async fn fetch_price(
        &self,
        ticker: &str,
        api_ticker: &str,
        source: &str,
        itick_api_key: Option<&str>,
    ) -> Result<(Decimal, String), AppError> {
        investments::fetch_price_with_source(
            &self.client,
            ticker,
            api_ticker,
            source,
            itick_api_key,
        )
        .await
    }

    async fn fetch_icon(&self, api_ticker: &str) -> Result<Option<String>, AppError> {
        investments::fetch_coingecko_icon(&self.client, api_ticker).await
    }
}

pub struct HttpExchangeRateProvider {
    client: reqwest::Client,
}

impl HttpExchangeRateProvider {
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl ExchangeRateProvider for HttpExchangeRateProvider {
    async fn fetch_rate(&self, from: &str, to: &str) -> Result<Decimal, AppError> {
        investments::fetch_exchange_rate(&self.client, from, to).await
    }
}

#[async_trait]
impl FinancePortfolioRepo for PortfolioRepository {
    async fn get_tickers(&self, user_id: Uuid) -> Result<Vec<String>, AppError> {
        self.get_tickers(user_id).await
    }

    async fn get_all_joined(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<crate::schemas::PortfolioJoinedRow>, AppError> {
        self.get_all_joined(user_id).await
    }

    async fn get_asset(&self, ticker: &str) -> Result<Option<Asset>, AppError> {
        self.get_asset(ticker).await
    }

    async fn update_asset_price(
        &self,
        ticker: &str,
        price: Decimal,
        currency: &str,
    ) -> Result<(), AppError> {
        self.update_asset_price(ticker, price, currency).await
    }

    async fn update_asset_icon(&self, ticker: &str, icon_url: &str) -> Result<(), AppError> {
        self.update_asset_icon(ticker, icon_url).await
    }

    async fn add_item(&self, user_id: Uuid, item: CreatePortfolioItem) -> Result<(), AppError> {
        self.add_item(user_id, item).await
    }

    async fn delete(&self, user_id: Uuid, ticker: &str) -> Result<u64, AppError> {
        self.delete(user_id, ticker).await
    }

    async fn update(
        &self,
        user_id: Uuid,
        ticker: &str,
        quantity: Option<Decimal>,
        avg_buy_price: Option<Decimal>,
    ) -> Result<(), AppError> {
        self.update(user_id, ticker, quantity, avg_buy_price).await
    }
}

#[async_trait]
impl FinanceTransactionRepo for TransactionRepository {
    async fn get_net_cash(&self, user_id: Uuid) -> Result<Decimal, AppError> {
        self.get_net_cash(user_id).await
    }

    async fn get_net_cash_before(
        &self,
        user_id: Uuid,
        start_date: DateTime<Utc>,
    ) -> Result<Decimal, AppError> {
        self.get_net_cash_before(user_id, start_date).await
    }

    async fn get_monthly_cash_flow(
        &self,
        user_id: Uuid,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<Vec<MonthlyCashFlowRow>, AppError> {
        self.get_monthly_cash_flow(user_id, start_date, end_date)
            .await
    }
}

#[async_trait]
impl FinanceSettingsRepo for SettingsRepository {
    async fn get_base_currency(&self, user_id: Uuid) -> Result<String, AppError> {
        self.get_base_currency(user_id).await
    }

    async fn validate_currency(&self, code: &str) -> Result<bool, AppError> {
        self.validate_currency(code).await
    }

    async fn set_base_currency(&self, user_id: Uuid, currency: &str) -> Result<(), AppError> {
        self.set_base_currency(user_id, currency).await
    }
}

pub type FinanceServiceImpl = FinanceService<
    PortfolioRepository,
    TransactionRepository,
    SettingsRepository,
    HttpPriceProvider,
    HttpExchangeRateProvider,
>;

pub struct FinanceService<PRepo, TRepo, SRepo, PriceP, RateP> {
    portfolio_repo: PRepo,
    transaction_repo: TRepo,
    settings_repo: SRepo,
    price_provider: PriceP,
    exchange_rate_provider: RateP,
    price_cache: moka::future::Cache<String, Decimal>,
    exchange_rate_cache: moka::future::Cache<String, Decimal>,
    itick_api_key: Option<String>,
}

impl<PRepo, TRepo, SRepo, PriceP, RateP> FinanceService<PRepo, TRepo, SRepo, PriceP, RateP>
where
    PRepo: FinancePortfolioRepo,
    TRepo: FinanceTransactionRepo,
    SRepo: FinanceSettingsRepo,
    PriceP: PriceProvider,
    RateP: ExchangeRateProvider,
{
    pub fn new(
        portfolio_repo: PRepo,
        transaction_repo: TRepo,
        settings_repo: SRepo,
        price_provider: PriceP,
        exchange_rate_provider: RateP,
        price_cache: moka::future::Cache<String, Decimal>,
        exchange_rate_cache: moka::future::Cache<String, Decimal>,
        itick_api_key: Option<String>,
    ) -> Self {
        Self {
            portfolio_repo,
            transaction_repo,
            settings_repo,
            price_provider,
            exchange_rate_provider,
            price_cache,
            exchange_rate_cache,
            itick_api_key,
        }
    }

    /// Cached exchange rate lookup with 60s TTL
    async fn get_cached_exchange_rate(&self, from: &str, to: &str) -> Result<Decimal, AppError> {
        if from == to {
            return Ok(Decimal::new(1, 0));
        }

        let cache_key = format!("{}_{}", from, to);
        if let Some(rate) = self.exchange_rate_cache.get(&cache_key).await {
            tracing::info!("Exchange rate cache HIT for {} -> {}", from, to);
            return Ok(rate);
        }

        tracing::info!("Exchange rate cache MISS for {} -> {}", from, to);
        let rate = self.exchange_rate_provider.fetch_rate(from, to).await?;
        self.exchange_rate_cache.insert(cache_key, rate).await;
        Ok(rate)
    }

    pub async fn get_financial_health(&self, user_id: Uuid) -> Result<FinancialHealth, AppError> {
        let base_currency = self.settings_repo.get_base_currency(user_id).await?;
        let cash = self.transaction_repo.get_net_cash(user_id).await?;

        // Use same logic as portfolio API for calculating investment value
        let items = self.portfolio_repo.get_all_joined(user_id).await?;

        // Calculate total investment value with proper currency conversion (matching portfolio logic)
        let mut investment_balance = Decimal::ZERO;
        for item in &items {
            let asset_currency = item.currency.clone().unwrap_or_else(|| "USD".to_string());
            let rate = if asset_currency != base_currency {
                self.get_cached_exchange_rate(&asset_currency, &base_currency)
                    .await?
            } else {
                Decimal::ONE
            };
            // current_value = quantity * current_price * exchange_rate
            let value_converted = item.quantity * item.current_price * rate;
            investment_balance += value_converted;
        }

        let net_worth = cash + investment_balance;

        Ok(FinancialHealth {
            cash_balance: cash,
            investment_balance,
            total_net_worth: net_worth,
        })
    }

    pub async fn get_net_worth_history(
        &self,
        user_id: Uuid,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<NetWorthHistoryResponse, AppError> {
        if end_date < start_date {
            return Err(AppError::ValidationError(
                "End date cannot be before start date".to_string(),
            ));
        }

        let opening_balance = self
            .transaction_repo
            .get_net_cash_before(user_id, start_date)
            .await?;
        let rows = self
            .transaction_repo
            .get_monthly_cash_flow(user_id, start_date, end_date)
            .await?;

        let mut by_month: HashMap<String, (Decimal, Decimal)> = HashMap::new();
        for row in rows {
            let key = format!("{:04}-{:02}", row.month.year(), row.month.month());
            by_month.insert(key, (row.total_income, row.total_spent));
        }

        let mut points = Vec::new();
        let mut running = opening_balance;
        let mut current = month_start(start_date)?;
        let end_month = month_start(end_date)?;

        loop {
            let key = format!("{:04}-{:02}", current.year(), current.month());
            let (total_income, total_spent) = by_month
                .get(&key)
                .cloned()
                .unwrap_or((Decimal::ZERO, Decimal::ZERO));
            let net_change = total_income - total_spent;
            running += net_change;

            points.push(NetWorthHistoryPoint {
                month: key,
                total_income,
                total_spent,
                net_change,
                net_worth_end: running,
            });

            if current.year() == end_month.year() && current.month() == end_month.month() {
                break;
            }
            current = next_month_start(current)?;
        }

        Ok(NetWorthHistoryResponse {
            start_date,
            end_date,
            opening_balance,
            points,
        })
    }

    pub async fn refresh_portfolio(&self, user_id: Uuid) -> Result<u64, AppError> {
        let tickers = self.portfolio_repo.get_tickers(user_id).await?;
        let count = tickers.len() as u64;

        let fetch_futures: Vec<_> = tickers
            .iter()
            .map(|ticker| async move {
                if let Err(e) = self.ensure_price_fresh(ticker).await {
                    tracing::error!("Failed to refresh price for {}: {:?}", ticker, e);
                }
            })
            .collect();
        futures::future::join_all(fetch_futures).await;

        Ok(count)
    }

    pub async fn add_investment(
        &self,
        user_id: Uuid,
        item: CreatePortfolioItem,
    ) -> Result<(), AppError> {
        // Validate ticker and ensure price is in assets table
        self.ensure_price_fresh(&item.ticker).await?;

        // Ensure asset exists in DB (already done by seed data or we error out in add_item)

        match self.portfolio_repo.add_item(user_id, item.clone()).await {
            Ok(_) => Ok(()),
            Err(e) => {
                // AppError is likely DatabaseError
                // Simple string matching for now
                let msg = format!("{:?}", e);
                if msg.contains("duplicate key value") {
                    Err(AppError::ValidationError(format!(
                        "{} is already in your portfolio",
                        &item.ticker,
                    )))
                } else {
                    Err(e)
                }
            }
        }
    }

    // Helper to get price with cache
    async fn ensure_price_fresh(&self, ticker: &str) -> Result<Decimal, AppError> {
        if let Some(price) = self.price_cache.get(ticker).await {
            tracing::info!("Cache HIT for {}", ticker);
            return Ok(price);
        }
        tracing::info!("Cache MISS for {}", ticker);

        // Fetch asset from DB to know Source and API Ticker
        let asset_opt = self.portfolio_repo.get_asset(ticker).await?;
        let (api_ticker, source) = if let Some(asset) = asset_opt {
            let api_ticker = asset.api_ticker.unwrap_or(ticker.to_string());
            let source = asset.source.unwrap_or("ITICK".to_string());

            // NEW: Check for missing icon and populate it lazily
            if asset.icon_url.is_none() && source == "COINGECKO" {
                tracing::info!("Missing icon for {}, fetching from CoinGecko...", ticker);
                // We don't want to fail the whole request if icon fetch fails
                match self.price_provider.fetch_icon(&api_ticker).await {
                    Ok(Some(url)) => {
                        if let Err(e) = self.portfolio_repo.update_asset_icon(ticker, &url).await {
                            tracing::error!("Failed to save icon for {}: {:?}", ticker, e);
                        } else {
                            tracing::info!("Updated icon for {}", ticker);
                        }
                    }
                    Ok(None) => tracing::warn!("No icon found for {}", ticker),
                    Err(e) => tracing::error!("Failed to fetch icon for {}: {:?}", ticker, e),
                }
            }

            (api_ticker, source)
        } else {
            // If asset not found in DB, for now we default to ITICK/Ticker
            // (e.g. legacy behavior or if someone manually inserted via sql)
            (ticker.to_string(), "ITICK".to_string())
        };

        let (price, currency) = self
            .price_provider
            .fetch_price(ticker, &api_ticker, &source, self.itick_api_key.as_deref())
            .await?;

        // Update DB
        self.portfolio_repo
            .update_asset_price(ticker, price, &currency)
            .await?;

        self.price_cache.insert(ticker.to_string(), price).await;

        Ok(price)
    }

    pub async fn get_portfolio_list(
        &self,
        user_id: Uuid,
    ) -> Result<crate::schemas::PortfolioResponse, AppError> {
        // Fetch prices in parallel for all tickers
        let tickers = self.portfolio_repo.get_tickers(user_id).await?;
        let fetch_futures: Vec<_> = tickers
            .iter()
            .map(|ticker| async move {
                if let Err(e) = self.ensure_price_fresh(ticker).await {
                    tracing::error!("Failed to refresh price for {}: {:?}", ticker, e);
                }
            })
            .collect();
        futures::future::join_all(fetch_futures).await;

        // Get data
        let base_currency = self.settings_repo.get_base_currency(user_id).await?;
        let items = self.portfolio_repo.get_all_joined(user_id).await?;

        // Pre-fetch all unique exchange rates (async part)
        let unique_currencies: std::collections::HashSet<String> = items
            .iter()
            .filter_map(|item| item.currency.clone())
            .filter(|c| c != &base_currency)
            .collect();

        let mut exchange_rates = std::collections::HashMap::new();
        for currency in unique_currencies {
            let rate = self
                .get_cached_exchange_rate(&currency, &base_currency)
                .await?;
            exchange_rates.insert(currency, rate);
        }

        // Use tested pure function to build the full response
        Ok(crate::portfolio::build_portfolio_response(
            items,
            &exchange_rates,
            &base_currency,
        ))
    }

    pub async fn update_base_currency(
        &self,
        user_id: Uuid,
        currency: String,
    ) -> Result<(), AppError> {
        if !self.settings_repo.validate_currency(&currency).await? {
            return Err(AppError::ValidationError(format!(
                "Invalid currency code: {}",
                currency
            )));
        }
        self.settings_repo
            .set_base_currency(user_id, &currency)
            .await
    }

    pub async fn remove_investment(&self, user_id: Uuid, ticker: String) -> Result<(), AppError> {
        let deleted = self.portfolio_repo.delete(user_id, &ticker).await?;
        if deleted == 0 {
            return Err(AppError::NotFoundError(format!(
                "Investment {} not found",
                ticker
            )));
        }
        Ok(())
    }

    pub async fn update_investment(
        &self,
        user_id: Uuid,
        ticker: String,
        payload: UpdateInvestment,
    ) -> Result<(), AppError> {
        self.portfolio_repo
            .update(user_id, &ticker, payload.quantity, payload.avg_buy_price)
            .await
    }
}

fn month_start(date: DateTime<Utc>) -> Result<DateTime<Utc>, AppError> {
    let naive = date
        .date_naive()
        .with_day(1)
        .and_then(|d| d.and_hms_nano_opt(0, 0, 0, 0))
        .ok_or_else(|| {
            AppError::InternalServerError("Failed to compute month start".to_string())
        })?;
    Ok(DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc))
}

fn next_month_start(date: DateTime<Utc>) -> Result<DateTime<Utc>, AppError> {
    let next_date = date
        .date_naive()
        .checked_add_months(Months::new(1))
        .ok_or_else(|| AppError::InternalServerError("Failed to compute next month".to_string()))?;
    let naive = next_date
        .with_day(1)
        .and_then(|d| d.and_hms_nano_opt(0, 0, 0, 0))
        .ok_or_else(|| AppError::InternalServerError("Failed to compute next month".to_string()))?;
    Ok(DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc))
}

#[cfg(test)]
mod tests {
    use super::{
        ExchangeRateProvider, FinancePortfolioRepo, FinanceService, FinanceSettingsRepo,
        FinanceTransactionRepo, PriceProvider,
    };
    use crate::error::AppError;
    use crate::schemas::{Asset, CreatePortfolioItem, MonthlyCashFlowRow, PortfolioJoinedRow};
    use async_trait::async_trait;
    use chrono::{DateTime, TimeZone, Utc};
    use moka::future::Cache;
    use rust_decimal::Decimal;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use uuid::Uuid;

    #[derive(Clone, Default)]
    struct MockPortfolioRepo {
        state: Arc<Mutex<MockPortfolioState>>,
    }

    struct MockPortfolioState {
        tickers: Vec<String>,
        items: Vec<PortfolioJoinedRow>,
        assets: HashMap<String, Asset>,
        asset_calls: usize,
        updated_prices: Vec<(String, Decimal, String)>,
        updated_icons: Vec<(String, String)>,
        delete_result: u64,
        add_item_calls: usize,
        update_calls: usize,
    }

    impl Default for MockPortfolioState {
        fn default() -> Self {
            Self {
                tickers: Vec::new(),
                items: Vec::new(),
                assets: HashMap::new(),
                asset_calls: 0,
                updated_prices: Vec::new(),
                updated_icons: Vec::new(),
                delete_result: 1,
                add_item_calls: 0,
                update_calls: 0,
            }
        }
    }

    #[async_trait]
    impl FinancePortfolioRepo for MockPortfolioRepo {
        async fn get_tickers(&self, _user_id: Uuid) -> Result<Vec<String>, AppError> {
            Ok(self.state.lock().unwrap().tickers.clone())
        }

        async fn get_all_joined(
            &self,
            _user_id: Uuid,
        ) -> Result<Vec<PortfolioJoinedRow>, AppError> {
            let state = self.state.lock().unwrap();
            Ok(state.items.iter().map(clone_joined_row_ref).collect())
        }

        async fn get_asset(&self, ticker: &str) -> Result<Option<Asset>, AppError> {
            let mut state = self.state.lock().unwrap();
            state.asset_calls += 1;
            Ok(state.assets.get(ticker).cloned())
        }

        async fn update_asset_price(
            &self,
            ticker: &str,
            price: Decimal,
            currency: &str,
        ) -> Result<(), AppError> {
            let mut state = self.state.lock().unwrap();
            state
                .updated_prices
                .push((ticker.to_string(), price, currency.to_string()));
            Ok(())
        }

        async fn update_asset_icon(&self, ticker: &str, icon_url: &str) -> Result<(), AppError> {
            let mut state = self.state.lock().unwrap();
            state
                .updated_icons
                .push((ticker.to_string(), icon_url.to_string()));
            Ok(())
        }

        async fn add_item(
            &self,
            _user_id: Uuid,
            _item: CreatePortfolioItem,
        ) -> Result<(), AppError> {
            let mut state = self.state.lock().unwrap();
            state.add_item_calls += 1;
            Ok(())
        }

        async fn delete(&self, _user_id: Uuid, _ticker: &str) -> Result<u64, AppError> {
            Ok(self.state.lock().unwrap().delete_result)
        }

        async fn update(
            &self,
            _user_id: Uuid,
            _ticker: &str,
            _quantity: Option<Decimal>,
            _avg_buy_price: Option<Decimal>,
        ) -> Result<(), AppError> {
            let mut state = self.state.lock().unwrap();
            state.update_calls += 1;
            Ok(())
        }
    }

    #[derive(Clone)]
    struct MockTransactionRepo {
        net_cash: Decimal,
        net_cash_before: Decimal,
        monthly_cash_flow: Vec<MonthlyCashFlowRow>,
    }

    #[async_trait]
    impl FinanceTransactionRepo for MockTransactionRepo {
        async fn get_net_cash(&self, _user_id: Uuid) -> Result<Decimal, AppError> {
            Ok(self.net_cash)
        }

        async fn get_net_cash_before(
            &self,
            _user_id: Uuid,
            _start_date: DateTime<Utc>,
        ) -> Result<Decimal, AppError> {
            Ok(self.net_cash_before)
        }

        async fn get_monthly_cash_flow(
            &self,
            _user_id: Uuid,
            _start_date: DateTime<Utc>,
            _end_date: DateTime<Utc>,
        ) -> Result<Vec<MonthlyCashFlowRow>, AppError> {
            Ok(self.monthly_cash_flow.clone())
        }
    }

    #[derive(Clone)]
    struct MockSettingsRepo {
        base_currency: String,
        validate_ok: bool,
        set_calls: Arc<Mutex<Vec<String>>>,
    }

    #[async_trait]
    impl FinanceSettingsRepo for MockSettingsRepo {
        async fn get_base_currency(&self, _user_id: Uuid) -> Result<String, AppError> {
            Ok(self.base_currency.clone())
        }

        async fn validate_currency(&self, _code: &str) -> Result<bool, AppError> {
            Ok(self.validate_ok)
        }

        async fn set_base_currency(&self, _user_id: Uuid, currency: &str) -> Result<(), AppError> {
            self.set_calls.lock().unwrap().push(currency.to_string());
            Ok(())
        }
    }

    #[derive(Clone, Default)]
    struct MockPriceProvider {
        calls: Arc<Mutex<Vec<(String, String, String, Option<String>)>>>,
        icon_calls: Arc<Mutex<Vec<String>>>,
        price: Decimal,
        currency: String,
    }

    #[async_trait]
    impl PriceProvider for MockPriceProvider {
        async fn fetch_price(
            &self,
            ticker: &str,
            api_ticker: &str,
            source: &str,
            itick_api_key: Option<&str>,
        ) -> Result<(Decimal, String), AppError> {
            self.calls.lock().unwrap().push((
                ticker.to_string(),
                api_ticker.to_string(),
                source.to_string(),
                itick_api_key.map(|s| s.to_string()),
            ));
            Ok((self.price, self.currency.clone()))
        }

        async fn fetch_icon(&self, api_ticker: &str) -> Result<Option<String>, AppError> {
            self.icon_calls.lock().unwrap().push(api_ticker.to_string());
            Ok(Some(format!("https://icons.example/{}.png", api_ticker)))
        }
    }

    #[derive(Clone, Default)]
    struct MockExchangeRateProvider {
        calls: Arc<Mutex<Vec<(String, String)>>>,
        rate: Decimal,
    }

    #[async_trait]
    impl ExchangeRateProvider for MockExchangeRateProvider {
        async fn fetch_rate(&self, from: &str, to: &str) -> Result<Decimal, AppError> {
            self.calls
                .lock()
                .unwrap()
                .push((from.to_string(), to.to_string()));
            Ok(self.rate)
        }
    }

    fn make_finance_service(
        portfolio_repo: MockPortfolioRepo,
        transaction_repo: MockTransactionRepo,
        settings_repo: MockSettingsRepo,
        price_provider: MockPriceProvider,
        exchange_rate_provider: MockExchangeRateProvider,
        price_cache: Cache<String, Decimal>,
        exchange_cache: Cache<String, Decimal>,
    ) -> FinanceService<
        MockPortfolioRepo,
        MockTransactionRepo,
        MockSettingsRepo,
        MockPriceProvider,
        MockExchangeRateProvider,
    > {
        FinanceService::new(
            portfolio_repo,
            transaction_repo,
            settings_repo,
            price_provider,
            exchange_rate_provider,
            price_cache,
            exchange_cache,
            None,
        )
    }

    fn make_joined_row(
        ticker: &str,
        quantity: Decimal,
        current_price: Decimal,
        currency: Option<&str>,
    ) -> PortfolioJoinedRow {
        PortfolioJoinedRow {
            ticker: ticker.to_string(),
            name: format!("{} Inc", ticker),
            quantity,
            avg_buy_price: Decimal::ZERO,
            current_price,
            source: Some("ITICK".to_string()),
            api_ticker: None,
            currency: currency.map(|c| c.to_string()),
            icon_url: None,
        }
    }

    fn make_cash_flow_row(
        year: i32,
        month: u32,
        total_income: Decimal,
        total_spent: Decimal,
    ) -> MonthlyCashFlowRow {
        let month_start = Utc
            .with_ymd_and_hms(year, month, 1, 0, 0, 0)
            .single()
            .expect("valid month");
        MonthlyCashFlowRow {
            month: month_start,
            total_income,
            total_spent,
        }
    }

    fn clone_joined_row_ref(row: &PortfolioJoinedRow) -> PortfolioJoinedRow {
        PortfolioJoinedRow {
            ticker: row.ticker.clone(),
            name: row.name.clone(),
            quantity: row.quantity,
            avg_buy_price: row.avg_buy_price,
            current_price: row.current_price,
            source: row.source.clone(),
            api_ticker: row.api_ticker.clone(),
            currency: row.currency.clone(),
            icon_url: row.icon_url.clone(),
        }
    }

    #[tokio::test]
    async fn get_financial_health_applies_exchange_rate() {
        let mut state = MockPortfolioState::default();
        state.items = vec![
            make_joined_row("AAA", Decimal::ONE, Decimal::new(10, 0), Some("USD")),
            make_joined_row("BBB", Decimal::new(2, 0), Decimal::new(5, 0), Some("EUR")),
        ];
        let portfolio_repo = MockPortfolioRepo {
            state: Arc::new(Mutex::new(state)),
        };
        let transaction_repo = MockTransactionRepo {
            net_cash: Decimal::new(100, 0),
            net_cash_before: Decimal::ZERO,
            monthly_cash_flow: Vec::new(),
        };
        let settings_repo = MockSettingsRepo {
            base_currency: "USD".to_string(),
            validate_ok: true,
            set_calls: Arc::new(Mutex::new(Vec::new())),
        };
        let price_provider = MockPriceProvider::default();
        let exchange_rate_provider = MockExchangeRateProvider {
            rate: Decimal::new(2, 0),
            ..Default::default()
        };
        let price_cache = Cache::new(100);
        let exchange_cache = Cache::new(100);
        let service = make_finance_service(
            portfolio_repo,
            transaction_repo,
            settings_repo,
            price_provider,
            exchange_rate_provider.clone(),
            price_cache,
            exchange_cache,
        );

        let health = service.get_financial_health(Uuid::new_v4()).await.unwrap();
        assert_eq!(health.cash_balance, Decimal::new(100, 0));
        assert_eq!(health.investment_balance, Decimal::new(30, 0));
        assert_eq!(health.total_net_worth, Decimal::new(130, 0));

        let calls = exchange_rate_provider.calls.lock().unwrap();
        assert_eq!(calls.as_slice(), &[("EUR".to_string(), "USD".to_string())]);
    }

    #[tokio::test]
    async fn get_financial_health_uses_cached_exchange_rate() {
        let mut state = MockPortfolioState::default();
        state.items = vec![make_joined_row(
            "BBB",
            Decimal::new(2, 0),
            Decimal::new(5, 0),
            Some("EUR"),
        )];
        let portfolio_repo = MockPortfolioRepo {
            state: Arc::new(Mutex::new(state)),
        };
        let transaction_repo = MockTransactionRepo {
            net_cash: Decimal::ZERO,
            net_cash_before: Decimal::ZERO,
            monthly_cash_flow: Vec::new(),
        };
        let settings_repo = MockSettingsRepo {
            base_currency: "USD".to_string(),
            validate_ok: true,
            set_calls: Arc::new(Mutex::new(Vec::new())),
        };
        let price_provider = MockPriceProvider::default();
        let exchange_rate_provider = MockExchangeRateProvider {
            rate: Decimal::new(9, 0),
            ..Default::default()
        };
        let price_cache = Cache::new(100);
        let exchange_cache = Cache::new(100);
        exchange_cache
            .insert("EUR_USD".to_string(), Decimal::new(3, 0))
            .await;
        let service = make_finance_service(
            portfolio_repo,
            transaction_repo,
            settings_repo,
            price_provider,
            exchange_rate_provider.clone(),
            price_cache,
            exchange_cache,
        );

        let health = service.get_financial_health(Uuid::new_v4()).await.unwrap();
        assert_eq!(health.investment_balance, Decimal::new(30, 0));

        let calls = exchange_rate_provider.calls.lock().unwrap();
        assert!(calls.is_empty());
    }

    #[tokio::test]
    async fn update_base_currency_rejects_invalid_currency() {
        let portfolio_repo = MockPortfolioRepo::default();
        let transaction_repo = MockTransactionRepo {
            net_cash: Decimal::ZERO,
            net_cash_before: Decimal::ZERO,
            monthly_cash_flow: Vec::new(),
        };
        let settings_repo = MockSettingsRepo {
            base_currency: "USD".to_string(),
            validate_ok: false,
            set_calls: Arc::new(Mutex::new(Vec::new())),
        };
        let price_provider = MockPriceProvider::default();
        let exchange_rate_provider = MockExchangeRateProvider::default();
        let service = make_finance_service(
            portfolio_repo,
            transaction_repo,
            settings_repo,
            price_provider,
            exchange_rate_provider,
            Cache::new(100),
            Cache::new(100),
        );

        let err = service
            .update_base_currency(Uuid::new_v4(), "BAD".to_string())
            .await
            .unwrap_err();
        assert!(
            matches!(err, AppError::ValidationError(msg) if msg == "Invalid currency code: BAD")
        );
    }

    #[tokio::test]
    async fn remove_investment_returns_not_found() {
        let mut state = MockPortfolioState::default();
        state.delete_result = 0;
        let portfolio_repo = MockPortfolioRepo {
            state: Arc::new(Mutex::new(state)),
        };
        let transaction_repo = MockTransactionRepo {
            net_cash: Decimal::ZERO,
            net_cash_before: Decimal::ZERO,
            monthly_cash_flow: Vec::new(),
        };
        let settings_repo = MockSettingsRepo {
            base_currency: "USD".to_string(),
            validate_ok: true,
            set_calls: Arc::new(Mutex::new(Vec::new())),
        };
        let price_provider = MockPriceProvider::default();
        let exchange_rate_provider = MockExchangeRateProvider::default();
        let service = make_finance_service(
            portfolio_repo,
            transaction_repo,
            settings_repo,
            price_provider,
            exchange_rate_provider,
            Cache::new(100),
            Cache::new(100),
        );

        let err = service
            .remove_investment(Uuid::new_v4(), "XYZ".to_string())
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::NotFoundError(msg) if msg == "Investment XYZ not found"));
    }

    #[tokio::test]
    async fn get_portfolio_list_skips_price_fetch_when_cached() {
        let mut state = MockPortfolioState::default();
        state.tickers = vec!["AAA".to_string()];
        state.items = vec![make_joined_row(
            "AAA",
            Decimal::new(1, 0),
            Decimal::new(10, 0),
            Some("USD"),
        )];
        let portfolio_repo = MockPortfolioRepo {
            state: Arc::new(Mutex::new(state)),
        };
        let transaction_repo = MockTransactionRepo {
            net_cash: Decimal::ZERO,
            net_cash_before: Decimal::ZERO,
            monthly_cash_flow: Vec::new(),
        };
        let settings_repo = MockSettingsRepo {
            base_currency: "USD".to_string(),
            validate_ok: true,
            set_calls: Arc::new(Mutex::new(Vec::new())),
        };
        let price_provider = MockPriceProvider {
            price: Decimal::new(10, 0),
            currency: "USD".to_string(),
            ..Default::default()
        };
        let exchange_rate_provider = MockExchangeRateProvider::default();
        let price_cache = Cache::new(100);
        price_cache
            .insert("AAA".to_string(), Decimal::new(10, 0))
            .await;
        let exchange_cache = Cache::new(100);
        let service = make_finance_service(
            portfolio_repo.clone(),
            transaction_repo,
            settings_repo,
            price_provider.clone(),
            exchange_rate_provider,
            price_cache,
            exchange_cache,
        );

        let _ = service.get_portfolio_list(Uuid::new_v4()).await.unwrap();

        let calls = price_provider.calls.lock().unwrap();
        assert!(calls.is_empty());

        let state = portfolio_repo.state.lock().unwrap();
        assert_eq!(state.asset_calls, 0);
        assert!(state.updated_prices.is_empty());
    }

    #[tokio::test]
    async fn get_net_worth_history_builds_monthly_series() {
        let portfolio_repo = MockPortfolioRepo::default();
        let transaction_repo = MockTransactionRepo {
            net_cash: Decimal::ZERO,
            net_cash_before: Decimal::new(100, 0),
            monthly_cash_flow: vec![
                make_cash_flow_row(2025, 1, Decimal::new(50, 0), Decimal::new(20, 0)),
                make_cash_flow_row(2025, 2, Decimal::new(10, 0), Decimal::new(40, 0)),
            ],
        };
        let settings_repo = MockSettingsRepo {
            base_currency: "USD".to_string(),
            validate_ok: true,
            set_calls: Arc::new(Mutex::new(Vec::new())),
        };
        let service = make_finance_service(
            portfolio_repo,
            transaction_repo,
            settings_repo,
            MockPriceProvider::default(),
            MockExchangeRateProvider::default(),
            Cache::new(100),
            Cache::new(100),
        );

        let start_date = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).single().unwrap();
        let end_date = Utc
            .with_ymd_and_hms(2025, 2, 28, 23, 59, 59)
            .single()
            .unwrap();

        let history = service
            .get_net_worth_history(Uuid::new_v4(), start_date, end_date)
            .await
            .unwrap();

        assert_eq!(history.opening_balance, Decimal::new(100, 0));
        assert_eq!(history.points.len(), 2);
        assert_eq!(history.points[0].month, "2025-01");
        assert_eq!(history.points[0].net_worth_end, Decimal::new(130, 0));
        assert_eq!(history.points[1].month, "2025-02");
        assert_eq!(history.points[1].net_worth_end, Decimal::new(100, 0));
    }

    #[tokio::test]
    async fn get_net_worth_history_fills_missing_months() {
        let portfolio_repo = MockPortfolioRepo::default();
        let transaction_repo = MockTransactionRepo {
            net_cash: Decimal::ZERO,
            net_cash_before: Decimal::ZERO,
            monthly_cash_flow: vec![make_cash_flow_row(
                2025,
                2,
                Decimal::new(20, 0),
                Decimal::new(5, 0),
            )],
        };
        let settings_repo = MockSettingsRepo {
            base_currency: "USD".to_string(),
            validate_ok: true,
            set_calls: Arc::new(Mutex::new(Vec::new())),
        };
        let service = make_finance_service(
            portfolio_repo,
            transaction_repo,
            settings_repo,
            MockPriceProvider::default(),
            MockExchangeRateProvider::default(),
            Cache::new(100),
            Cache::new(100),
        );

        let start_date = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).single().unwrap();
        let end_date = Utc
            .with_ymd_and_hms(2025, 3, 31, 23, 59, 59)
            .single()
            .unwrap();

        let history = service
            .get_net_worth_history(Uuid::new_v4(), start_date, end_date)
            .await
            .unwrap();

        assert_eq!(history.points.len(), 3);
        assert_eq!(history.points[0].month, "2025-01");
        assert_eq!(history.points[0].net_change, Decimal::ZERO);
        assert_eq!(history.points[1].month, "2025-02");
        assert_eq!(history.points[1].net_change, Decimal::new(15, 0));
        assert_eq!(history.points[2].month, "2025-03");
        assert_eq!(history.points[2].net_change, Decimal::ZERO);
    }

    #[tokio::test]
    async fn get_net_worth_history_rejects_invalid_range() {
        let portfolio_repo = MockPortfolioRepo::default();
        let transaction_repo = MockTransactionRepo {
            net_cash: Decimal::ZERO,
            net_cash_before: Decimal::ZERO,
            monthly_cash_flow: Vec::new(),
        };
        let settings_repo = MockSettingsRepo {
            base_currency: "USD".to_string(),
            validate_ok: true,
            set_calls: Arc::new(Mutex::new(Vec::new())),
        };
        let service = make_finance_service(
            portfolio_repo,
            transaction_repo,
            settings_repo,
            MockPriceProvider::default(),
            MockExchangeRateProvider::default(),
            Cache::new(100),
            Cache::new(100),
        );

        let start_date = Utc.with_ymd_and_hms(2025, 3, 1, 0, 0, 0).single().unwrap();
        let end_date = Utc.with_ymd_and_hms(2025, 2, 1, 0, 0, 0).single().unwrap();

        let err = service
            .get_net_worth_history(Uuid::new_v4(), start_date, end_date)
            .await
            .unwrap_err();

        assert!(
            matches!(err, AppError::ValidationError(msg) if msg == "End date cannot be before start date")
        );
    }
}
