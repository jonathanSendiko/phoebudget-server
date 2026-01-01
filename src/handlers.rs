use axum::{
    Json,
    extract::{Query, State},
};

use crate::AppState;
use crate::auth::UserId;
use crate::error::AppError;
use crate::repository::{
    PortfolioRepository, SettingsRepository, TransactionRepository, UserRepository,
};
use crate::response::ApiResponse;
use crate::schemas::{
    AuthResponse, Category, CreatePortfolioItem, CreateTransaction, DateRangeParams,
    FinancialHealth, LoginRequest, RegisterRequest, SpendingAnalysisResponse, Transaction,
    TransactionDetail, TransactionId, UpdateCurrency, UpdateInvestment, UpdateTransaction,
    UserProfile,
};
use crate::services::{AuthService, FinanceService, TransactionService};

// --- Auth Handlers ---

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<ApiResponse<AuthResponse>>, AppError> {
    let user_repo = UserRepository::new(state.db.clone());
    let settings_repo = SettingsRepository::new(state.db.clone());
    let auth_service = AuthService::new(user_repo, settings_repo);

    let response = auth_service.register(payload).await?;

    Ok(Json(ApiResponse::success(response, None)))
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<ApiResponse<AuthResponse>>, AppError> {
    let user_repo = UserRepository::new(state.db.clone());
    let settings_repo = SettingsRepository::new(state.db.clone());
    let auth_service = AuthService::new(user_repo, settings_repo);

    let response = auth_service.login(payload).await?;

    Ok(Json(ApiResponse::success(response, None)))
}

pub async fn get_profile(
    State(state): State<AppState>,
    user_id: UserId,
) -> Result<Json<ApiResponse<UserProfile>>, AppError> {
    let user_repo = UserRepository::new(state.db.clone());
    let settings_repo = SettingsRepository::new(state.db.clone());
    let auth_service = AuthService::new(user_repo, settings_repo);

    let profile = auth_service.get_profile(user_id.0).await?;

    Ok(Json(ApiResponse::success(profile, None)))
}

// --- Protected Handlers ---

pub async fn create_transaction(
    State(state): State<AppState>,
    user_id: UserId,
    Json(payload): Json<CreateTransaction>,
) -> Result<Json<ApiResponse<TransactionId>>, AppError> {
    let transaction_repo = TransactionRepository::new(state.db.clone());
    let settings_repo = SettingsRepository::new(state.db.clone());
    let service = TransactionService::new(transaction_repo, settings_repo);

    let id = service.create_transaction(user_id.0, payload).await?;

    Ok(Json(ApiResponse::success(
        TransactionId { id },
        Some("Transaction saved".to_string()),
    )))
}

pub async fn update_transaction(
    State(state): State<AppState>,
    user_id: UserId,
    path: axum::extract::Path<uuid::Uuid>,
    Json(payload): Json<UpdateTransaction>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    let transaction_repo = TransactionRepository::new(state.db.clone());
    let settings_repo = SettingsRepository::new(state.db.clone());
    let service = TransactionService::new(transaction_repo, settings_repo);

    service
        .update_transaction(path.0, user_id.0, payload)
        .await?;

    Ok(Json(ApiResponse::success(
        "Transaction updated".to_string(),
        None,
    )))
}

pub async fn delete_transaction(
    State(state): State<AppState>,
    user_id: UserId,
    path: axum::extract::Path<uuid::Uuid>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    let transaction_repo = TransactionRepository::new(state.db.clone());
    let settings_repo = SettingsRepository::new(state.db.clone());
    let service = TransactionService::new(transaction_repo, settings_repo);

    service.delete_transaction(path.0, user_id.0).await?;

    Ok(Json(ApiResponse::success(
        "Transaction deleted".to_string(),
        None,
    )))
}

pub async fn get_transactions(
    State(state): State<AppState>,
    user_id: UserId,
    Query(params): Query<DateRangeParams>,
) -> Result<Json<ApiResponse<Vec<Transaction>>>, AppError> {
    let transaction_repo = TransactionRepository::new(state.db.clone());
    let settings_repo = SettingsRepository::new(state.db.clone());
    let service = TransactionService::new(transaction_repo, settings_repo);

    let rows = service
        .get_transactions(user_id.0, params.start_date, params.end_date)
        .await?;

    Ok(Json(ApiResponse::success(rows, None)))
}

pub async fn get_transaction(
    State(state): State<AppState>,
    user_id: UserId,
    path: axum::extract::Path<uuid::Uuid>,
) -> Result<Json<ApiResponse<TransactionDetail>>, AppError> {
    let transaction_repo = TransactionRepository::new(state.db.clone());
    let settings_repo = SettingsRepository::new(state.db.clone());
    let service = TransactionService::new(transaction_repo, settings_repo);

    let transaction = service.get_transaction(user_id.0, path.0).await?;

    Ok(Json(ApiResponse::success(transaction, None)))
}

pub async fn get_spending_analysis(
    State(state): State<AppState>,
    user_id: UserId,
    Query(params): Query<DateRangeParams>,
) -> Result<Json<ApiResponse<SpendingAnalysisResponse>>, AppError> {
    let transaction_repo = TransactionRepository::new(state.db.clone());
    let settings_repo = SettingsRepository::new(state.db.clone());
    let service = TransactionService::new(transaction_repo, settings_repo);

    let rows = service
        .get_spending_analysis(user_id.0, params.start_date, params.end_date)
        .await?;

    Ok(Json(ApiResponse::success(rows, None)))
}

pub async fn get_categories(
    state: State<AppState>,
    _user_id: UserId,
) -> Result<Json<ApiResponse<Vec<Category>>>, AppError> {
    let transaction_repo = TransactionRepository::new(state.db.clone());
    let settings_repo = SettingsRepository::new(state.db.clone());
    let service = TransactionService::new(transaction_repo, settings_repo);

    let categories = service.get_categories().await?;

    Ok(Json(ApiResponse::success(categories, None)))
}

pub async fn get_financial_health(
    State(state): State<AppState>,
    user_id: UserId,
) -> Result<Json<ApiResponse<FinancialHealth>>, AppError> {
    let portfolio_repo = PortfolioRepository::new(state.db.clone());
    let transaction_repo = TransactionRepository::new(state.db.clone());
    let settings_repo = SettingsRepository::new(state.db.clone());

    let service = FinanceService::new(
        portfolio_repo,
        transaction_repo,
        settings_repo,
        state.price_cache.clone(),
        state.exchange_rate_cache.clone(),
    );

    let health = service.get_financial_health(user_id.0).await?;

    Ok(Json(ApiResponse::success(health, None)))
}

pub async fn refresh_portfolio(
    State(state): State<AppState>,
    user_id: UserId,
) -> Result<Json<ApiResponse<String>>, AppError> {
    let portfolio_repo = PortfolioRepository::new(state.db.clone());
    let transaction_repo = TransactionRepository::new(state.db.clone());
    let settings_repo = SettingsRepository::new(state.db.clone());

    let service = FinanceService::new(
        portfolio_repo,
        transaction_repo,
        settings_repo,
        state.price_cache.clone(),
        state.exchange_rate_cache.clone(),
    );

    let updated_count = service.refresh_portfolio(user_id.0).await?;

    Ok(Json(ApiResponse::success(
        format!("Updated {} assets", updated_count),
        None,
    )))
}

pub async fn add_investment(
    State(state): State<AppState>,
    user_id: UserId,
    Json(payload): Json<CreatePortfolioItem>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    let portfolio_repo = PortfolioRepository::new(state.db.clone());
    let transaction_repo = TransactionRepository::new(state.db.clone());
    let settings_repo = SettingsRepository::new(state.db.clone());

    let service = FinanceService::new(
        portfolio_repo,
        transaction_repo,
        settings_repo,
        state.price_cache.clone(),
        state.exchange_rate_cache.clone(),
    );
    let ticker = payload.ticker.clone();

    service.add_investment(user_id.0, payload).await?;

    Ok(Json(ApiResponse::success(
        format!("Added {} to portfolio", ticker),
        None,
    )))
}

pub async fn get_portfolio(
    State(state): State<AppState>,
    user_id: UserId,
) -> Result<Json<ApiResponse<crate::schemas::PortfolioResponse>>, AppError> {
    let portfolio_repo = PortfolioRepository::new(state.db.clone());
    let transaction_repo = TransactionRepository::new(state.db.clone());
    let settings_repo = SettingsRepository::new(state.db.clone());

    let service = FinanceService::new(
        portfolio_repo,
        transaction_repo,
        settings_repo,
        state.price_cache.clone(),
        state.exchange_rate_cache.clone(),
    );
    let summary = service.get_portfolio_list(user_id.0).await?;

    Ok(Json(ApiResponse::success(summary, None)))
}

pub async fn update_base_currency(
    State(state): State<AppState>,
    user_id: UserId,
    Json(payload): Json<UpdateCurrency>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    let portfolio_repo = PortfolioRepository::new(state.db.clone());
    let transaction_repo = TransactionRepository::new(state.db.clone());
    let settings_repo = SettingsRepository::new(state.db.clone());

    let service = FinanceService::new(
        portfolio_repo,
        transaction_repo,
        settings_repo,
        state.price_cache.clone(),
        state.exchange_rate_cache.clone(),
    );

    service
        .update_base_currency(user_id.0, payload.base_currency)
        .await?;

    Ok(Json(ApiResponse::success(
        "Base currency updated".to_string(),
        None,
    )))
}

pub async fn remove_investment(
    State(state): State<AppState>,
    user_id: UserId,
    path: axum::extract::Path<String>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    let portfolio_repo = PortfolioRepository::new(state.db.clone());
    let transaction_repo = TransactionRepository::new(state.db.clone());
    let settings_repo = SettingsRepository::new(state.db.clone());

    let service = FinanceService::new(
        portfolio_repo,
        transaction_repo,
        settings_repo,
        state.price_cache.clone(),
        state.exchange_rate_cache.clone(),
    );
    service.remove_investment(user_id.0, path.0).await?;

    Ok(Json(ApiResponse::success(
        "Investment removed".to_string(),
        None,
    )))
}

pub async fn update_investment(
    State(state): State<AppState>,
    user_id: UserId,
    path: axum::extract::Path<String>,
    Json(payload): Json<UpdateInvestment>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    let portfolio_repo = PortfolioRepository::new(state.db.clone());
    let transaction_repo = TransactionRepository::new(state.db.clone());
    let settings_repo = SettingsRepository::new(state.db.clone());

    let service = FinanceService::new(
        portfolio_repo,
        transaction_repo,
        settings_repo,
        state.price_cache.clone(),
        state.exchange_rate_cache.clone(),
    );
    service
        .update_investment(user_id.0, path.0, payload)
        .await?;

    Ok(Json(ApiResponse::success(
        "Investment updated".to_string(),
        None,
    )))
}

pub async fn get_available_currencies(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<String>>>, AppError> {
    let settings_repo = SettingsRepository::new(state.db.clone());
    let currencies = settings_repo.get_available_currencies().await?;

    Ok(Json(ApiResponse::success(currencies, None)))
}

pub async fn get_assets(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<crate::schemas::Asset>>>, AppError> {
    let portfolio_repo = PortfolioRepository::new(state.db.clone());
    let assets = portfolio_repo.get_all_assets().await?;

    Ok(Json(ApiResponse::success(assets, None)))
}
