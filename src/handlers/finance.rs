use axum::{Json, extract::State, extract::Query};

use crate::AppState;
use crate::auth::UserId;
use crate::error::AppError;
use crate::response::ApiResponse;
use crate::schemas::{
    CreatePortfolioItem, DateRangeParams, FinancialHealth, NetWorthHistoryResponse, UpdateCurrency,
    UpdateInvestment,
};

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

pub async fn get_net_worth_history(
    State(state): State<AppState>,
    user_id: UserId,
    Query(params): Query<DateRangeParams>,
) -> Result<Json<ApiResponse<NetWorthHistoryResponse>>, AppError> {
    let history = state
        .finance_service()
        .get_net_worth_history(user_id.0, params.start_date, params.end_date)
        .await?;
    Ok(Json(ApiResponse::success(history, None)))
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
