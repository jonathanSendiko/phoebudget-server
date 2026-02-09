use axum::{Json, extract::State};

use crate::AppState;
use crate::auth::UserId;
use crate::error::AppError;
use crate::response::ApiResponse;
use crate::schemas::{
    AuthResponse, LoginRequest, OAuthLoginRequest, RefreshTokenRequest, RegisterRequest,
    SubscriptionResponse, UserProfile,
};

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

pub async fn oauth_login(
    State(state): State<AppState>,
    Json(payload): Json<OAuthLoginRequest>,
) -> Result<Json<ApiResponse<AuthResponse>>, AppError> {
    let response = state.auth_service().oauth_login(payload).await?;
    Ok(Json(ApiResponse::success(response, None)))
}

pub async fn get_profile(
    State(state): State<AppState>,
    user_id: UserId,
) -> Result<Json<ApiResponse<UserProfile>>, AppError> {
    let profile = state.auth_service().get_profile(user_id.0).await?;
    Ok(Json(ApiResponse::success(profile, None)))
}

pub async fn get_subscription(
    State(state): State<AppState>,
    user_id: UserId,
) -> Result<Json<ApiResponse<SubscriptionResponse>>, AppError> {
    let subscription = state
        .subscription_service()
        .get_subscription(user_id.0)
        .await?;
    Ok(Json(ApiResponse::success(subscription, None)))
}

pub async fn nuke_user_data(
    State(state): State<AppState>,
    user_id: UserId,
) -> Result<Json<ApiResponse<()>>, AppError> {
    state.data_deletion_service().nuke_user_data(user_id.0).await?;
    Ok(Json(ApiResponse::success(
        (),
        Some("User data deleted".to_string()),
    )))
}
