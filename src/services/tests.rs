use super::{
    ExchangeRateProvider, PocketRepo, SettingsRepo, SubscriptionService, TransactionRepo,
    TransactionService, UserSubscriptionService,
};
use async_trait::async_trait;
use crate::error::AppError;
use crate::schemas::{
    Category, CategorySummary, CreateTransaction, Pocket, SubscriptionRow, Transaction,
};
use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

fn sample_subscription(plan: &str) -> SubscriptionRow {
    SubscriptionRow {
        id: Uuid::new_v4(),
        user_id: Uuid::new_v4(),
        plan: plan.to_string(),
        status: "active".to_string(),
        started_at: DateTime::<Utc>::from_timestamp(1_700_000_000, 0).unwrap(),
        expires_at: None,
        payment_provider: None,
        external_subscription_id: None,
    }
}

#[test]
fn compute_limits_free_defaults() {
    let limits = SubscriptionService::compute_limits(&None);
    assert_eq!(limits.max_investments, Some(3));
    assert_eq!(limits.max_pockets, Some(2));
    assert_eq!(limits.history_days, Some(90));
    assert!(!limits.multi_currency);
    assert!(!limits.pocket_transfers);
    assert!(!limits.advanced_analytics);
    assert!(!limits.data_export);
}

#[test]
fn compute_limits_premium_unlimited() {
    let sub = Some(sample_subscription("premium"));
    let limits = SubscriptionService::compute_limits(&sub);
    assert_eq!(limits.max_investments, None);
    assert_eq!(limits.max_pockets, None);
    assert_eq!(limits.history_days, None);
    assert!(limits.multi_currency);
    assert!(limits.pocket_transfers);
    assert!(limits.advanced_analytics);
    assert!(limits.data_export);
}

#[test]
fn compute_limits_lifetime_unlimited() {
    let sub = Some(sample_subscription("lifetime"));
    let limits = SubscriptionService::compute_limits(&sub);
    assert_eq!(limits.max_investments, None);
    assert_eq!(limits.max_pockets, None);
    assert_eq!(limits.history_days, None);
    assert!(limits.multi_currency);
    assert!(limits.pocket_transfers);
    assert!(limits.advanced_analytics);
    assert!(limits.data_export);
}

#[test]
fn monthly_billing_same_month_when_before_day() {
    let reference = NaiveDate::from_ymd_opt(2026, 2, 2).unwrap();
    let next = UserSubscriptionService::calculate_next_charge_date(
        "monthly",
        15,
        None,
        reference,
        false,
    );
    assert_eq!(next, NaiveDate::from_ymd_opt(2026, 2, 15).unwrap());
}

#[test]
fn monthly_billing_next_month_when_after_day() {
    let reference = NaiveDate::from_ymd_opt(2026, 2, 20).unwrap();
    let next = UserSubscriptionService::calculate_next_charge_date(
        "monthly",
        15,
        None,
        reference,
        false,
    );
    assert_eq!(next, NaiveDate::from_ymd_opt(2026, 3, 15).unwrap());
}

#[test]
fn monthly_retry_always_next_month() {
    let reference = NaiveDate::from_ymd_opt(2026, 12, 15).unwrap();
    let next = UserSubscriptionService::calculate_next_charge_date(
        "monthly",
        15,
        None,
        reference,
        true,
    );
    assert_eq!(next, NaiveDate::from_ymd_opt(2027, 1, 15).unwrap());
}

#[test]
fn annual_billing_this_year_when_before_date() {
    let reference = NaiveDate::from_ymd_opt(2026, 2, 2).unwrap();
    let next = UserSubscriptionService::calculate_next_charge_date(
        "annually",
        15,
        Some(3),
        reference,
        false,
    );
    assert_eq!(next, NaiveDate::from_ymd_opt(2026, 3, 15).unwrap());
}

#[test]
fn annual_billing_next_year_when_after_date() {
    let reference = NaiveDate::from_ymd_opt(2026, 12, 20).unwrap();
    let next = UserSubscriptionService::calculate_next_charge_date(
        "annually",
        15,
        Some(3),
        reference,
        false,
    );
    assert_eq!(next, NaiveDate::from_ymd_opt(2027, 3, 15).unwrap());
}

#[test]
fn annual_retry_moves_to_next_year() {
    let reference = NaiveDate::from_ymd_opt(2026, 3, 15).unwrap();
    let next = UserSubscriptionService::calculate_next_charge_date(
        "annually",
        15,
        Some(3),
        reference,
        true,
    );
    assert_eq!(next, NaiveDate::from_ymd_opt(2027, 3, 15).unwrap());
}

#[test]
fn get_valid_date_handles_non_leap_february() {
    let date = UserSubscriptionService::get_valid_date(2025, 2, 31);
    assert_eq!(date, NaiveDate::from_ymd_opt(2025, 2, 28).unwrap());
}

