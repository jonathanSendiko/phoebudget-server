use axum::{Json, extract::State};

use crate::AppState;
use crate::error::AppError;
use crate::repository::PortfolioRepository;
use crate::response::ApiResponse;

pub async fn get_assets(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<crate::schemas::Asset>>>, AppError> {
    let portfolio_repo = PortfolioRepository::new(state.db.clone());
    let assets = portfolio_repo.get_all_assets().await?;
    Ok(Json(ApiResponse::success(assets, None)))
}
