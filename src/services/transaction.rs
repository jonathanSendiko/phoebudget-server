use async_trait::async_trait;
use chrono::{DateTime, Datelike, Duration, Months, Timelike, Utc};
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
        category_ids: Option<Vec<i32>>,
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
        category_ids: Option<Vec<i32>>,
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
    async fn get_transaction(&self, id: Uuid, user_id: Uuid)
    -> Result<TransactionDetail, AppError>;
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
        category_ids: Option<Vec<i32>>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<crate::schemas::Transaction>, AppError> {
        self.find_by_user_and_date(
            user_id,
            start_date,
            end_date,
            pocket_id,
            search,
            category_ids,
            limit,
            offset,
        )
        .await
    }

    async fn count_by_user_and_date(
        &self,
        user_id: Uuid,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
        pocket_id: Option<Uuid>,
        search: Option<String>,
        category_ids: Option<Vec<i32>>,
    ) -> Result<i64, AppError> {
        self.count_by_user_and_date(user_id, start_date, end_date, pocket_id, search, category_ids)
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

    async fn get_transaction(
        &self,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<TransactionDetail, AppError> {
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
        let (amount, original_currency, original_amount, exchange_rate) =
            if let Some(currency) = &req.currency_code {
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
        category_ids: Option<Vec<i32>>,
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

        let category_ids = category_ids.filter(|ids| !ids.is_empty());
        let transactions = self
            .transaction_repo
            .find_by_user_and_date(
                user_id,
                start_date,
                end_date,
                pocket_id,
                search.clone(),
                category_ids.clone(),
                limit,
                offset,
            )
            .await?;

        let total = self
            .transaction_repo
            .count_by_user_and_date(user_id, start_date, end_date, pocket_id, search, category_ids)
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
        let comparison_percentage = if is_full_month_range(start_date, end_date) {
            let (previous_start, previous_end) = previous_month_bounds(start_date)?;
            let previous_categories = self
                .transaction_repo
                .get_spending_analysis(user_id, previous_start, previous_end)
                .await?;

            let previous_total_spent = previous_categories
                .iter()
                .filter(|cat| !cat.is_income)
                .fold(Decimal::ZERO, |acc, cat| acc + cat.total);

            calculate_percentage_change(total_spent, previous_total_spent)
        } else {
            None
        };

        Ok(crate::schemas::SpendingAnalysisResponse {
            total_income,
            total_spent,
            net_income,
            comparison_percentage,
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

fn is_full_month_range(start_date: DateTime<Utc>, end_date: DateTime<Utc>) -> bool {
    if start_date.year() != end_date.year() || start_date.month() != end_date.month() {
        return false;
    }

    let last_day = last_day_of_month(start_date.year(), start_date.month());

    start_date.day() == 1
        && start_date.hour() == 0
        && start_date.minute() == 0
        && start_date.second() == 0
        && start_date.nanosecond() == 0
        && end_date.day() == last_day
        && end_date.hour() == 23
        && end_date.minute() == 59
        && end_date.second() == 59
}

fn previous_month_bounds(
    current_month_start: DateTime<Utc>,
) -> Result<(DateTime<Utc>, DateTime<Utc>), AppError> {
    let previous_month_start_date = current_month_start
        .date_naive()
        .checked_sub_months(Months::new(1))
        .ok_or_else(|| {
            AppError::InternalServerError("Failed to compute previous month bounds".to_string())
        })?;

    let previous_month_start = DateTime::<Utc>::from_naive_utc_and_offset(
        previous_month_start_date
            .and_hms_nano_opt(0, 0, 0, 0)
            .ok_or_else(|| {
                AppError::InternalServerError("Failed to compute previous month start".to_string())
            })?,
        Utc,
    );
    let previous_month_end = current_month_start - Duration::nanoseconds(1);

    Ok((previous_month_start, previous_month_end))
}

fn calculate_percentage_change(current: Decimal, previous: Decimal) -> Option<Decimal> {
    if previous == Decimal::ZERO {
        return None;
    }

    Some((current - previous) / previous * Decimal::new(100, 0))
}

fn last_day_of_month(year: i32, month: u32) -> u32 {
    let (next_year, next_month) = if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    };

    let next_month_first =
        chrono::NaiveDate::from_ymd_opt(next_year, next_month, 1).expect("valid month start");
    (next_month_first - Duration::days(1)).day()
}

#[cfg(test)]
mod tests {
    use super::{
        ExchangeRateProvider, PocketRepo, SettingsRepo, TransactionRepo, TransactionService,
    };
    use crate::error::AppError;
    use crate::schemas::{
        Category, CategorySummary, CreateTransaction, Pocket, Transaction, TransferRequest,
    };
    use async_trait::async_trait;
    use chrono::{DateTime, TimeZone, Timelike, Utc};
    use rust_decimal::Decimal;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use uuid::Uuid;

    #[derive(Clone, Default)]
    struct MockTransactionRepo {
        state: Arc<Mutex<MockTransactionState>>,
    }

    struct MockTransactionState {
        create_calls: Vec<CreateCall>,
        find_args: Option<FindArgs>,
        count_args: Option<CountArgs>,
        spending_analysis_calls: Vec<SpendingAnalysisCall>,
        spending_analysis_results: Vec<SpendingAnalysisResult>,
        category_by_name_calls: Vec<String>,
        get_pocket_balance: Decimal,
        categories: HashMap<String, CategoryStub>,
        total_count: i64,
        next_create_id: Uuid,
    }

    impl Default for MockTransactionState {
        fn default() -> Self {
            Self {
                create_calls: Vec::new(),
                find_args: None,
                count_args: None,
                spending_analysis_calls: Vec::new(),
                spending_analysis_results: Vec::new(),
                category_by_name_calls: Vec::new(),
                get_pocket_balance: Decimal::ZERO,
                categories: HashMap::new(),
                total_count: 0,
                next_create_id: Uuid::nil(),
            }
        }
    }

    #[derive(Clone)]
    struct CategoryStub {
        id: i32,
        is_income: bool,
        icon: String,
        exclude_from_analysis: bool,
    }

    #[derive(Clone)]
    #[allow(dead_code)]
    struct CreateCall {
        user_id: Uuid,
        amount: Decimal,
        description: Option<String>,
        category_id: i32,
        occurred_at: DateTime<Utc>,
        original_currency: Option<String>,
        original_amount: Option<Decimal>,
        exchange_rate: Option<Decimal>,
        pocket_id: Uuid,
    }

    #[derive(Clone)]
    #[allow(dead_code)]
    struct FindArgs {
        user_id: Uuid,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
        pocket_id: Option<Uuid>,
        search: Option<String>,
        category_ids: Option<Vec<i32>>,
        limit: i64,
        offset: i64,
    }

    #[derive(Clone)]
    #[allow(dead_code)]
    struct CountArgs {
        user_id: Uuid,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
        pocket_id: Option<Uuid>,
        search: Option<String>,
        category_ids: Option<Vec<i32>>,
    }

    #[derive(Clone)]
    struct SpendingAnalysisCall {
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    }

    #[derive(Clone)]
    struct SpendingAnalysisResult {
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
        categories: Vec<CategorySummary>,
    }

    #[async_trait]
    impl TransactionRepo for MockTransactionRepo {
        async fn get_all_categories(&self) -> Result<Vec<Category>, AppError> {
            Ok(Vec::new())
        }

        async fn get_category_by_name(&self, name: &str) -> Result<Category, AppError> {
            let mut state = self.state.lock().unwrap();
            state.category_by_name_calls.push(name.to_string());
            state
                .categories
                .get(name)
                .map(|stub| Category {
                    id: stub.id,
                    name: name.to_string(),
                    is_income: stub.is_income,
                    icon: stub.icon.clone(),
                    exclude_from_analysis: stub.exclude_from_analysis,
                })
                .ok_or_else(|| AppError::NotFoundError(format!("Category '{}' not found", name)))
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
            let mut state = self.state.lock().unwrap();
            state.create_calls.push(CreateCall {
                user_id,
                amount,
                description,
                category_id,
                occurred_at,
                original_currency,
                original_amount,
                exchange_rate,
                pocket_id,
            });
            Ok(state.next_create_id)
        }

        async fn find_by_user_and_date(
            &self,
            user_id: Uuid,
            start_date: Option<DateTime<Utc>>,
            end_date: Option<DateTime<Utc>>,
            pocket_id: Option<Uuid>,
            search: Option<String>,
            category_ids: Option<Vec<i32>>,
            limit: i64,
            offset: i64,
        ) -> Result<Vec<Transaction>, AppError> {
            let mut state = self.state.lock().unwrap();
            state.find_args = Some(FindArgs {
                user_id,
                start_date,
                end_date,
                pocket_id,
                search,
                category_ids,
                limit,
                offset,
            });
            Ok(Vec::new())
        }

        async fn count_by_user_and_date(
            &self,
            user_id: Uuid,
            start_date: Option<DateTime<Utc>>,
            end_date: Option<DateTime<Utc>>,
            pocket_id: Option<Uuid>,
            search: Option<String>,
            category_ids: Option<Vec<i32>>,
        ) -> Result<i64, AppError> {
            let mut state = self.state.lock().unwrap();
            state.count_args = Some(CountArgs {
                user_id,
                start_date,
                end_date,
                pocket_id,
                search,
                category_ids,
            });
            Ok(state.total_count)
        }

        async fn get_spending_analysis(
            &self,
            _user_id: Uuid,
            start_date: DateTime<Utc>,
            end_date: DateTime<Utc>,
        ) -> Result<Vec<CategorySummary>, AppError> {
            let mut state = self.state.lock().unwrap();
            state.spending_analysis_calls.push(SpendingAnalysisCall {
                start_date,
                end_date,
            });

            Ok(state
                .spending_analysis_results
                .iter()
                .find(|result| result.start_date == start_date && result.end_date == end_date)
                .map(|result| result.categories.clone())
                .unwrap_or_default())
        }

        async fn update(
            &self,
            _id: Uuid,
            _user_id: Uuid,
            _amount: Option<Decimal>,
            _description: Option<String>,
            _category_id: Option<i32>,
            _occurred_at: Option<DateTime<Utc>>,
            _original_currency: Option<String>,
            _original_amount: Option<Decimal>,
            _exchange_rate: Option<Decimal>,
        ) -> Result<(), AppError> {
            Ok(())
        }

        async fn delete(&self, _id: Uuid, _user_id: Uuid) -> Result<u64, AppError> {
            Ok(1)
        }

        async fn restore(&self, _id: Uuid, _user_id: Uuid) -> Result<u64, AppError> {
            Ok(1)
        }

        async fn get_transaction(
            &self,
            _id: Uuid,
            _user_id: Uuid,
        ) -> Result<crate::schemas::TransactionDetail, AppError> {
            Err(AppError::InternalServerError(
                "Mock not configured".to_string(),
            ))
        }

        async fn get_net_cash(&self, _user_id: Uuid) -> Result<Decimal, AppError> {
            Ok(Decimal::ZERO)
        }

        async fn get_pocket_balance(
            &self,
            _user_id: Uuid,
            _pocket_id: Uuid,
        ) -> Result<Decimal, AppError> {
            let state = self.state.lock().unwrap();
            Ok(state.get_pocket_balance)
        }
    }

    #[derive(Clone)]
    struct MockPocketRepo {
        state: Arc<Mutex<MockPocketState>>,
    }

    struct MockPocketState {
        default_pocket: Pocket,
        pockets: HashMap<Uuid, Pocket>,
        get_default_calls: usize,
        get_by_id_calls: usize,
    }

    impl Default for MockPocketState {
        fn default() -> Self {
            Self {
                default_pocket: default_pocket(Uuid::nil()),
                pockets: HashMap::new(),
                get_default_calls: 0,
                get_by_id_calls: 0,
            }
        }
    }

    #[async_trait]
    impl PocketRepo for MockPocketRepo {
        async fn get_default(&self, _user_id: Uuid) -> Result<Pocket, AppError> {
            let mut state = self.state.lock().unwrap();
            state.get_default_calls += 1;
            Ok(state.default_pocket.clone())
        }

        async fn get_by_id(&self, id: Uuid, _user_id: Uuid) -> Result<Pocket, AppError> {
            let mut state = self.state.lock().unwrap();
            state.get_by_id_calls += 1;
            state
                .pockets
                .get(&id)
                .cloned()
                .ok_or_else(|| AppError::NotFoundError("Pocket not found".to_string()))
        }
    }

    #[derive(Clone)]
    struct MockSettingsRepo {
        base_currency: String,
    }

    #[async_trait]
    impl SettingsRepo for MockSettingsRepo {
        async fn get_base_currency(&self, _user_id: Uuid) -> Result<String, AppError> {
            Ok(self.base_currency.clone())
        }

        async fn validate_currency(&self, _code: &str) -> Result<bool, AppError> {
            Ok(true)
        }

        async fn set_base_currency(&self, _user_id: Uuid, _currency: &str) -> Result<(), AppError> {
            Ok(())
        }
    }

    #[derive(Clone, Default)]
    struct MockExchangeRateProvider {
        rate: Decimal,
        calls: Arc<Mutex<Vec<(String, String)>>>,
    }

    #[async_trait]
    impl ExchangeRateProvider for MockExchangeRateProvider {
        async fn fetch_rate(&self, from: &str, to: &str) -> Result<Decimal, AppError> {
            let mut calls = self.calls.lock().unwrap();
            calls.push((from.to_string(), to.to_string()));
            Ok(self.rate)
        }
    }

    fn default_pocket(id: Uuid) -> Pocket {
        Pocket {
            id,
            name: "Default".to_string(),
            description: None,
            icon: "wallet".to_string(),
            is_default: true,
            created_at: None,
        }
    }

    fn utc_datetime(
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        minute: u32,
        second: u32,
        nanosecond: u32,
    ) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(year, month, day, hour, minute, second)
            .single()
            .unwrap()
            .with_nanosecond(nanosecond)
            .unwrap()
    }

    fn make_transaction_service(
        tx_repo: MockTransactionRepo,
        pocket_repo: MockPocketRepo,
        settings_repo: MockSettingsRepo,
        fx: MockExchangeRateProvider,
    ) -> TransactionService<
        MockTransactionRepo,
        MockPocketRepo,
        MockSettingsRepo,
        MockExchangeRateProvider,
    > {
        TransactionService::new(tx_repo, pocket_repo, settings_repo, fx)
    }

    #[tokio::test]
    async fn create_transaction_rejects_non_positive_amount() {
        let tx_repo = MockTransactionRepo::default();
        let pocket_repo = MockPocketRepo {
            state: Arc::new(Mutex::new(MockPocketState {
                default_pocket: default_pocket(Uuid::new_v4()),
                ..Default::default()
            })),
        };
        let settings_repo = MockSettingsRepo {
            base_currency: "USD".to_string(),
        };
        let fx = MockExchangeRateProvider::default();
        let service = make_transaction_service(tx_repo, pocket_repo, settings_repo, fx);

        let req = CreateTransaction {
            amount: Decimal::ZERO,
            description: None,
            category_id: 1,
            occurred_at: DateTime::<Utc>::from_timestamp(1_700_000_000, 0).unwrap(),
            currency_code: None,
            pocket_id: None,
        };

        let err = service
            .create_transaction(Uuid::new_v4(), req)
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::ValidationError(msg) if msg == "Amount must be positive"));
    }

    #[tokio::test]
    async fn create_transaction_converts_currency_and_sets_originals() {
        let mut tx_state = MockTransactionState::default();
        tx_state.next_create_id = Uuid::new_v4();
        let tx_repo = MockTransactionRepo {
            state: Arc::new(Mutex::new(tx_state)),
        };
        let pocket_id = Uuid::new_v4();
        let pocket_repo = MockPocketRepo {
            state: Arc::new(Mutex::new(MockPocketState {
                default_pocket: default_pocket(pocket_id),
                ..Default::default()
            })),
        };
        let settings_repo = MockSettingsRepo {
            base_currency: "USD".to_string(),
        };
        let fx = MockExchangeRateProvider {
            rate: Decimal::new(2, 0),
            ..Default::default()
        };
        let service =
            make_transaction_service(tx_repo.clone(), pocket_repo, settings_repo, fx.clone());

        let req = CreateTransaction {
            amount: Decimal::new(10, 0),
            description: Some("Lunch".to_string()),
            category_id: 7,
            occurred_at: DateTime::<Utc>::from_timestamp(1_700_000_000, 0).unwrap(),
            currency_code: Some("EUR".to_string()),
            pocket_id: None,
        };

        let id = service
            .create_transaction(Uuid::new_v4(), req)
            .await
            .unwrap();
        assert_eq!(id, tx_repo.state.lock().unwrap().next_create_id);

        let state = tx_repo.state.lock().unwrap();
        let call = state.create_calls.first().expect("create call");
        assert_eq!(call.amount, Decimal::new(20, 0));
        assert_eq!(call.original_currency, Some("EUR".to_string()));
        assert_eq!(call.original_amount, Some(Decimal::new(10, 0)));
        assert_eq!(call.exchange_rate, Some(Decimal::new(2, 0)));

        let calls = fx.calls.lock().unwrap();
        assert_eq!(calls.as_slice(), &[("EUR".to_string(), "USD".to_string())]);
    }

    #[tokio::test]
    async fn create_transaction_uses_default_pocket_when_missing() {
        let tx_repo = MockTransactionRepo::default();
        let default_id = Uuid::new_v4();
        let pocket_repo = MockPocketRepo {
            state: Arc::new(Mutex::new(MockPocketState {
                default_pocket: default_pocket(default_id),
                ..Default::default()
            })),
        };
        let settings_repo = MockSettingsRepo {
            base_currency: "USD".to_string(),
        };
        let fx = MockExchangeRateProvider::default();
        let service =
            make_transaction_service(tx_repo.clone(), pocket_repo.clone(), settings_repo, fx);

        let req = CreateTransaction {
            amount: Decimal::new(5, 0),
            description: Some("".to_string()),
            category_id: 1,
            occurred_at: DateTime::<Utc>::from_timestamp(1_700_000_000, 0).unwrap(),
            currency_code: None,
            pocket_id: None,
        };

        service
            .create_transaction(Uuid::new_v4(), req)
            .await
            .unwrap();
        let state = tx_repo.state.lock().unwrap();
        let call = state.create_calls.first().expect("create call");
        assert_eq!(call.pocket_id, default_id);
        assert_eq!(call.description, None);

        let pocket_state = pocket_repo.state.lock().unwrap();
        assert_eq!(pocket_state.get_default_calls, 1);
    }

    #[tokio::test]
    async fn create_transaction_skips_conversion_when_same_currency() {
        let tx_repo = MockTransactionRepo::default();
        let pocket_repo = MockPocketRepo {
            state: Arc::new(Mutex::new(MockPocketState {
                default_pocket: default_pocket(Uuid::new_v4()),
                ..Default::default()
            })),
        };
        let settings_repo = MockSettingsRepo {
            base_currency: "USD".to_string(),
        };
        let fx = MockExchangeRateProvider::default();
        let service =
            make_transaction_service(tx_repo.clone(), pocket_repo, settings_repo, fx.clone());

        let req = CreateTransaction {
            amount: Decimal::new(12, 0),
            description: Some("Coffee".to_string()),
            category_id: 1,
            occurred_at: DateTime::<Utc>::from_timestamp(1_700_000_000, 0).unwrap(),
            currency_code: Some("USD".to_string()),
            pocket_id: None,
        };

        service
            .create_transaction(Uuid::new_v4(), req)
            .await
            .unwrap();
        let state = tx_repo.state.lock().unwrap();
        let call = state.create_calls.first().expect("create call");
        assert_eq!(call.amount, Decimal::new(12, 0));
        assert_eq!(call.original_currency, None);
        assert_eq!(call.original_amount, None);
        assert_eq!(call.exchange_rate, None);

        let calls = fx.calls.lock().unwrap();
        assert!(calls.is_empty());
    }

    #[tokio::test]
    async fn get_transactions_rejects_invalid_date_range() {
        let tx_repo = MockTransactionRepo::default();
        let pocket_repo = MockPocketRepo {
            state: Arc::new(Mutex::new(MockPocketState {
                default_pocket: default_pocket(Uuid::new_v4()),
                ..Default::default()
            })),
        };
        let settings_repo = MockSettingsRepo {
            base_currency: "USD".to_string(),
        };
        let fx = MockExchangeRateProvider::default();
        let service = make_transaction_service(tx_repo, pocket_repo, settings_repo, fx);

        let start = DateTime::<Utc>::from_timestamp(1_700_000_100, 0).unwrap();
        let end = DateTime::<Utc>::from_timestamp(1_700_000_000, 0).unwrap();

        let err = service
            .get_transactions(Uuid::new_v4(), Some(start), Some(end), None, None, None, 1, 20)
            .await
            .unwrap_err();
        assert!(
            matches!(err, AppError::ValidationError(msg) if msg == "End date cannot be before start date")
        );
    }

    #[tokio::test]
    async fn get_transactions_clamps_limit_and_page() {
        let mut tx_state = MockTransactionState::default();
        tx_state.total_count = 10;
        let tx_repo = MockTransactionRepo {
            state: Arc::new(Mutex::new(tx_state)),
        };
        let pocket_repo = MockPocketRepo {
            state: Arc::new(Mutex::new(MockPocketState {
                default_pocket: default_pocket(Uuid::new_v4()),
                ..Default::default()
            })),
        };
        let settings_repo = MockSettingsRepo {
            base_currency: "USD".to_string(),
        };
        let fx = MockExchangeRateProvider::default();
        let service = make_transaction_service(tx_repo.clone(), pocket_repo, settings_repo, fx);

        let response = service
            .get_transactions(Uuid::new_v4(), None, None, None, None, None, 0, 1000)
            .await
            .unwrap();
        assert_eq!(response.limit, 100);
        assert_eq!(response.page, 1);
        assert_eq!(response.total_pages, 1);

        let state = tx_repo.state.lock().unwrap();
        let find_args = state.find_args.as_ref().expect("find args");
        assert_eq!(find_args.limit, 100);
        assert_eq!(find_args.offset, 0);
    }

    #[tokio::test]
    async fn get_transactions_passes_category_ids() {
        let mut tx_state = MockTransactionState::default();
        tx_state.total_count = 5;
        let tx_repo = MockTransactionRepo {
            state: Arc::new(Mutex::new(tx_state)),
        };
        let pocket_repo = MockPocketRepo {
            state: Arc::new(Mutex::new(MockPocketState {
                default_pocket: default_pocket(Uuid::new_v4()),
                ..Default::default()
            })),
        };
        let settings_repo = MockSettingsRepo {
            base_currency: "USD".to_string(),
        };
        let fx = MockExchangeRateProvider::default();
        let service = make_transaction_service(tx_repo.clone(), pocket_repo, settings_repo, fx);

        let category_ids = vec![2, 3];
        service
            .get_transactions(
                Uuid::new_v4(),
                None,
                None,
                None,
                None,
                Some(category_ids.clone()),
                1,
                20,
            )
            .await
            .unwrap();

        let state = tx_repo.state.lock().unwrap();
        let find_args = state.find_args.as_ref().expect("find args");
        assert_eq!(find_args.category_ids.as_ref().unwrap(), &category_ids);
        let count_args = state.count_args.as_ref().expect("count args");
        assert_eq!(count_args.category_ids.as_ref().unwrap(), &category_ids);
    }

    #[tokio::test]
    async fn get_spending_analysis_returns_comparison_for_full_month_range() {
        let start = utc_datetime(2025, 3, 1, 0, 0, 0, 0);
        let end = utc_datetime(2025, 3, 31, 23, 59, 59, 999_000_000);
        let previous_start = utc_datetime(2025, 2, 1, 0, 0, 0, 0);
        let previous_end = utc_datetime(2025, 2, 28, 23, 59, 59, 999_999_999);

        let mut tx_state = MockTransactionState::default();
        tx_state.spending_analysis_results = vec![
            SpendingAnalysisResult {
                start_date: start,
                end_date: end,
                categories: vec![
                    CategorySummary {
                        category: "Salary".to_string(),
                        total: Decimal::new(1000, 0),
                        is_income: true,
                        icon: "payments".to_string(),
                    },
                    CategorySummary {
                        category: "Food".to_string(),
                        total: Decimal::new(200, 0),
                        is_income: false,
                        icon: "restaurant".to_string(),
                    },
                ],
            },
            SpendingAnalysisResult {
                start_date: previous_start,
                end_date: previous_end,
                categories: vec![CategorySummary {
                    category: "Food".to_string(),
                    total: Decimal::new(100, 0),
                    is_income: false,
                    icon: "restaurant".to_string(),
                }],
            },
        ];

        let tx_repo = MockTransactionRepo {
            state: Arc::new(Mutex::new(tx_state)),
        };
        let pocket_repo = MockPocketRepo {
            state: Arc::new(Mutex::new(MockPocketState {
                default_pocket: default_pocket(Uuid::new_v4()),
                ..Default::default()
            })),
        };
        let settings_repo = MockSettingsRepo {
            base_currency: "USD".to_string(),
        };
        let fx = MockExchangeRateProvider::default();
        let service = make_transaction_service(tx_repo.clone(), pocket_repo, settings_repo, fx);

        let response = service
            .get_spending_analysis(Uuid::new_v4(), start, end)
            .await
            .unwrap();

        assert_eq!(response.comparison_percentage, Some(Decimal::new(100, 0)));

        let state = tx_repo.state.lock().unwrap();
        assert_eq!(state.spending_analysis_calls.len(), 2);
        assert_eq!(state.spending_analysis_calls[1].start_date, previous_start);
        assert_eq!(state.spending_analysis_calls[1].end_date, previous_end);
    }

    #[tokio::test]
    async fn get_spending_analysis_returns_negative_comparison_when_spend_decreases() {
        let start = utc_datetime(2025, 3, 1, 0, 0, 0, 0);
        let end = utc_datetime(2025, 3, 31, 23, 59, 59, 999_000_000);
        let previous_start = utc_datetime(2025, 2, 1, 0, 0, 0, 0);
        let previous_end = utc_datetime(2025, 2, 28, 23, 59, 59, 999_999_999);

        let mut tx_state = MockTransactionState::default();
        tx_state.spending_analysis_results = vec![
            SpendingAnalysisResult {
                start_date: start,
                end_date: end,
                categories: vec![CategorySummary {
                    category: "Food".to_string(),
                    total: Decimal::new(80, 0),
                    is_income: false,
                    icon: "restaurant".to_string(),
                }],
            },
            SpendingAnalysisResult {
                start_date: previous_start,
                end_date: previous_end,
                categories: vec![CategorySummary {
                    category: "Food".to_string(),
                    total: Decimal::new(100, 0),
                    is_income: false,
                    icon: "restaurant".to_string(),
                }],
            },
        ];

        let tx_repo = MockTransactionRepo {
            state: Arc::new(Mutex::new(tx_state)),
        };
        let pocket_repo = MockPocketRepo {
            state: Arc::new(Mutex::new(MockPocketState {
                default_pocket: default_pocket(Uuid::new_v4()),
                ..Default::default()
            })),
        };
        let settings_repo = MockSettingsRepo {
            base_currency: "USD".to_string(),
        };
        let fx = MockExchangeRateProvider::default();
        let service = make_transaction_service(tx_repo, pocket_repo, settings_repo, fx);

        let response = service
            .get_spending_analysis(Uuid::new_v4(), start, end)
            .await
            .unwrap();

        assert_eq!(response.comparison_percentage, Some(Decimal::new(-20, 0)));
    }

    #[tokio::test]
    async fn get_spending_analysis_returns_null_comparison_for_custom_range() {
        let start = utc_datetime(2025, 3, 2, 0, 0, 0, 0);
        let end = utc_datetime(2025, 3, 31, 23, 59, 59, 999_000_000);

        let mut tx_state = MockTransactionState::default();
        tx_state.spending_analysis_results = vec![SpendingAnalysisResult {
            start_date: start,
            end_date: end,
            categories: vec![CategorySummary {
                category: "Food".to_string(),
                total: Decimal::new(200, 0),
                is_income: false,
                icon: "restaurant".to_string(),
            }],
        }];

        let tx_repo = MockTransactionRepo {
            state: Arc::new(Mutex::new(tx_state)),
        };
        let pocket_repo = MockPocketRepo {
            state: Arc::new(Mutex::new(MockPocketState {
                default_pocket: default_pocket(Uuid::new_v4()),
                ..Default::default()
            })),
        };
        let settings_repo = MockSettingsRepo {
            base_currency: "USD".to_string(),
        };
        let fx = MockExchangeRateProvider::default();
        let service = make_transaction_service(tx_repo.clone(), pocket_repo, settings_repo, fx);

        let response = service
            .get_spending_analysis(Uuid::new_v4(), start, end)
            .await
            .unwrap();

        assert_eq!(response.comparison_percentage, None);

        let state = tx_repo.state.lock().unwrap();
        assert_eq!(state.spending_analysis_calls.len(), 1);
    }

    #[tokio::test]
    async fn transfer_funds_rejects_same_pocket() {
        let tx_repo = MockTransactionRepo::default();
        let pocket_repo = MockPocketRepo {
            state: Arc::new(Mutex::new(MockPocketState {
                default_pocket: default_pocket(Uuid::new_v4()),
                ..Default::default()
            })),
        };
        let settings_repo = MockSettingsRepo {
            base_currency: "USD".to_string(),
        };
        let fx = MockExchangeRateProvider::default();
        let service = make_transaction_service(tx_repo, pocket_repo, settings_repo, fx);

        let pocket_id = Uuid::new_v4();
        let req = TransferRequest {
            amount: Decimal::new(5, 0),
            source_pocket_id: pocket_id,
            destination_pocket_id: pocket_id,
            description: None,
        };

        let err = service
            .transfer_funds(Uuid::new_v4(), req)
            .await
            .unwrap_err();
        assert!(
            matches!(err, AppError::ValidationError(msg) if msg == "Cannot transfer to the same pocket")
        );
    }

    #[tokio::test]
    async fn transfer_funds_rejects_insufficient_balance() {
        let mut tx_state = MockTransactionState::default();
        tx_state.get_pocket_balance = Decimal::new(3, 0);
        let tx_repo = MockTransactionRepo {
            state: Arc::new(Mutex::new(tx_state)),
        };

        let source_id = Uuid::new_v4();
        let dest_id = Uuid::new_v4();
        let mut pocket_state = MockPocketState {
            default_pocket: default_pocket(source_id),
            ..Default::default()
        };
        pocket_state
            .pockets
            .insert(source_id, default_pocket(source_id));
        pocket_state
            .pockets
            .insert(dest_id, default_pocket(dest_id));
        let pocket_repo = MockPocketRepo {
            state: Arc::new(Mutex::new(pocket_state)),
        };
        let settings_repo = MockSettingsRepo {
            base_currency: "USD".to_string(),
        };
        let fx = MockExchangeRateProvider::default();
        let service = make_transaction_service(tx_repo, pocket_repo, settings_repo, fx);

        let req = TransferRequest {
            amount: Decimal::new(5, 0),
            source_pocket_id: source_id,
            destination_pocket_id: dest_id,
            description: None,
        };

        let err = service
            .transfer_funds(Uuid::new_v4(), req)
            .await
            .unwrap_err();
        assert!(
            matches!(err, AppError::ValidationError(msg) if msg == "Insufficient funds in source pocket")
        );
    }

    #[tokio::test]
    async fn transfer_funds_creates_out_and_in_transactions() {
        let mut tx_state = MockTransactionState::default();
        tx_state.get_pocket_balance = Decimal::new(10, 0);
        tx_state.categories.insert(
            "Transfer Out".to_string(),
            CategoryStub {
                id: 101,
                is_income: false,
                icon: "out".to_string(),
                exclude_from_analysis: false,
            },
        );
        tx_state.categories.insert(
            "Transfer In".to_string(),
            CategoryStub {
                id: 102,
                is_income: true,
                icon: "in".to_string(),
                exclude_from_analysis: false,
            },
        );
        let tx_repo = MockTransactionRepo {
            state: Arc::new(Mutex::new(tx_state)),
        };

        let source_id = Uuid::new_v4();
        let dest_id = Uuid::new_v4();
        let mut pocket_state = MockPocketState {
            default_pocket: default_pocket(source_id),
            ..Default::default()
        };
        pocket_state
            .pockets
            .insert(source_id, default_pocket(source_id));
        pocket_state
            .pockets
            .insert(dest_id, default_pocket(dest_id));
        let pocket_repo = MockPocketRepo {
            state: Arc::new(Mutex::new(pocket_state)),
        };
        let settings_repo = MockSettingsRepo {
            base_currency: "USD".to_string(),
        };
        let fx = MockExchangeRateProvider::default();
        let service = make_transaction_service(tx_repo.clone(), pocket_repo, settings_repo, fx);

        let req = TransferRequest {
            amount: Decimal::new(5, 0),
            source_pocket_id: source_id,
            destination_pocket_id: dest_id,
            description: None,
        };

        service.transfer_funds(Uuid::new_v4(), req).await.unwrap();

        let state = tx_repo.state.lock().unwrap();
        assert_eq!(state.create_calls.len(), 2);
        assert_eq!(state.create_calls[0].category_id, 101);
        assert_eq!(state.create_calls[0].pocket_id, source_id);
        assert_eq!(
            state.create_calls[0].description,
            Some("Transfer Out".to_string())
        );
        assert_eq!(state.create_calls[1].category_id, 102);
        assert_eq!(state.create_calls[1].pocket_id, dest_id);
        assert_eq!(
            state.create_calls[1].description,
            Some("Transfer In".to_string())
        );
    }
}
