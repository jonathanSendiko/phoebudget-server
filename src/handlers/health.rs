pub async fn health_check() -> impl axum::response::IntoResponse {
    axum::http::StatusCode::OK
}
