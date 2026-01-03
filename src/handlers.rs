use axum::{
    Json,
    extract::{Query, State},
};

use crate::AppState;
use crate::auth::UserId;
use crate::error::AppError;
use crate::repository::{PortfolioRepository, SettingsRepository};
use crate::response::ApiResponse;
use crate::schemas::{
    AuthResponse, Category, CreatePocket, CreatePortfolioItem, CreateTransaction, DateRangeParams,
    FinancialHealth, LoginRequest, PaginatedTransactions, Pocket, PocketId, RefreshTokenRequest,
    RegisterRequest, SpendingAnalysisResponse, TransactionDetail, TransactionId,
    TransactionQueryParams, TransferRequest, UpdateCurrency, UpdateInvestment, UpdatePocket,
    UpdateTransaction, UserProfile,
};

// --- Auth Handlers ---

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<ApiResponse<AuthResponse>>, AppError> {
    let response = state.auth_service().register(payload).await?;
    Ok(Json(ApiResponse::success(response, None)))
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<ApiResponse<AuthResponse>>, AppError> {
    let response = state.auth_service().login(payload).await?;
    Ok(Json(ApiResponse::success(response, None)))
}

pub async fn refresh_token(
    State(state): State<AppState>,
    Json(payload): Json<RefreshTokenRequest>,
) -> Result<Json<ApiResponse<AuthResponse>>, AppError> {
    let response = state
        .auth_service()
        .refresh_access(&payload.refresh_token)
        .await?;
    Ok(Json(ApiResponse::success(response, None)))
}

pub async fn get_profile(
    State(state): State<AppState>,
    user_id: UserId,
) -> Result<Json<ApiResponse<UserProfile>>, AppError> {
    let profile = state.auth_service().get_profile(user_id.0).await?;
    Ok(Json(ApiResponse::success(profile, None)))
}

// --- Transaction Handlers ---

pub async fn create_transaction(
    State(state): State<AppState>,
    user_id: UserId,
    Json(payload): Json<CreateTransaction>,
) -> Result<Json<ApiResponse<TransactionId>>, AppError> {
    let id = state
        .transaction_service()
        .create_transaction(user_id.0, payload)
        .await?;
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
    state
        .transaction_service()
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
    state
        .transaction_service()
        .delete_transaction(path.0, user_id.0)
        .await?;
    Ok(Json(ApiResponse::success(
        "Transaction deleted".to_string(),
        None,
    )))
}

pub async fn restore_transaction(
    State(state): State<AppState>,
    user_id: UserId,
    path: axum::extract::Path<uuid::Uuid>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    state
        .transaction_service()
        .restore_transaction(path.0, user_id.0)
        .await?;
    Ok(Json(ApiResponse::success(
        "Transaction restored".to_string(),
        None,
    )))
}

pub async fn get_transactions(
    State(state): State<AppState>,
    user_id: UserId,
    Query(params): Query<TransactionQueryParams>,
) -> Result<Json<ApiResponse<PaginatedTransactions>>, AppError> {
    let result = state
        .transaction_service()
        .get_transactions(
            user_id.0,
            params.start_date,
            params.end_date,
            params.pocket_id,
            params.page,
            params.limit,
        )
        .await?;
    Ok(Json(ApiResponse::success(result, None)))
}

pub async fn get_transaction(
    State(state): State<AppState>,
    user_id: UserId,
    path: axum::extract::Path<uuid::Uuid>,
) -> Result<Json<ApiResponse<TransactionDetail>>, AppError> {
    let transaction = state
        .transaction_service()
        .get_transaction(user_id.0, path.0)
        .await?;
    Ok(Json(ApiResponse::success(transaction, None)))
}

pub async fn get_spending_analysis(
    State(state): State<AppState>,
    user_id: UserId,
    Query(params): Query<DateRangeParams>,
) -> Result<Json<ApiResponse<SpendingAnalysisResponse>>, AppError> {
    let rows = state
        .transaction_service()
        .get_spending_analysis(user_id.0, params.start_date, params.end_date)
        .await?;
    Ok(Json(ApiResponse::success(rows, None)))
}

pub async fn get_categories(
    State(state): State<AppState>,
    _user_id: UserId,
) -> Result<Json<ApiResponse<Vec<Category>>>, AppError> {
    let categories = state.transaction_service().get_categories().await?;
    Ok(Json(ApiResponse::success(categories, None)))
}

// --- Finance Handlers ---

pub async fn get_financial_health(
    State(state): State<AppState>,
    user_id: UserId,
) -> Result<Json<ApiResponse<FinancialHealth>>, AppError> {
    let health = state
        .finance_service()
        .get_financial_health(user_id.0)
        .await?;
    Ok(Json(ApiResponse::success(health, None)))
}

pub async fn refresh_portfolio(
    State(state): State<AppState>,
    user_id: UserId,
) -> Result<Json<ApiResponse<String>>, AppError> {
    let updated_count = state.finance_service().refresh_portfolio(user_id.0).await?;
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
    let ticker = payload.ticker.clone();
    state
        .finance_service()
        .add_investment(user_id.0, payload)
        .await?;
    Ok(Json(ApiResponse::success(
        format!("Added {} to portfolio", ticker),
        None,
    )))
}

pub async fn get_portfolio(
    State(state): State<AppState>,
    user_id: UserId,
) -> Result<Json<ApiResponse<crate::schemas::PortfolioResponse>>, AppError> {
    let summary = state
        .finance_service()
        .get_portfolio_list(user_id.0)
        .await?;
    Ok(Json(ApiResponse::success(summary, None)))
}

pub async fn update_base_currency(
    State(state): State<AppState>,
    user_id: UserId,
    Json(payload): Json<UpdateCurrency>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    state
        .finance_service()
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
    state
        .finance_service()
        .remove_investment(user_id.0, path.0)
        .await?;
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
    state
        .finance_service()
        .update_investment(user_id.0, path.0, payload)
        .await?;
    Ok(Json(ApiResponse::success(
        "Investment updated".to_string(),
        None,
    )))
}

// --- Settings & Assets Handlers ---

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

// --- Pocket Handlers ---

pub async fn create_pocket(
    State(state): State<AppState>,
    user_id: UserId,
    Json(payload): Json<CreatePocket>,
) -> Result<Json<ApiResponse<PocketId>>, AppError> {
    let id = state
        .pocket_service()
        .create_pocket(user_id.0, payload)
        .await?;
    Ok(Json(ApiResponse::success(
        PocketId { id },
        Some("Pocket created".to_string()),
    )))
}

pub async fn get_pockets(
    State(state): State<AppState>,
    user_id: UserId,
) -> Result<Json<ApiResponse<Vec<Pocket>>>, AppError> {
    let pockets = state.pocket_service().get_pockets(user_id.0).await?;
    Ok(Json(ApiResponse::success(pockets, None)))
}

pub async fn get_pocket(
    State(state): State<AppState>,
    user_id: UserId,
    path: axum::extract::Path<uuid::Uuid>,
) -> Result<Json<ApiResponse<Pocket>>, AppError> {
    let pocket = state.pocket_service().get_pocket(path.0, user_id.0).await?;
    Ok(Json(ApiResponse::success(pocket, None)))
}

pub async fn update_pocket(
    State(state): State<AppState>,
    user_id: UserId,
    path: axum::extract::Path<uuid::Uuid>,
    Json(payload): Json<UpdatePocket>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    state
        .pocket_service()
        .update_pocket(path.0, user_id.0, payload)
        .await?;
    Ok(Json(ApiResponse::success(
        "Pocket updated".to_string(),
        None,
    )))
}

pub async fn delete_pocket(
    State(state): State<AppState>,
    user_id: UserId,
    path: axum::extract::Path<uuid::Uuid>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    state
        .pocket_service()
        .delete_pocket(path.0, user_id.0)
        .await?;
    Ok(Json(ApiResponse::success(
        "Pocket deleted".to_string(),
        None,
    )))
}

pub async fn transfer_funds(
    State(state): State<AppState>,
    user_id: UserId,
    Json(payload): Json<TransferRequest>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    state
        .transaction_service()
        .transfer_funds(user_id.0, payload)
        .await?;
    Ok(Json(ApiResponse::success(
        "Transfer successful".to_string(),
        None,
    )))
}
