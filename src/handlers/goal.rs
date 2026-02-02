use axum::{
    Json,
    extract::{Path, State},
};
use uuid::Uuid;

use crate::AppState;
use crate::auth::UserId;
use crate::error::AppError;
use crate::response::ApiResponse;
use crate::schemas::{CreateGoal, GoalDetail, GoalSummary};

pub async fn create_goal(
    State(state): State<AppState>,
    user_id: UserId,
    Json(payload): Json<CreateGoal>,
) -> Result<Json<ApiResponse<crate::schemas::GoalId>>, AppError> {
    let id = state.goal_service().create_goal(user_id.0, payload).await?;
    Ok(Json(ApiResponse::success(
        crate::schemas::GoalId { id },
        None,
    )))
}

pub async fn get_goals(
    State(state): State<AppState>,
    user_id: UserId,
) -> Result<Json<ApiResponse<Vec<GoalSummary>>>, AppError> {
    let goals = state.goal_service().get_goals(user_id.0).await?;
    Ok(Json(ApiResponse::success(goals, None)))
}

pub async fn get_goal(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    user_id: UserId,
) -> Result<Json<ApiResponse<GoalDetail>>, AppError> {
    let goal = state.goal_service().get_goal(id, user_id.0).await?;
    Ok(Json(ApiResponse::success(goal, None)))
}

pub async fn update_goal(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    user_id: UserId,
    Json(payload): Json<crate::schemas::UpdateGoal>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    state
        .goal_service()
        .update_goal(id, user_id.0, payload)
        .await?;
    Ok(Json(ApiResponse::success((), None)))
}

pub async fn delete_goal(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    user_id: UserId,
) -> Result<Json<ApiResponse<()>>, AppError> {
    state.goal_service().delete_goal(id, user_id.0).await?;
    Ok(Json(ApiResponse::success((), None)))
}

pub async fn create_goal_entry(
    State(state): State<AppState>,
    Path(goal_id): Path<Uuid>,
    user_id: UserId,
    Json(payload): Json<crate::schemas::CreateGoalEntry>,
) -> Result<Json<ApiResponse<crate::schemas::GoalId>>, AppError> {
    let id = state
        .goal_service()
        .create_goal_entry(goal_id, user_id.0, payload)
        .await?;
    Ok(Json(ApiResponse::success(
        crate::schemas::GoalId { id },
        None,
    )))
}

pub async fn get_goal_entries(
    State(state): State<AppState>,
    Path(goal_id): Path<Uuid>,
    user_id: UserId,
) -> Result<Json<ApiResponse<Vec<crate::schemas::GoalEntry>>>, AppError> {
    let entries = state
        .goal_service()
        .get_goal_entries(goal_id, user_id.0)
        .await?;
    Ok(Json(ApiResponse::success(entries, None)))
}
