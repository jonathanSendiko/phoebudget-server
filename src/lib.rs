pub mod auth;
pub mod error;
pub mod handlers;
pub mod investments;
pub mod portfolio;
pub mod repository;
pub mod response;
pub mod schemas;
pub mod services;

use axum::body::Body;
use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;
use http_body_util::BodyExt;

// Shared AppState (Needs to be public now)
#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::PgPool,
    pub price_cache: moka::future::Cache<String, rust_decimal::Decimal>,
    pub exchange_rate_cache: moka::future::Cache<String, rust_decimal::Decimal>,
    pub http_client: reqwest::Client,
    pub redis_client: redis::Client, // Added Redis Client
    pub itick_api_key: Option<String>,
}

impl AppState {
    pub fn auth_service(&self) -> services::AuthService {
        services::AuthService::new(
            repository::UserRepository::new(self.db.clone()),
            repository::SettingsRepository::new(self.db.clone()),
            repository::PocketRepository::new(self.db.clone()),
            repository::RefreshTokenRepository::new(self.db.clone()),
            repository::SubscriptionRepository::new(self.db.clone()),
        )
    }

    pub fn transaction_service(&self) -> services::TransactionService {
        services::TransactionService::new(
            repository::TransactionRepository::new(self.db.clone()),
            repository::PocketRepository::new(self.db.clone()),
            repository::SettingsRepository::new(self.db.clone()),
            self.http_client.clone(),
        )
    }

    pub fn finance_service(&self) -> services::FinanceService {
        services::FinanceService::new(
            repository::PortfolioRepository::new(self.db.clone()),
            repository::TransactionRepository::new(self.db.clone()),
            repository::SettingsRepository::new(self.db.clone()),
            self.price_cache.clone(),
            self.exchange_rate_cache.clone(),
            self.http_client.clone(),
            self.itick_api_key.clone(),
        )
    }

    pub fn pocket_service(&self) -> services::PocketService {
        services::PocketService::new(
            repository::PocketRepository::new(self.db.clone()),
            repository::TransactionRepository::new(self.db.clone()),
        )
    }

    pub fn subscription_service(&self) -> services::SubscriptionService {
        services::SubscriptionService::new(repository::SubscriptionRepository::new(self.db.clone()))
    }

    pub fn goal_service(&self) -> services::GoalService {
        services::GoalService::new(
            repository::GoalRepository::new(self.db.clone()),
            repository::GoalEntryRepository::new(self.db.clone()),
            repository::PocketRepository::new(self.db.clone()),
        )
    }

    pub fn user_subscription_service(&self) -> services::UserSubscriptionService {
        services::UserSubscriptionService::new(
            repository::UserSubscriptionRepository::new(self.db.clone()),
            repository::PocketRepository::new(self.db.clone()),
            repository::TransactionRepository::new(self.db.clone()),
        )
    }
}

pub async fn print_request_response(
    request: Request,
    next: Next,
) -> Result<Response, axum::http::StatusCode> {
    let (parts, body) = request.into_parts();
    let bytes = buffer_and_print("request", body).await?;
    let req = Request::from_parts(parts, Body::from(bytes));

    let res = next.run(req).await;

    let (parts, body) = res.into_parts();
    let bytes = buffer_and_print("response", body).await?;
    let res = Response::from_parts(parts, Body::from(bytes));

    Ok(res)
}

pub async fn buffer_and_print<B>(
    direction: &str,
    body: B,
) -> Result<bytes::Bytes, axum::http::StatusCode>
where
    B: axum::body::HttpBody<Data = bytes::Bytes>,
    B::Error: std::fmt::Display,
{
    let bytes = match body.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(_err) => {
            return Err(axum::http::StatusCode::BAD_REQUEST);
        }
    };

    if let Ok(body_str) = std::str::from_utf8(&bytes) {
        tracing::debug!("{} body = {:?}", direction, body_str);
    }

    Ok(bytes)
}