#[test]
fn get_valid_date_handles_leap_february() {
    let date = UserSubscriptionService::get_valid_date(2024, 2, 31);
    assert_eq!(date, NaiveDate::from_ymd_opt(2024, 2, 29).unwrap());
}

#[derive(Clone, Default)]
struct MockTransactionRepo {
    state: Arc<Mutex<MockTransactionState>>,
}

struct MockTransactionState {
    create_calls: Vec<CreateCall>,
    find_args: Option<FindArgs>,
    count_args: Option<CountArgs>,
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
    ) -> Result<i64, AppError> {
        let mut state = self.state.lock().unwrap();
        state.count_args = Some(CountArgs {
            user_id,
            start_date,
            end_date,
            pocket_id,
            search,
        });
        Ok(state.total_count)
    }

    async fn get_spending_analysis(
        &self,
        _user_id: Uuid,
        _start_date: DateTime<Utc>,
        _end_date: DateTime<Utc>,
    ) -> Result<Vec<CategorySummary>, AppError> {
        Ok(Vec::new())
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

fn make_transaction_service(
    tx_repo: MockTransactionRepo,
    pocket_repo: MockPocketRepo,
    settings_repo: MockSettingsRepo,
    fx: MockExchangeRateProvider,
) -> TransactionService<MockTransactionRepo, MockPocketRepo, MockSettingsRepo, MockExchangeRateProvider>
{
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

    let id = service.create_transaction(Uuid::new_v4(), req).await.unwrap();
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

    service.create_transaction(Uuid::new_v4(), req).await.unwrap();
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
    let service = make_transaction_service(tx_repo.clone(), pocket_repo, settings_repo, fx.clone());

    let req = CreateTransaction {
        amount: Decimal::new(12, 0),
        description: Some("Coffee".to_string()),
        category_id: 1,
        occurred_at: DateTime::<Utc>::from_timestamp(1_700_000_000, 0).unwrap(),
        currency_code: Some("USD".to_string()),
        pocket_id: None,
    };

    service.create_transaction(Uuid::new_v4(), req).await.unwrap();
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
        .get_transactions(Uuid::new_v4(), Some(start), Some(end), None, None, 1, 20)
        .await
        .unwrap_err();
    assert!(matches!(err, AppError::ValidationError(msg) if msg == "End date cannot be before start date"));
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
        .get_transactions(Uuid::new_v4(), None, None, None, None, 0, 1000)
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
    let req = crate::schemas::TransferRequest {
        amount: Decimal::new(5, 0),
        source_pocket_id: pocket_id,
        destination_pocket_id: pocket_id,
        description: None,
    };

    let err = service
        .transfer_funds(Uuid::new_v4(), req)
        .await
        .unwrap_err();
    assert!(matches!(err, AppError::ValidationError(msg) if msg == "Cannot transfer to the same pocket"));
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
    pocket_state.pockets.insert(source_id, default_pocket(source_id));
    pocket_state.pockets.insert(dest_id, default_pocket(dest_id));
    let pocket_repo = MockPocketRepo {
        state: Arc::new(Mutex::new(pocket_state)),
    };
    let settings_repo = MockSettingsRepo {
        base_currency: "USD".to_string(),
    };
    let fx = MockExchangeRateProvider::default();
    let service = make_transaction_service(tx_repo, pocket_repo, settings_repo, fx);

    let req = crate::schemas::TransferRequest {
        amount: Decimal::new(5, 0),
        source_pocket_id: source_id,
        destination_pocket_id: dest_id,
        description: None,
    };

    let err = service
        .transfer_funds(Uuid::new_v4(), req)
        .await
        .unwrap_err();
    assert!(matches!(err, AppError::ValidationError(msg) if msg == "Insufficient funds in source pocket"));
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
    pocket_state.pockets.insert(source_id, default_pocket(source_id));
    pocket_state.pockets.insert(dest_id, default_pocket(dest_id));
    let pocket_repo = MockPocketRepo {
        state: Arc::new(Mutex::new(pocket_state)),
    };
    let settings_repo = MockSettingsRepo {
        base_currency: "USD".to_string(),
    };
    let fx = MockExchangeRateProvider::default();
    let service = make_transaction_service(tx_repo.clone(), pocket_repo, settings_repo, fx);

    let req = crate::schemas::TransferRequest {
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
    assert_eq!(state.create_calls[0].description, Some("Transfer Out".to_string()));
    assert_eq!(state.create_calls[1].category_id, 102);
    assert_eq!(state.create_calls[1].pocket_id, dest_id);
    assert_eq!(state.create_calls[1].description, Some("Transfer In".to_string()));
}
