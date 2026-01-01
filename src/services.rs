use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::auth::{Claims, get_keys, hash_password, verify_password};
use crate::error::AppError;
use crate::investments;
use crate::repository::{
    PortfolioRepository, SettingsRepository, TransactionRepository, UserRepository,
};
use crate::schemas::{
    AuthResponse, Category, CreatePortfolioItem, CreateTransaction, FinancialHealth, LoginRequest,
    RegisterRequest, Transaction, TransactionDetail, UpdateInvestment, UserProfile,
};

use jsonwebtoken::{Header, encode};

pub struct AuthService {
    user_repo: UserRepository,
    settings_repo: SettingsRepository,
}

impl AuthService {
    pub fn new(user_repo: UserRepository, settings_repo: SettingsRepository) -> Self {
        Self {
            user_repo,
            settings_repo,
        }
    }

    pub async fn register(&self, req: RegisterRequest) -> Result<AuthResponse, AppError> {
        if self
            .user_repo
            .check_exists(&req.email, &req.username)
            .await?
        {
            return Err(AppError::ValidationError(
                "User with this email or username already exists".to_string(),
            ));
        }

        if !self
            .settings_repo
            .validate_currency(&req.base_currency)
            .await?
        {
            return Err(AppError::ValidationError(format!(
                "Invalid currency code: {}",
                req.base_currency
            )));
        }

        let hashed = hash_password(&req.password)?;
        let user_id = self
            .user_repo
            .create(&req.username, &req.email, &hashed)
            .await?;

        self.settings_repo
            .set_base_currency(user_id, &req.base_currency)
            .await?;

        // Auto-login (generate token)
        let token = self.generate_token(user_id)?;

        Ok(AuthResponse {
            token,
            message: "Registration successful".to_string(),
        })
    }

    pub async fn login(&self, req: LoginRequest) -> Result<AuthResponse, AppError> {
        let user = self
            .user_repo
            .find_by_email(&req.email)
            .await?
            .ok_or(AppError::AuthError("Invalid credentials".to_string()))?;

        if !verify_password(&req.password, &user.password_hash)? {
            return Err(AppError::AuthError("Invalid credentials".to_string()));
        }

        let token = self.generate_token(user.id)?;

        Ok(AuthResponse {
            token,
            message: "Login successful".to_string(),
        })
    }

    fn generate_token(&self, user_id: Uuid) -> Result<String, AppError> {
        let claims = Claims {
            sub: user_id.to_string(),
            company: "Phoebudget".to_string(),
            exp: (Utc::now() + chrono::Duration::hours(24)).timestamp() as usize,
        };

        encode(&Header::default(), &claims, &get_keys().encoding)
            .map_err(|_| AppError::InternalServerError("Token creation failed".to_string()))
    }

    pub async fn get_profile(&self, user_id: Uuid) -> Result<UserProfile, AppError> {
        self.user_repo.get_profile(user_id).await
    }
}

pub struct TransactionService {
    transaction_repo: TransactionRepository,
    settings_repo: SettingsRepository,
    http_client: reqwest::Client,
}

impl TransactionService {
    pub fn new(
        transaction_repo: TransactionRepository,
        settings_repo: SettingsRepository,
        http_client: reqwest::Client,
    ) -> Self {
        Self {
            transaction_repo,
            settings_repo,
            http_client,
        }
    }

    pub async fn get_categories(&self) -> Result<Vec<Category>, AppError> {
        self.transaction_repo.get_all_categories().await
    }

    pub async fn create_transaction(
        &self,
        user_id: Uuid,
        req: CreateTransaction,
    ) -> Result<Uuid, AppError> {
        if req.amount <= Decimal::ZERO {
            return Err(AppError::ValidationError(
                "Amount must be positive".to_string(),
            ));
        }

        let base_currency = self.settings_repo.get_base_currency(user_id).await?;
        let (amount, original_currency, original_amount, exchange_rate) = if let Some(currency) =
            &req.currency_code
        {
            if currency != &base_currency {
                let rate =
                    investments::fetch_exchange_rate(&self.http_client, currency, &base_currency)
                        .await?;
                let converted_amount = req.amount * rate;
                (
                    converted_amount,
                    Some(currency.clone()),
                    Some(req.amount),
                    Some(rate),
                )
            } else {
                (req.amount, None, None, None)
            }
        } else {
            (req.amount, None, None, None)
        };

        let description = req.description.filter(|d| !d.trim().is_empty());

        self.transaction_repo
            .create(
                user_id,
                amount,
                description,
                req.category_id,
                req.occurred_at,
                original_currency,
                original_amount,
                exchange_rate,
            )
            .await
    }

