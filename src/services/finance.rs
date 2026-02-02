use rust_decimal::Decimal;
use uuid::Uuid;

use crate::error::AppError;
use crate::investments;
use crate::repository::{PortfolioRepository, SettingsRepository, TransactionRepository};
use crate::schemas::{CreatePortfolioItem, FinancialHealth, UpdateInvestment};

pub struct FinanceService {
    portfolio_repo: PortfolioRepository,
    transaction_repo: TransactionRepository,
    settings_repo: SettingsRepository,
    price_cache: moka::future::Cache<String, Decimal>,
    exchange_rate_cache: moka::future::Cache<String, Decimal>,
    http_client: reqwest::Client,
    itick_api_key: Option<String>,
}

impl FinanceService {
    pub fn new(
        portfolio_repo: PortfolioRepository,
        transaction_repo: TransactionRepository,
        settings_repo: SettingsRepository,
        price_cache: moka::future::Cache<String, Decimal>,
        exchange_rate_cache: moka::future::Cache<String, Decimal>,
        http_client: reqwest::Client,
        itick_api_key: Option<String>,
    ) -> Self {
        Self {
            portfolio_repo,
            transaction_repo,
            settings_repo,
            price_cache,
            exchange_rate_cache,
            http_client,
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
        let rate = investments::fetch_exchange_rate(&self.http_client, from, to).await?;
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
                match investments::fetch_coingecko_icon(&self.http_client, &api_ticker).await {
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

        let (price, currency) = investments::fetch_price_with_source(
            &self.http_client,
            ticker,
            &api_ticker,
            &source,
            self.itick_api_key.as_deref(),
        )
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
