use chrono::{DateTime, Datelike, NaiveDate, Utc};
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::auth::{Claims, get_keys, hash_password, verify_password};
use crate::error::AppError;
use crate::investments;
use crate::repository::{
    GoalEntryRepository, GoalRepository, PocketRepository, PortfolioRepository, SettingsRepository,
    SubscriptionRepository, TransactionRepository, UserRepository, UserSubscriptionRepository,
};
use crate::schemas::{
    AuthResponse, Category, CreatePocket, CreatePortfolioItem, CreateTransaction, FinancialHealth,
    LoginRequest, Pocket, RegisterRequest, SubscriptionLimits, SubscriptionResponse,
    TransactionDetail, UpdateInvestment, UpdatePocket, UserProfile,
};

use jsonwebtoken::{Header, encode};

use sha2::{Digest, Sha256};

pub struct AuthService {
    user_repo: UserRepository,
    settings_repo: SettingsRepository,
    pocket_repo: PocketRepository,
    refresh_token_repo: crate::repository::RefreshTokenRepository,
    subscription_repo: SubscriptionRepository,
}

impl AuthService {
    pub fn new(
        user_repo: UserRepository,
        settings_repo: SettingsRepository,
        pocket_repo: PocketRepository,
        refresh_token_repo: crate::repository::RefreshTokenRepository,
        subscription_repo: SubscriptionRepository,
    ) -> Self {
        Self {
            user_repo,
            settings_repo,
            pocket_repo,
            refresh_token_repo,
            subscription_repo,
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

        // Create default pocket for the new user
        self.pocket_repo.create_default_for_user(user_id).await?;

        // Create default free subscription for the new user
        self.subscription_repo.create_default(user_id).await?;

        // Auto-login (generate token)
        let (token, refresh_token) = self.generate_tokens(user_id).await?;

        Ok(AuthResponse {
            token,
            refresh_token,
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

        let (token, refresh_token) = self.generate_tokens(user.id).await?;

        Ok(AuthResponse {
            token,
            refresh_token,
            message: "Login successful".to_string(),
        })
    }

    pub async fn refresh_access(&self, refresh_token: &str) -> Result<AuthResponse, AppError> {
        // 1. Hash the incoming token
        let mut hasher = Sha256::new();
        hasher.update(refresh_token.as_bytes());
        let hash = hex::encode(hasher.finalize());

        // 2. Find in DB
        let token_row = self
            .refresh_token_repo
            .find_by_hash_and_user(&hash)
            .await?
            .ok_or(AppError::AuthError("Invalid refresh token".to_string()))?;

        // 3. Security checks
        if token_row.is_revoked.unwrap_or(false) {
            // Already revoked explicitly
            return Err(AppError::AuthError("Token revoked".to_string()));
        }

        if let Some(_replacement) = token_row.replaced_by {
            // REUSE DETECTED!
            // This token was already rotated. Someone is trying to use an old token.
            // Revoke EVERYTHING for this user.
            tracing::warn!(
                "Refresh token reuse detected for user {}. Revoking all sessions.",
                token_row.user_id
            );
            self.refresh_token_repo
                .revoke_all_for_user(token_row.user_id)
                .await?;
            return Err(AppError::AuthError(
                "Security alert: Token reuse detected".to_string(),
            ));
        }

        if token_row.expires_at < Utc::now() {
            return Err(AppError::AuthError("Token expired".to_string()));
        }

        // 4. Rotate: Generate new pair, mark old as replaced
        let (new_access_token, new_refresh_token) = self.generate_tokens(token_row.user_id).await?;

        // Calculate hash of new token to link
        let mut new_hasher = Sha256::new();
        new_hasher.update(new_refresh_token.as_bytes());
        let new_hash = hex::encode(new_hasher.finalize());

        self.refresh_token_repo
            .rotate(token_row.id, &new_hash)
            .await?;

        Ok(AuthResponse {
            token: new_access_token,
            refresh_token: new_refresh_token,
            message: "Token refreshed".to_string(),
        })
    }

    async fn generate_tokens(&self, user_id: Uuid) -> Result<(String, String), AppError> {
        // JWT
        let access_token = self.generate_jwt(user_id)?;

        // Refresh Token (64 char hex string from 2 UUIDs)
        let refresh_token = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());

        // Hash it
        let mut hasher = Sha256::new();
        hasher.update(refresh_token.as_bytes());
        let hash = hex::encode(hasher.finalize());

        // Save to DB (expires in 7 days)
        let expires_at = Utc::now() + chrono::Duration::days(7);
        self.refresh_token_repo
            .create(user_id, &hash, expires_at)
            .await?;

        Ok((access_token, refresh_token))
    }

    fn generate_jwt(&self, user_id: Uuid) -> Result<String, AppError> {
        let claims = Claims {
            sub: user_id.to_string(),
            company: "Phoebudget".to_string(),
            exp: (Utc::now() + chrono::Duration::hours(1)).timestamp() as usize, // Reduced to 1 hour
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
    pocket_repo: PocketRepository,
    settings_repo: SettingsRepository,
    http_client: reqwest::Client,
}

impl TransactionService {
    pub fn new(
        transaction_repo: TransactionRepository,
        pocket_repo: PocketRepository,
        settings_repo: SettingsRepository,
        http_client: reqwest::Client,
    ) -> Self {
        Self {
            transaction_repo,
            pocket_repo,
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

        // Get pocket_id: use provided one, or fall back to default pocket
        let pocket_id = match req.pocket_id {
            Some(id) => id,
            None => self.pocket_repo.get_default(user_id).await?.id,
        };

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
                pocket_id,
            )
            .await
    }

    pub async fn get_transactions(
        &self,
        user_id: Uuid,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
        pocket_id: Option<Uuid>,
        search: Option<String>,
        page: i64,
        limit: i64,
    ) -> Result<crate::schemas::PaginatedTransactions, AppError> {
        if let (Some(start), Some(end)) = (start_date, end_date) {
            if end < start {
                return Err(AppError::ValidationError(
                    "End date cannot be before start date".to_string(),
                ));
            }
        }

        // Clamp limit to reasonable values
        let limit = limit.clamp(1, 100);
        let page = page.max(1);
        let offset = (page - 1) * limit;

        let transactions = self
            .transaction_repo
            .find_by_user_and_date(
                user_id,
                start_date,
                end_date,
                pocket_id,
                search.clone(),
                limit,
                offset,
            )
            .await?;

        let total = self
            .transaction_repo
            .count_by_user_and_date(user_id, start_date, end_date, pocket_id, search)
            .await?;

        let total_pages = (total as f64 / limit as f64).ceil() as i64;

        Ok(crate::schemas::PaginatedTransactions {
            transactions,
            total,
            page,
            limit,
            total_pages,
        })
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

    pub async fn restore_transaction(&self, id: Uuid, user_id: Uuid) -> Result<(), AppError> {
        let restored = self.transaction_repo.restore(id, user_id).await?;
        if restored == 0 {
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

    pub async fn transfer_funds(
        &self,
        user_id: Uuid,
        req: crate::schemas::TransferRequest,
    ) -> Result<(), AppError> {
        if req.amount <= Decimal::ZERO {
            return Err(AppError::ValidationError(
                "Transfer amount must be positive".to_string(),
            ));
        }

        if req.source_pocket_id == req.destination_pocket_id {
            return Err(AppError::ValidationError(
                "Cannot transfer to the same pocket".to_string(),
            ));
        }

        // Verify pockets exist and belong to user
        let _source_pocket = self
            .pocket_repo
            .get_by_id(req.source_pocket_id, user_id)
            .await?;
        let _dest_pocket = self
            .pocket_repo
            .get_by_id(req.destination_pocket_id, user_id)
            .await?;

        // Check source pocket has sufficient balance
        let source_balance = self
            .transaction_repo
            .get_pocket_balance(user_id, req.source_pocket_id)
            .await?;
        if source_balance < req.amount {
            return Err(AppError::ValidationError(
                "Insufficient funds in source pocket".to_string(),
            ));
        }

        // Get special categories
        let cat_out = self
            .transaction_repo
            .get_category_by_name("Transfer Out")
            .await?;
        let cat_in = self
            .transaction_repo
            .get_category_by_name("Transfer In")
            .await?;

        // 1. Withdraw from Source
        self.transaction_repo
            .create(
                user_id,
                req.amount, // Positive amount (category indicates it's an outflow)
                Some(
                    req.description
                        .clone()
                        .unwrap_or_else(|| "Transfer Out".to_string()),
                ),
                cat_out.id,
                Utc::now(),
                None,
                None,
                None,
                req.source_pocket_id,
            )
            .await?;

        // 2. Deposit to Destination
        self.transaction_repo
            .create(
                user_id,
                req.amount, // Positive amount for income
                Some(
                    req.description
                        .clone()
                        .unwrap_or_else(|| "Transfer In".to_string()),
                ),
                cat_in.id,
                Utc::now(),
                None,
                None,
                None,
                req.destination_pocket_id,
            )
            .await?;

        Ok(())
    }
}

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

pub struct PocketService {
    pocket_repo: PocketRepository,
    transaction_repo: TransactionRepository,
}

impl PocketService {
    pub fn new(pocket_repo: PocketRepository, transaction_repo: TransactionRepository) -> Self {
        Self {
            pocket_repo,
            transaction_repo,
        }
    }

    pub async fn create_pocket(&self, user_id: Uuid, req: CreatePocket) -> Result<Uuid, AppError> {
        if req.name.trim().is_empty() {
            return Err(AppError::ValidationError(
                "Pocket name cannot be empty".to_string(),
            ));
        }

        self.pocket_repo
            .create(user_id, &req.name, req.description, req.icon)
            .await
    }

    pub async fn get_pockets(&self, user_id: Uuid) -> Result<Vec<Pocket>, AppError> {
        self.pocket_repo.get_all(user_id).await
    }

    pub async fn get_pocket(
        &self,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<crate::schemas::PocketDetail, AppError> {
        let pocket = self.pocket_repo.get_by_id(id, user_id).await?;
        let balance = self
            .transaction_repo
            .get_pocket_balance(user_id, id)
            .await?;

        Ok(crate::schemas::PocketDetail {
            id: pocket.id,
            name: pocket.name,
            description: pocket.description,
            icon: pocket.icon,
            is_default: pocket.is_default,
            created_at: pocket.created_at,
            balance,
        })
    }

    pub async fn update_pocket(
        &self,
        id: Uuid,
        user_id: Uuid,
        req: UpdatePocket,
    ) -> Result<(), AppError> {
        // Validate name if provided
        if let Some(ref name) = req.name {
            if name.trim().is_empty() {
                return Err(AppError::ValidationError(
                    "Pocket name cannot be empty".to_string(),
                ));
            }
        }

        self.pocket_repo
            .update(id, user_id, req.name, req.description, req.icon)
            .await
    }

    pub async fn delete_pocket(&self, id: Uuid, user_id: Uuid) -> Result<(), AppError> {
        let deleted = self.pocket_repo.delete(id, user_id).await?;
        if deleted == 0 {
            return Err(AppError::NotFoundError("Pocket not found".to_string()));
        }
        Ok(())
    }
}

pub struct SubscriptionService {
    subscription_repo: SubscriptionRepository,
}

impl SubscriptionService {
    pub fn new(subscription_repo: SubscriptionRepository) -> Self {
        Self { subscription_repo }
    }

    pub async fn get_subscription(&self, user_id: Uuid) -> Result<SubscriptionResponse, AppError> {
        let sub = self.subscription_repo.get_by_user(user_id).await?;
        let limits = self.compute_limits(&sub);

        Ok(SubscriptionResponse {
            plan: sub
                .as_ref()
                .map(|s| s.plan.clone())
                .unwrap_or_else(|| "free".to_string()),
            status: sub
                .as_ref()
                .map(|s| s.status.clone())
                .unwrap_or_else(|| "active".to_string()),
            expires_at: sub.as_ref().and_then(|s| s.expires_at),
            limits,
        })
    }

    pub fn compute_limits(
        &self,
        sub: &Option<crate::schemas::SubscriptionRow>,
    ) -> SubscriptionLimits {
        let plan = sub.as_ref().map(|s| s.plan.as_str()).unwrap_or("free");

        match plan {
            "premium" | "lifetime" => SubscriptionLimits {
                max_investments: None,
                max_pockets: None,
                history_days: None,
                multi_currency: true,
                pocket_transfers: true,
                advanced_analytics: true,
                data_export: true,
            },
            _ => SubscriptionLimits {
                max_investments: Some(3),
                max_pockets: Some(2),
                history_days: Some(90),
                multi_currency: false,
                pocket_transfers: false,
                advanced_analytics: false,
                data_export: false,
            },
        }
    }
}

pub struct GoalService {
    goal_repo: GoalRepository,
    entry_repo: GoalEntryRepository,
    pocket_repo: PocketRepository,
}

impl GoalService {
    pub fn new(
        goal_repo: GoalRepository,
        entry_repo: GoalEntryRepository,
        pocket_repo: PocketRepository,
    ) -> Self {
        Self {
            goal_repo,
            entry_repo,
            pocket_repo,
        }
    }

    pub async fn create_goal(
        &self,
        user_id: Uuid,
        req: crate::schemas::CreateGoal,
    ) -> Result<Uuid, AppError> {
        if req.target_amount <= Decimal::ZERO {
            return Err(AppError::ValidationError(
                "Target amount must be positive".to_string(),
            ));
        }

        if req.name.trim().is_empty() {
            return Err(AppError::ValidationError(
                "Goal name cannot be empty".to_string(),
            ));
        }

        // Verify pocket exists and belongs to user
        let _ = self.pocket_repo.get_by_id(req.pocket_id, user_id).await?;

        self.goal_repo
            .create(
                user_id,
                req.pocket_id,
                &req.name,
                req.description,
                req.target_amount,
                req.current_amount,
                req.icon,
            )
            .await
    }

    pub async fn get_goals(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<crate::schemas::GoalSummary>, AppError> {
        self.goal_repo.get_all(user_id).await
    }

    pub async fn get_goal(
        &self,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<crate::schemas::GoalDetail, AppError> {
        self.goal_repo.get_by_id(id, user_id).await
    }

    pub async fn update_goal(
        &self,
        id: Uuid,
        user_id: Uuid,
        req: crate::schemas::UpdateGoal,
    ) -> Result<(), AppError> {
        if let Some(target) = req.target_amount {
            if target <= Decimal::ZERO {
                return Err(AppError::ValidationError(
                    "Target amount must be positive".to_string(),
                ));
            }
        }

        self.goal_repo
            .update(
                id,
                user_id,
                req.name,
                req.description,
                req.target_amount,
                req.current_amount,
                req.icon,
            )
            .await
    }

    pub async fn delete_goal(&self, id: Uuid, user_id: Uuid) -> Result<(), AppError> {
        let deleted = self.goal_repo.delete(id, user_id).await?;
        if deleted == 0 {
            return Err(AppError::NotFoundError("Goal not found".to_string()));
        }
        Ok(())
    }

    pub async fn create_goal_entry(
        &self,
        goal_id: Uuid,
        user_id: Uuid,
        req: crate::schemas::CreateGoalEntry,
    ) -> Result<Uuid, AppError> {
        // 1. Verify goal ownership and get current amount
        let goal = self.goal_repo.get_by_id(goal_id, user_id).await?;

        // 2. Create entry
        let entry_id = self
            .entry_repo
            .create(goal_id, req.amount, req.description, req.date)
            .await?;

        // 3. Update goal current_amount
        let new_amount = goal.current_amount + req.amount;
        self.goal_repo
            .update(goal_id, user_id, None, None, None, Some(new_amount), None)
            .await?;

        Ok(entry_id)
    }

    pub async fn get_goal_entries(
        &self,
        goal_id: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<crate::schemas::GoalEntry>, AppError> {
        // Verify ownership
        let _ = self.goal_repo.get_by_id(goal_id, user_id).await?;
        self.entry_repo.get_by_goal(goal_id).await
    }
}

pub struct UserSubscriptionService {
    sub_repo: UserSubscriptionRepository,
    pocket_repo: PocketRepository,
    transaction_repo: TransactionRepository, // Needed for processing
}

impl UserSubscriptionService {
    pub fn new(
        sub_repo: UserSubscriptionRepository,
        pocket_repo: PocketRepository,
        transaction_repo: TransactionRepository,
    ) -> Self {
        Self {
            sub_repo,
            pocket_repo,
            transaction_repo,
        }
    }

    pub async fn create_subscription(
        &self,
        user_id: Uuid,
        req: crate::schemas::CreateUserSubscription,
    ) -> Result<Uuid, AppError> {
        if req.amount <= Decimal::ZERO {
            return Err(AppError::ValidationError(
                "Amount must be positive".to_string(),
            ));
        }

        // Validate Basis
        match req.basis.as_str() {
            "monthly" => {
                if req.billing_month.is_some() {
                    return Err(AppError::ValidationError(
                        "Billing month must be null for monthly subscriptions".to_string(),
                    ));
                }
                if !(1..=31).contains(&req.billing_day) {
                    return Err(AppError::ValidationError(
                        "Billing day must be between 1 and 31".to_string(),
                    ));
                }
            }
            "annually" => {
                if req.billing_month.is_none() {
                    return Err(AppError::ValidationError(
                        "Billing month is required for annual subscriptions".to_string(),
                    ));
                }
                if let Some(m) = req.billing_month {
                    if !(1..=12).contains(&m) {
                        return Err(AppError::ValidationError(
                            "Billing month must be between 1 and 12".to_string(),
                        ));
                    }
                }
                if !(1..=31).contains(&req.billing_day) {
                    return Err(AppError::ValidationError(
                        "Billing day must be between 1 and 31".to_string(),
                    ));
                }
            }
            _ => return Err(AppError::ValidationError("Invalid basis".to_string())),
        }

        // Verify pocket exists
        let _ = self.pocket_repo.get_by_id(req.pocket_id, user_id).await?;

        // Calculate next charge date
        let next_charge_date = self.calculate_next_charge_date(
            &req.basis,
            req.billing_day,
            req.billing_month,
            Utc::now().date_naive(), // From today
            false,                   // Start date is NOT last charge date, but today
        );

        self.sub_repo
            .create(
                user_id,
                req.pocket_id,
                &req.name,
                req.description,
                req.amount,
                &req.basis,
                req.billing_day,
                req.billing_month,
                req.category_id,
                next_charge_date,
            )
            .await
    }

    pub async fn get_subscriptions(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<crate::schemas::UserSubscriptionSummary>, AppError> {
        self.sub_repo.get_all(user_id).await
    }

    pub async fn get_subscription(
        &self,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<crate::schemas::UserSubscriptionDetail, AppError> {
        self.sub_repo.get_by_id(id, user_id).await
    }

    pub async fn update_subscription(
        &self,
        id: Uuid,
        user_id: Uuid,
        req: crate::schemas::UpdateUserSubscription,
    ) -> Result<(), AppError> {
        // Validation (simplified, full validation if basis changed is complex, assuming frontend sends valid combo)
        if let Some(amount) = req.amount {
            if amount <= Decimal::ZERO {
                return Err(AppError::ValidationError(
                    "Amount must be positive".to_string(),
                ));
            }
        }

        let mut next_charge = None;

        // If timing changed, recalculate next_charge_date
        // Fetch current to merge
        if req.basis.is_some() || req.billing_day.is_some() || req.billing_month.is_some() {
            let current = self.sub_repo.get_by_id(id, user_id).await?;
            let basis = req.basis.as_deref().unwrap_or(&current.basis);
            let day = req.billing_day.unwrap_or(current.billing_day);
            let month = req.billing_month.or(current.billing_month); // Careful with Option merging

            // Recalculate from TODAY. If user changes date, we assume they want next occurance from now.
            next_charge = Some(self.calculate_next_charge_date(
                basis,
                day,
                month,
                Utc::now().date_naive(),
                false,
            ));
        }

        self.sub_repo
            .update(
                id,
                user_id,
                req.name,
                req.description,
                req.amount,
                req.basis,
                req.billing_day,
                req.billing_month,
                req.category_id,
                req.is_active,
                req.pocket_id,
                next_charge,
            )
            .await
    }

    pub async fn delete_subscription(&self, id: Uuid, user_id: Uuid) -> Result<(), AppError> {
        let deleted = self.sub_repo.delete(id, user_id).await?;
        if deleted == 0 {
            return Err(AppError::NotFoundError(
                "Subscription not found".to_string(),
            ));
        }
        Ok(())
    }

    /// Core logic for calculating the next charge date
    fn calculate_next_charge_date(
        &self,
        basis: &str,
        billing_day: i32,
        billing_month: Option<i32>,
        reference_date: NaiveDate, // Usually today, or last charge date
        is_retry: bool, // If true, reference_date is last charge date, find next. If false, find first occurance >= reference_date
    ) -> NaiveDate {
        let billing_day = billing_day as u32;
        let mut target_year = reference_date.year();
        let mut target_month = reference_date.month();

        if basis == "monthly" {
            if is_retry {
                // Next month from reference
                if target_month == 12 {
                    target_month = 1;
                    target_year += 1;
                } else {
                    target_month += 1;
                }
            } else {
                // If today > billing_day, move to next month
                if reference_date.day() > billing_day {
                    if target_month == 12 {
                        target_month = 1;
                        target_year += 1;
                    } else {
                        target_month += 1;
                    }
                }
            }
            // Handle end of month logic
            Self::get_valid_date(target_year, target_month, billing_day)
        } else {
            // Annual
            let billing_month = billing_month.unwrap_or(1) as u32;

            if is_retry {
                // Next year
                target_year += 1;
            } else {
                // If today > billing_date (approx), move to next year
                // Simplistic check: constructed date < reference?
                let this_year_date = Self::get_valid_date(target_year, billing_month, billing_day);
                if this_year_date < reference_date {
                    target_year += 1;
                }
            }
            Self::get_valid_date(target_year, billing_month, billing_day)
        }
    }

    /// Helper to handle "Feb 31" -> "Feb 28/29"
    fn get_valid_date(year: i32, month: u32, day: u32) -> NaiveDate {
        // Try to create date. If fail, it's likely invalid day (e.g. 31 for Feb)
        // chrono::NaiveDate::from_ymd_opt(year, month, day).unwrap() // panic if invalid

        // Better approach:
        // If day is valid, return it.
        // If not, find last day of that month.
        if let Some(d) = NaiveDate::from_ymd_opt(year, month, day) {
            d
        } else {
            // Day is too large. Get last day of month.
            // Go to next month, day 1, subtract 1 day.
            let (next_y, next_m) = if month == 12 {
                (year + 1, 1)
            } else {
                (year, month + 1)
            };
            NaiveDate::from_ymd_opt(next_y, next_m, 1)
                .unwrap()
                .pred_opt()
                .unwrap()
        }
    }

    // Logic for Worker: Process a subscription
    pub async fn process_subscription(&self, _sub_id: Uuid) -> Result<(), AppError> {
        // 1. Fetch sub directly (bypass user check as worker is system)
        // We need a system-level get method or reuse get_by_id if we assume user_id context not needed for lookup (but repo enforces it).
        // Actually, we likely need a `get_by_id_system` in repo.
        // For now, let's assume we fetch the row using `TransactionRepository` or similar raw query if necessary,
        // OR we add `get_by_id_system` to `UserSubscriptionRepository`.
        // Let's assume we add it or use a raw query here for simplicity if needed, but adding to repo is cleaner.
        // I'll assume `sub_repo` has `get_for_processing` or similar.
        // Wait, I implemented `get_due_subscriptions` in repo which returns `UserSubscriptionRow`.
        // I can use that row. But the worker passes `sub_id`.
        // So I need `get_row_by_id`.

        // Let's implement `get_row_by_id` in repo? Or just do it here.
        // I'll add `process_subscription_transaction` which takes the ROW.
        // But the worker structure in the plan says: "Listens to jobs. Fetches sub details".

        // I will add `get_system` to repo later. For now, let's implement the core Logic assuming we have the data.
        Ok(())
    }

    // Using the row data to process
    pub async fn process_due_subscription(
        &self,
        sub: &crate::schemas::UserSubscriptionRow,
    ) -> Result<(), AppError> {
        // 1. Create Transaction
        // We need a category. If sub.category_id is None, use "Subscriptions" (we created it in migration).
        // If we don't know the ID, we look it up.
        // Ideally we cache this or fetch it.

        let category_id = if let Some(id) = sub.category_id {
            id
        } else {
            // Fallback to finding "Subscriptions"
            let cat = self
                .transaction_repo
                .get_category_by_name("Subscriptions")
                .await?;
            cat.id
        };

        // Create transaction
        let _tx_id = self
            .transaction_repo
            .create(
                sub.user_id,
                sub.amount,
                Some(format!("Subscription: {}", sub.name)),
                category_id,
                Utc::now(), // Occurred NOW
                None,
                None,
                None,
                sub.pocket_id,
            )
            .await?;

        // 2. Calculate NEXT charge date
        let next_date = self.calculate_next_charge_date(
            &sub.basis,
            sub.billing_day,
            sub.billing_month,
            sub.next_charge_date, // base on PREVIOUS scheduled date to keep cadence? OR Today?
            // Usually base on previous to keep "1st of month" alignment even if run late.
            true,
        );

        // 3. Update subscription
        self.sub_repo
            .update_next_charge_date(sub.id, next_date)
            .await?;

        Ok(())
    }
}
