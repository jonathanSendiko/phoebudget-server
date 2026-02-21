use axum::{
    Json,
    extract::{Path, Query, State},
};
use uuid::Uuid;

use crate::AppState;
use crate::auth::UserId;
use crate::error::AppError;
use crate::i18n;
use crate::response::ApiResponse;
use crate::schemas::{
    Category, CreateTransaction, DateRangeParams, PaginatedTransactions, TransactionDetail,
    TransactionId, TransactionQueryParams, TransferRequest, UpdateTransaction,
};

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
        Some(i18n::localize_message("Transaction saved")),
    )))
}

pub async fn update_transaction(
    State(state): State<AppState>,
    user_id: UserId,
    path: Path<Uuid>,
    Json(payload): Json<UpdateTransaction>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    state
        .transaction_service()
        .update_transaction(path.0, user_id.0, payload)
        .await?;
    Ok(Json(ApiResponse::success(
        i18n::localize_message("Transaction updated"),
        None,
    )))
}

pub async fn delete_transaction(
    State(state): State<AppState>,
    user_id: UserId,
    path: Path<Uuid>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    state
        .transaction_service()
        .delete_transaction(path.0, user_id.0)
        .await?;
    Ok(Json(ApiResponse::success(
        i18n::localize_message("Transaction deleted"),
        None,
    )))
}

pub async fn restore_transaction(
    State(state): State<AppState>,
    user_id: UserId,
    path: Path<Uuid>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    state
        .transaction_service()
        .restore_transaction(path.0, user_id.0)
        .await?;
    Ok(Json(ApiResponse::success(
        i18n::localize_message("Transaction restored"),
        None,
    )))
}

pub async fn get_transactions(
    State(state): State<AppState>,
    user_id: UserId,
    Query(params): Query<TransactionQueryParams>,
) -> Result<Json<ApiResponse<PaginatedTransactions>>, AppError> {
    let category_ids = params.category_filter();
    let result = state
        .transaction_service()
        .get_transactions(
            user_id.0,
            params.start_date,
            params.end_date,
            params.pocket_id,
            params.category_id,
            params.search,
            category_ids,
            params.page,
            params.limit,
        )
        .await?;
    Ok(Json(ApiResponse::success(result, None)))
}

pub async fn get_transaction(
    State(state): State<AppState>,
    user_id: UserId,
    path: Path<Uuid>,
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
) -> Result<Json<ApiResponse<crate::schemas::SpendingAnalysisResponse>>, AppError> {
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
        i18n::localize_message("Transfer successful"),
        None,
    )))
}
