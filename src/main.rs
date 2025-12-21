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
    routing::{delete, get, post, put},
};
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;

#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::PgPool,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
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

    let state = AppState { db: pool };

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
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> &'static str {
    "Phoebudget Backend is Online!"
}
