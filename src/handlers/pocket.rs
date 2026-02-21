use axum::{Json, extract::State};
use uuid::Uuid;

use crate::AppState;
use crate::auth::UserId;
use crate::error::AppError;
use crate::i18n;
use crate::response::ApiResponse;
use crate::schemas::{CreatePocket, Pocket, PocketId, UpdatePocket};

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
        Some(i18n::localize_message("Pocket created")),
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
    path: axum::extract::Path<Uuid>,
) -> Result<Json<ApiResponse<crate::schemas::PocketDetail>>, AppError> {
    let pocket = state.pocket_service().get_pocket(path.0, user_id.0).await?;
    Ok(Json(ApiResponse::success(pocket, None)))
}

pub async fn update_pocket(
    State(state): State<AppState>,
    user_id: UserId,
    path: axum::extract::Path<Uuid>,
    Json(payload): Json<UpdatePocket>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    state
        .pocket_service()
        .update_pocket(path.0, user_id.0, payload)
        .await?;
    Ok(Json(ApiResponse::success(
        i18n::localize_message("Pocket updated"),
        None,
    )))
}

pub async fn delete_pocket(
    State(state): State<AppState>,
    user_id: UserId,
    path: axum::extract::Path<Uuid>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    state
        .pocket_service()
        .delete_pocket(path.0, user_id.0)
        .await?;
    Ok(Json(ApiResponse::success(
        i18n::localize_message("Pocket deleted"),
        None,
    )))
}
