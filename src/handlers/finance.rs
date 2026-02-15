use axum::{Json, extract::State};

use crate::AppState;
use crate::auth::UserId;
use crate::error::AppError;
use crate::i18n;
use crate::response::ApiResponse;
use crate::schemas::{CreatePortfolioItem, FinancialHealth, UpdateCurrency, UpdateInvestment};

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
        i18n::localize_message(&format!("Updated {} assets", updated_count)),
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
        i18n::localize_message(&format!("Added {} to portfolio", ticker)),
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
        i18n::localize_message("Base currency updated"),
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
        i18n::localize_message("Investment removed"),
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
        i18n::localize_message("Investment updated"),
        None,
    )))
}
