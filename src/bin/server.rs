use axum::{
    Router, middleware,
    routing::{delete, get, post, put},
};
use phoebudget::{AppState, handlers, print_request_response};
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use std::time::Duration;
use tower_governor::{
    GovernorLayer, governor::GovernorConfigBuilder, key_extractor::SmartIpKeyExtractor,
};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

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
    let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL must be set");

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

    println!("Running database migrations...");
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to migrate database");

    // Initialize Redis
    let redis_client = redis::Client::open(redis_url).expect("Invalid Redis URL");

    let cache = moka::future::Cache::builder()
        .time_to_live(std::time::Duration::from_secs(3))
        .build();

    let exchange_rate_cache = moka::future::Cache::builder()
        .time_to_live(std::time::Duration::from_secs(60))
        .build();

    let http_client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .build()
        .expect("Failed to build HTTP client");

    let state = AppState {
        db: pool,
        price_cache: cache,
        exchange_rate_cache,
        http_client,
        redis_client,
    };

    // Configure rate limiter: 60 requests per minute per IP
    let governor_conf = GovernorConfigBuilder::default()
        .key_extractor(SmartIpKeyExtractor)
        .per_second(1)
        .burst_size(60)
        .finish()
        .expect("Failed to build rate limiter config");

    let governor_limiter = governor_conf.limiter().clone();

    // Background task to clean up stale rate limiter entries
    let cleanup_interval = Duration::from_secs(60);
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(cleanup_interval).await;
            tracing::debug!("Rate limiter storage size: {}", governor_limiter.len());
            governor_limiter.retain_recent();
        }
    });

    let api_routes = Router::new()
        .route("/auth/register", post(handlers::register))
        .route("/auth/login", post(handlers::login))
        .route("/auth/refresh", post(handlers::refresh_token))
        .route(
            "/transactions",
            post(handlers::create_transaction).get(handlers::get_transactions),
        )
        .route(
            "/transactions/{id}",
            put(handlers::update_transaction)
                .delete(handlers::delete_transaction)
                .get(handlers::get_transaction),
        )
        .route(
            "/transactions/{id}/restore",
            post(handlers::restore_transaction),
        )
        .route("/settings/currency", put(handlers::update_base_currency))
        .route(
            "/settings/currencies",
            get(handlers::get_available_currencies),
        )
        .route("/categories", get(handlers::get_categories))
        .route("/analysis/category", get(handlers::get_spending_analysis))
        .route("/analysis/net-worth", get(handlers::get_financial_health))
        .route("/portfolio/refresh", post(handlers::refresh_portfolio))
        .route(
            "/portfolio/{ticker}",
            delete(handlers::remove_investment).put(handlers::update_investment),
        )
        .route("/auth/profile", get(handlers::get_profile))
        .route("/auth/subscription", get(handlers::get_subscription))
        .route(
            "/portfolio",
            post(handlers::add_investment).get(handlers::get_portfolio),
        )
        .route("/assets", get(handlers::get_assets))
        .route(
            "/pockets",
            post(handlers::create_pocket).get(handlers::get_pockets),
        )
        .route(
            "/pockets/{id}",
            get(handlers::get_pocket)
                .put(handlers::update_pocket)
                .delete(handlers::delete_pocket),
        )
        .route("/pockets/transfer", post(handlers::transfer_funds))
        .route(
            "/goals",
            post(handlers::create_goal).get(handlers::get_goals),
        )
        .route(
            "/goals/{id}",
            get(handlers::get_goal)
                .put(handlers::update_goal)
                .delete(handlers::delete_goal),
        )
        .route(
            "/goals/{id}/entries",
            post(handlers::create_goal_entry).get(handlers::get_goal_entries),
        )
        .route(
            "/subscriptions",
            post(handlers::create_user_subscription).get(handlers::get_user_subscriptions),
        )
        .route(
            "/subscriptions/{id}",
            get(handlers::get_user_subscription)
                .put(handlers::update_user_subscription)
                .delete(handlers::delete_user_subscription),
        )
        .layer(GovernorLayer::new(governor_conf));

    let app = Router::new()
        .route("/", get(handlers::health_check))
        .nest("/api/v1", api_routes)
        .layer(TraceLayer::new_for_http())
        .layer(middleware::from_fn(print_request_response))
        .with_state(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let port = port.parse::<u16>().expect("Invalid PORT");
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}
