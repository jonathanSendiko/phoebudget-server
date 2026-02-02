use axum::{Json, extract::State};

use crate::AppState;
use crate::error::AppError;
use crate::repository::SettingsRepository;
use crate::response::ApiResponse;

pub async fn get_available_currencies(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<String>>>, AppError> {
    let settings_repo = SettingsRepository::new(state.db.clone());
    let currencies = settings_repo.get_available_currencies().await?;
    Ok(Json(ApiResponse::success(currencies, None)))
}
