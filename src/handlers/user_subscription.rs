use axum::{
    Json,
    extract::{Path, State},
};
use uuid::Uuid;

use crate::AppState;
use crate::auth::UserId;
use crate::error::AppError;
use crate::response::ApiResponse;
use crate::schemas::{
    CreateUserSubscription, UpdateUserSubscription, UserSubscriptionDetail, UserSubscriptionSummary,
};

pub async fn create_user_subscription(
    State(state): State<AppState>,
    user_id: UserId,
    Json(payload): Json<CreateUserSubscription>,
) -> Result<Json<ApiResponse<crate::schemas::UserSubscriptionId>>, AppError> {
    let id = state
        .user_subscription_service()
        .create_subscription(user_id.0, payload)
        .await?;
    Ok(Json(ApiResponse::success(
        crate::schemas::UserSubscriptionId { id },
        None,
    )))
}

pub async fn get_user_subscriptions(
    State(state): State<AppState>,
    user_id: UserId,
) -> Result<Json<ApiResponse<Vec<UserSubscriptionSummary>>>, AppError> {
    let subs = state
        .user_subscription_service()
        .get_subscriptions(user_id.0)
        .await?;
    Ok(Json(ApiResponse::success(subs, None)))
}

pub async fn get_user_subscription(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    user_id: UserId,
) -> Result<Json<ApiResponse<UserSubscriptionDetail>>, AppError> {
    let sub = state
        .user_subscription_service()
        .get_subscription(id, user_id.0)
        .await?;
    Ok(Json(ApiResponse::success(sub, None)))
}

pub async fn update_user_subscription(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    user_id: UserId,
    Json(payload): Json<UpdateUserSubscription>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    state
        .user_subscription_service()
        .update_subscription(id, user_id.0, payload)
        .await?;
    Ok(Json(ApiResponse::success((), None)))
}

pub async fn delete_user_subscription(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    user_id: UserId,
) -> Result<Json<ApiResponse<()>>, AppError> {
    state
        .user_subscription_service()
        .delete_subscription(id, user_id.0)
        .await?;
    Ok(Json(ApiResponse::success((), None)))
}