    pub async fn get_transactions(
        &self,
        user_id: Uuid,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<Vec<Transaction>, AppError> {
        if end_date < start_date {
            return Err(AppError::ValidationError(
                "End date cannot be before start date".to_string(),
            ));
        }
        self.transaction_repo
            .find_by_user_and_date(user_id, start_date, end_date)
            .await
    }

    pub async fn get_spending_analysis(
        &self,
        user_id: Uuid,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<crate::schemas::SpendingAnalysisResponse, AppError> {
        let categories = self
            .transaction_repo
            .get_spending_analysis(user_id, start_date, end_date)
            .await?;

        let mut total_income = Decimal::ZERO;
        let mut total_spent = Decimal::ZERO;

        for cat in &categories {
            if cat.is_income {
                total_income += cat.total;
            } else {
                total_spent += cat.total;
            }
        }

        let net_income = total_income - total_spent;

        Ok(crate::schemas::SpendingAnalysisResponse {
            total_income,
            total_spent,
            net_income,
            categories,
        })
    }
    pub async fn update_transaction(
        &self,
        id: Uuid,
        user_id: Uuid,
        req: crate::schemas::UpdateTransaction,
    ) -> Result<(), AppError> {
        if let Some(amount) = req.amount {
            if amount <= Decimal::ZERO {
                return Err(AppError::ValidationError(
                    "Amount must be positive".to_string(),
                ));
            }
        }
        let description = req.description.filter(|d| !d.trim().is_empty());

        self.transaction_repo
            .update(
                id,
                user_id,
                req.amount,
                description,
                req.category_id,
                req.occurred_at,
                req.original_currency,
                req.original_amount,
                req.exchange_rate,
            )
            .await
    }

    pub async fn delete_transaction(&self, id: Uuid, user_id: Uuid) -> Result<(), AppError> {
        let deleted = self.transaction_repo.delete(id, user_id).await?;
        if deleted == 0 {
            return Err(AppError::NotFoundError("Transaction not found".to_string()));
        }
        Ok(())
    }
    pub async fn get_transaction(
        &self,
        user_id: Uuid,
        id: Uuid,
    ) -> Result<TransactionDetail, AppError> {
        self.transaction_repo.get_transaction(id, user_id).await
    }
}

pub struct FinanceService {
    portfolio_repo: PortfolioRepository,
    transaction_repo: TransactionRepository,
    settings_repo: SettingsRepository,
    price_cache: moka::future::Cache<String, Decimal>,
    exchange_rate_cache: moka::future::Cache<String, Decimal>,
    http_client: reqwest::Client,
}

impl FinanceService {
    pub fn new(
        portfolio_repo: PortfolioRepository,
        transaction_repo: TransactionRepository,
        settings_repo: SettingsRepository,
        price_cache: moka::future::Cache<String, Decimal>,
        exchange_rate_cache: moka::future::Cache<String, Decimal>,
        http_client: reqwest::Client,
    ) -> Self {
        Self {
            portfolio_repo,
            transaction_repo,
            settings_repo,
            price_cache,
            exchange_rate_cache,
            http_client,
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
        let invested_usd = self.portfolio_repo.get_total_invested(user_id).await?;

        let rate = if base_currency != "USD" {
            self.get_cached_exchange_rate("USD", &base_currency).await?
        } else {
            Decimal::new(1, 0)
        };

        let invested_converted = invested_usd * rate;
        let net_worth = cash + invested_converted;

        Ok(FinancialHealth {
            cash_balance: cash,
            investment_balance: invested_converted,
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
            let source = asset.source.unwrap_or("YAHOO".to_string());

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
            // If asset not found in DB, for now we default to YAHOO/Ticker
            // (e.g. legacy behavior or if someone manually inserted via sql)
            (ticker.to_string(), "YAHOO".to_string())
        };

        let (price, currency) =
            investments::fetch_price_with_source(&self.http_client, ticker, &api_ticker, &source)
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
