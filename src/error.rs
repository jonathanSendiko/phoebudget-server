use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

use crate::i18n;

// The JSON structure for errors
#[derive(Serialize)]
pub struct ErrorResponse {
    pub success: bool,
    pub errors: Vec<ErrorDetail>,
}

#[derive(Serialize)]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
}

// The Enum for code logic
#[derive(Debug)]
pub enum AppError {
    DatabaseError(sqlx::Error),
    ValidationError(String),
    AuthError(String),
    NotFoundError(String),
    InternalServerError(String),
    SubscriptionLimit {
        feature: String,
        limit: i32,
        current: i64,
    },
    PremiumRequired {
        feature: String,
    },
}

// Convert AppError -> HTTP Response
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            AppError::DatabaseError(e) => {
                println!("Database Error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "DB-500".to_string(),
                    i18n::localize_message("Internal Server Error"),
                )
            }
            AppError::ValidationError(msg) => (
                StatusCode::BAD_REQUEST,
                "VAL-400".to_string(),
                i18n::localize_message(&msg),
            ),
            AppError::AuthError(msg) => (
                StatusCode::UNAUTHORIZED,
                "AUTH-401".to_string(),
                i18n::localize_message(&msg),
            ),
            AppError::NotFoundError(msg) => (
                StatusCode::NOT_FOUND,
                "NOT-404".to_string(),
                i18n::localize_message(&msg),
            ),
            AppError::InternalServerError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INT-500".to_string(),
                i18n::localize_message(&msg),
            ),
            AppError::SubscriptionLimit {
                feature,
                limit,
                current,
            } => (
                StatusCode::FORBIDDEN,
                "SUBSCRIPTION_LIMIT".to_string(),
                i18n::subscription_limit_message(&feature, limit, current),
            ),
            AppError::PremiumRequired { feature } => (
                StatusCode::FORBIDDEN,
                "PREMIUM_REQUIRED".to_string(),
                i18n::premium_required_message(&feature),
            ),
        };

        let body = Json(ErrorResponse {
            success: false,
            errors: vec![ErrorDetail { code, message }],
        });

        (status, body).into_response()
    }
}

// Allow ? operator for SQLx errors
impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::DatabaseError(err)
    }
}
