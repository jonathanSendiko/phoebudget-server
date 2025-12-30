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
    AuthResponse, CategorySummary, CreatePortfolioItem, CreateTransaction, FinancialHealth,
    LoginRequest, RegisterRequest, Transaction, UpdateInvestment, UserProfile,
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
}

impl TransactionService {
    pub fn new(transaction_repo: TransactionRepository) -> Self {
        Self { transaction_repo }
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
        let description = req.description.filter(|d| !d.trim().is_empty());

        self.transaction_repo
            .create(
                user_id,
                req.amount,
                description,
                req.category_id,
                req.occurred_at,
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
    ) -> Result<Vec<CategorySummary>, AppError> {
        self.transaction_repo
            .get_spending_analysis(user_id, start_date, end_date)
            .await
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
}

pub struct FinanceService {
    portfolio_repo: PortfolioRepository,
    transaction_repo: TransactionRepository,
    settings_repo: SettingsRepository,
    price_cache: moka::future::Cache<String, Decimal>,
}

impl FinanceService {
    pub fn new(
        portfolio_repo: PortfolioRepository,
        transaction_repo: TransactionRepository,
        settings_repo: SettingsRepository,
        price_cache: moka::future::Cache<String, Decimal>,
    ) -> Self {
        Self {
            portfolio_repo,
            transaction_repo,
            settings_repo,
            price_cache,
        }
    }

    pub async fn get_financial_health(&self, user_id: Uuid) -> Result<FinancialHealth, AppError> {
        let base_currency = self.settings_repo.get_base_currency(user_id).await?;
        let cash = self.transaction_repo.get_net_cash(user_id).await?;
        let invested_usd = self.portfolio_repo.get_total_invested(user_id).await?;

        let rate = if base_currency != "USD" {
            investments::fetch_exchange_rate("USD", &base_currency).await?
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
        let mut updated_count = 0;

        for ticker in tickers {
            self.ensure_price_fresh(&ticker).await?;
            updated_count += 1;
        }

        Ok(updated_count)
    }

    pub async fn add_investment(
        &self,
        user_id: Uuid,
        item: CreatePortfolioItem,
    ) -> Result<(), AppError> {
        // Validate ticker and ensure price is in assets table
        self.ensure_price_fresh(&item.ticker).await?;

        // Ensure asset exists in DB (already done by update_asset_price if we did it right,
        // but ensure_asset_exists handles the initial insert if not present)
        // Actually fetch_price calls Yahoo, if valid we proceed.

        self.portfolio_repo
            .ensure_asset_exists(&item.ticker)
            .await?;

        match self.portfolio_repo.add_item(user_id, item.clone()).await {
            Ok(_) => Ok(()),
            Err(e) => {
                // AppError is likely DatabaseError
                // Simple string matching for now
                let msg = format!("{:?}", e);
                if msg.contains("duplicate key value") {
                    Err(AppError::ValidationError(format!(
                        "{} is already in your portfolio",
                        &item.ticker
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

        let price = investments::fetch_price(ticker).await?;

        // Update DB
        // If asset doesn't exist, update_asset_price might list 0 rows affected if used strict update.
        // But we usually ensure asset exists before.
        // Let's assume asset exists or we act gracefully.
        // Actually, for "Refresh", assets exist.
        // For "Add", we call ensure_asset_exists first?
        // Let's call ensure_asset_exists here if we want to be safe, or just update.
        // PortfolioRepo::update_asset_price works on 'assets' table.
        // If the asset isn't in 'assets' table, this does nothing.
        // Optimization: Check DB first? No.

        // We need to ensure the asset record exists to store the price.
        self.portfolio_repo.ensure_asset_exists(ticker).await?;
        self.portfolio_repo
            .update_asset_price(ticker, price)
            .await?;

        self.price_cache.insert(ticker.to_string(), price).await;

        Ok(price)
    }

    pub async fn get_portfolio_list(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<crate::schemas::InvestmentSummary>, AppError> {
        // Ensure prices are fresh first
        let tickers = self.portfolio_repo.get_tickers(user_id).await?;
        for ticker in tickers {
            // We ignore errors here to allow partial success?
            // Or we strictly fail if one price/API fails?
            // "get investments... fetch from cache... avoiding race condition"
            // Let's propagate error for now to be safe, or log it?
            // Given fetch_price might fail if API is down, maybe we shouldn't block the UI?
            // But user requirement implies we must fetch.
            // Let's await properly.
            self.ensure_price_fresh(&ticker).await?;
        }

        let raw_items = self.portfolio_repo.get_all_joined(user_id).await?;
        let mut summary_list = Vec::new();

        for (ticker, name, quantity, avg_buy, current_price) in raw_items {
            let total_value = quantity * current_price;

            // Calculate Change %
            // If avg_buy is 0 (shouldn't happen properly but safety first), change is 0.
            let change_pct = if avg_buy > Decimal::ZERO {
                ((current_price - avg_buy) / avg_buy) * Decimal::from(100)
            } else {
                Decimal::ZERO
            };

            summary_list.push(crate::schemas::InvestmentSummary {
                ticker,
                name,
                quantity,
                avg_buy_price: avg_buy,
                current_price,
                total_value,
                change_pct,
            });
        }

        Ok(summary_list)
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
