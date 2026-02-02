use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::error::AppError;
use crate::investments;
use crate::repository::{PocketRepository, SettingsRepository, TransactionRepository};
use crate::schemas::{Category, CreateTransaction, Pocket, TransactionDetail};

#[async_trait]
pub trait TransactionRepo: Send + Sync {
    async fn get_all_categories(&self) -> Result<Vec<Category>, AppError>;
    async fn get_category_by_name(&self, name: &str) -> Result<Category, AppError>;
    async fn create(
        &self,
        user_id: Uuid,
        amount: Decimal,
        description: Option<String>,
        category_id: i32,
        occurred_at: DateTime<Utc>,
        original_currency: Option<String>,
        original_amount: Option<Decimal>,
        exchange_rate: Option<Decimal>,
        pocket_id: Uuid,
    ) -> Result<Uuid, AppError>;
    async fn find_by_user_and_date(
        &self,
        user_id: Uuid,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
        pocket_id: Option<Uuid>,
        search: Option<String>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<crate::schemas::Transaction>, AppError>;
    async fn count_by_user_and_date(
        &self,
        user_id: Uuid,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
        pocket_id: Option<Uuid>,
        search: Option<String>,
    ) -> Result<i64, AppError>;
    async fn get_spending_analysis(
        &self,
        user_id: Uuid,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<Vec<crate::schemas::CategorySummary>, AppError>;
    async fn update(
        &self,
        id: Uuid,
        user_id: Uuid,
        amount: Option<Decimal>,
        description: Option<String>,
        category_id: Option<i32>,
        occurred_at: Option<DateTime<Utc>>,
        original_currency: Option<String>,
        original_amount: Option<Decimal>,
        exchange_rate: Option<Decimal>,
    ) -> Result<(), AppError>;
    async fn delete(&self, id: Uuid, user_id: Uuid) -> Result<u64, AppError>;
    async fn restore(&self, id: Uuid, user_id: Uuid) -> Result<u64, AppError>;
    async fn get_transaction(&self, id: Uuid, user_id: Uuid) -> Result<TransactionDetail, AppError>;
    async fn get_net_cash(&self, user_id: Uuid) -> Result<Decimal, AppError>;
    async fn get_pocket_balance(&self, user_id: Uuid, pocket_id: Uuid)
        -> Result<Decimal, AppError>;
}

#[async_trait]
pub trait PocketRepo: Send + Sync {
    async fn get_default(&self, user_id: Uuid) -> Result<Pocket, AppError>;
    async fn get_by_id(&self, id: Uuid, user_id: Uuid) -> Result<Pocket, AppError>;
}

#[async_trait]
pub trait SettingsRepo: Send + Sync {
    async fn get_base_currency(&self, user_id: Uuid) -> Result<String, AppError>;
    async fn validate_currency(&self, code: &str) -> Result<bool, AppError>;
    async fn set_base_currency(&self, user_id: Uuid, currency: &str) -> Result<(), AppError>;
}

#[async_trait]
pub trait ExchangeRateProvider: Send + Sync {
    async fn fetch_rate(&self, from: &str, to: &str) -> Result<Decimal, AppError>;
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
impl TransactionRepo for TransactionRepository {
    async fn get_all_categories(&self) -> Result<Vec<Category>, AppError> {
        self.get_all_categories().await
    }

    async fn get_category_by_name(&self, name: &str) -> Result<Category, AppError> {
        self.get_category_by_name(name).await
    }

    async fn create(
        &self,
        user_id: Uuid,
        amount: Decimal,
        description: Option<String>,
        category_id: i32,
        occurred_at: DateTime<Utc>,
        original_currency: Option<String>,
        original_amount: Option<Decimal>,
        exchange_rate: Option<Decimal>,
        pocket_id: Uuid,
    ) -> Result<Uuid, AppError> {
        self.create(
            user_id,
            amount,
            description,
            category_id,
            occurred_at,
            original_currency,
            original_amount,
            exchange_rate,
            pocket_id,
        )
        .await
    }

    async fn find_by_user_and_date(
        &self,
        user_id: Uuid,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
        pocket_id: Option<Uuid>,
        search: Option<String>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<crate::schemas::Transaction>, AppError> {
        self.find_by_user_and_date(user_id, start_date, end_date, pocket_id, search, limit, offset)
            .await
    }

    async fn count_by_user_and_date(
        &self,
        user_id: Uuid,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
        pocket_id: Option<Uuid>,
        search: Option<String>,
    ) -> Result<i64, AppError> {
        self.count_by_user_and_date(user_id, start_date, end_date, pocket_id, search)
            .await
    }

    async fn get_spending_analysis(
        &self,
        user_id: Uuid,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<Vec<crate::schemas::CategorySummary>, AppError> {
        self.get_spending_analysis(user_id, start_date, end_date)
            .await
    }

    async fn update(
        &self,
        id: Uuid,
        user_id: Uuid,
        amount: Option<Decimal>,
        description: Option<String>,
        category_id: Option<i32>,
        occurred_at: Option<DateTime<Utc>>,
        original_currency: Option<String>,
        original_amount: Option<Decimal>,
        exchange_rate: Option<Decimal>,
    ) -> Result<(), AppError> {
        self.update(
            id,
            user_id,
            amount,
            description,
            category_id,
            occurred_at,
            original_currency,
            original_amount,
            exchange_rate,
        )
        .await
    }

    async fn delete(&self, id: Uuid, user_id: Uuid) -> Result<u64, AppError> {
        self.delete(id, user_id).await
    }

    async fn restore(&self, id: Uuid, user_id: Uuid) -> Result<u64, AppError> {
        self.restore(id, user_id).await
    }

    async fn get_transaction(&self, id: Uuid, user_id: Uuid) -> Result<TransactionDetail, AppError> {
        self.get_transaction(id, user_id).await
    }

    async fn get_net_cash(&self, user_id: Uuid) -> Result<Decimal, AppError> {
        self.get_net_cash(user_id).await
    }

    async fn get_pocket_balance(
        &self,
        user_id: Uuid,
        pocket_id: Uuid,
    ) -> Result<Decimal, AppError> {
        self.get_pocket_balance(user_id, pocket_id).await
    }
}

#[async_trait]
impl PocketRepo for PocketRepository {
    async fn get_default(&self, user_id: Uuid) -> Result<Pocket, AppError> {
        self.get_default(user_id).await
    }

    async fn get_by_id(&self, id: Uuid, user_id: Uuid) -> Result<Pocket, AppError> {
        self.get_by_id(id, user_id).await
    }
}

#[async_trait]
impl SettingsRepo for SettingsRepository {
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

pub type TransactionServiceImpl = TransactionService<
    TransactionRepository,
    PocketRepository,
    SettingsRepository,
    HttpExchangeRateProvider,
>;

pub struct TransactionService<TxRepo, PRepo, SRepo, FxProvider> {
    transaction_repo: TxRepo,
    pocket_repo: PRepo,
    settings_repo: SRepo,
    exchange_rate_provider: FxProvider,
}

impl<TxRepo, PRepo, SRepo, FxProvider> TransactionService<TxRepo, PRepo, SRepo, FxProvider>
where
    TxRepo: TransactionRepo,
    PRepo: PocketRepo,
    SRepo: SettingsRepo,
    FxProvider: ExchangeRateProvider,
{
    pub fn new(
        transaction_repo: TxRepo,
        pocket_repo: PRepo,
        settings_repo: SRepo,
        exchange_rate_provider: FxProvider,
    ) -> Self {
        Self {
            transaction_repo,
            pocket_repo,
            settings_repo,
            exchange_rate_provider,
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
                let rate = self
                    .exchange_rate_provider
                    .fetch_rate(currency, &base_currency)
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
