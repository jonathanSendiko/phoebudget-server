mod auth;
mod error;
mod handlers;
mod investments;
mod repository;
mod response;
mod schemas;
mod services;

use axum::{
    Router,
    body::Body,
    extract::Request,
    middleware::{self, Next},
    response::Response,
    routing::{delete, get, post, put},
};
use http_body_util::BodyExt;
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

async fn print_request_body(
    request: Request,
    next: Next,
) -> Result<Response, axum::http::StatusCode> {
    let (parts, body) = request.into_parts();
    let bytes = buffer_and_print("request", body).await?;
    let req = Request::from_parts(parts, Body::from(bytes));
    Ok(next.run(req).await)
}

async fn buffer_and_print<B>(
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

#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::PgPool,
    pub price_cache: moka::future::Cache<String, rust_decimal::Decimal>,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "phoebudget=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let db_user = std::env::var("DB_USERNAME").expect("DB_USERNAME must be set");
    let db_password = std::env::var("DB_PASSWORD").expect("DB_PASSWORD must be set");
    let db_host = std::env::var("DB_HOST").expect("DB_HOST must be set");
    let db_port = std::env::var("DB_PORT").expect("DB_PORT must be set");
    let db_name = std::env::var("DB_NAME").expect("DB_NAME must be set");

    let database_url = format!(
        "postgres://{}:{}@{}:{}/{}",
        db_user, db_password, db_host, db_port, db_name
    );
    println!("Connecting to DB: {}", database_url);

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    let cache = moka::future::Cache::builder()
        .time_to_live(std::time::Duration::from_secs(3))
        .build();

    let state = AppState {
        db: pool,
        price_cache: cache,
    };

    let api_routes = Router::new()
        .route("/auth/register", post(handlers::register))
        .route("/auth/login", post(handlers::login))
        .route(
            "/transactions",
            post(handlers::create_transaction).get(handlers::get_transactions),
        )
        .route(
            "/transactions/{id}",
            put(handlers::update_transaction).delete(handlers::delete_transaction),
        )
        .route("/settings/currency", put(handlers::update_base_currency))
        .route("/analysis/category", get(handlers::get_spending_analysis))
        .route("/analysis/net-worth", get(handlers::get_financial_health))
        .route("/portfolio/refresh", post(handlers::refresh_portfolio))
        .route(
            "/portfolio/{ticker}",
            delete(handlers::remove_investment).put(handlers::update_investment),
        )
        .route("/auth/profile", get(handlers::get_profile))
        .route(
            "/portfolio",
            post(handlers::add_investment).get(handlers::get_portfolio),
        );

    let app = Router::new()
        .route("/", get(health_check))
        .nest("/api/v1", api_routes)
        .layer(TraceLayer::new_for_http())
        .layer(middleware::from_fn(print_request_body))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> &'static str {
    "Phoebudget Backend is Online!"
}
