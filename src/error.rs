use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

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
                    "Internal Server Error".to_string(),
                )
            }
            AppError::ValidationError(msg) => (StatusCode::BAD_REQUEST, "VAL-400".to_string(), msg),
            AppError::AuthError(msg) => (StatusCode::UNAUTHORIZED, "AUTH-401".to_string(), msg),
            AppError::NotFoundError(msg) => (StatusCode::NOT_FOUND, "NOT-404".to_string(), msg),
            AppError::InternalServerError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INT-500".to_string(),
                msg,
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
