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
    println!(
        "Connecting to DB at host={} db={}",
        db_host, db_name
    );

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    let run_migrations = std::env::var("RUN_MIGRATIONS")
        .map(|value| value == "1" || value.eq_ignore_ascii_case("true"))
        .unwrap_or(true);
    if run_migrations {
        println!("Running database migrations...");
        sqlx::migrate!()
            .run(&pool)
            .await
            .expect("Failed to migrate database");

        let migrations_only = std::env::var("RUN_MIGRATIONS_ONLY")
            .map(|value| value == "1" || value.eq_ignore_ascii_case("true"))
            .unwrap_or(false);
        if migrations_only {
            println!("Migrations completed; exiting because RUN_MIGRATIONS_ONLY is set.");
            return;
        }
    }

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
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(15))
        .build()
        .expect("Failed to build HTTP client");

    let itick_api_key = std::env::var("ITICK_API_KEY").ok();
    if itick_api_key.is_none() {
        tracing::warn!("ITICK_API_KEY not set - iTick stock price fetching will not work");
    }

    let state = AppState {
        db: pool,
        price_cache: cache,
        exchange_rate_cache,
        http_client,
        redis_client,
        itick_api_key,
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
        .route("/auth/register", post(handlers::auth::register))
        .route("/auth/login", post(handlers::auth::login))
        .route("/auth/oauth", post(handlers::auth::oauth_login))
        .route("/auth/refresh", post(handlers::auth::refresh_token))
        .route(
            "/transactions",
            post(handlers::transaction::create_transaction)
                .get(handlers::transaction::get_transactions),
        )
        .route(
            "/transactions/{id}",
            put(handlers::transaction::update_transaction)
                .delete(handlers::transaction::delete_transaction)
                .get(handlers::transaction::get_transaction),
        )
        .route(
            "/transactions/{id}/restore",
            post(handlers::transaction::restore_transaction),
        )
        .route(
            "/settings/currency",
            put(handlers::finance::update_base_currency),
        )
        .route(
            "/settings/currencies",
            get(handlers::settings::get_available_currencies),
        )
        .route("/categories", get(handlers::transaction::get_categories))
        .route(
            "/analysis/category",
            get(handlers::transaction::get_spending_analysis),
        )
        .route(
            "/analysis/net-worth",
            get(handlers::finance::get_financial_health),
        )
        .route(
            "/portfolio/refresh",
            post(handlers::finance::refresh_portfolio),
        )
        .route(
            "/portfolio/{ticker}",
            delete(handlers::finance::remove_investment).put(handlers::finance::update_investment),
        )
        .route("/auth/profile", get(handlers::auth::get_profile))
        .route("/auth/subscription", get(handlers::auth::get_subscription))
        .route(
            "/portfolio",
            post(handlers::finance::add_investment).get(handlers::finance::get_portfolio),
        )
        .route("/assets", get(handlers::assets::get_assets))
        .route(
            "/pockets",
            post(handlers::pocket::create_pocket).get(handlers::pocket::get_pockets),
        )
        .route(
            "/pockets/{id}",
            get(handlers::pocket::get_pocket)
                .put(handlers::pocket::update_pocket)
                .delete(handlers::pocket::delete_pocket),
        )
        .route(
            "/pockets/transfer",
            post(handlers::transaction::transfer_funds),
        )
        .route(
            "/goals",
            post(handlers::goal::create_goal).get(handlers::goal::get_goals),
        )
        .route(
            "/goals/{id}",
            get(handlers::goal::get_goal)
                .put(handlers::goal::update_goal)
                .delete(handlers::goal::delete_goal),
        )
        .route(
            "/goals/{id}/entries",
            post(handlers::goal::create_goal_entry).get(handlers::goal::get_goal_entries),
        )
        .route(
            "/subscriptions",
            post(handlers::user_subscription::create_user_subscription)
                .get(handlers::user_subscription::get_user_subscriptions),
        )
        .route(
            "/subscriptions/{id}",
            get(handlers::user_subscription::get_user_subscription)
                .put(handlers::user_subscription::update_user_subscription)
                .delete(handlers::user_subscription::delete_user_subscription),
        )
        .layer(GovernorLayer::new(governor_conf));

    let mut app = Router::new()
        .route("/", get(handlers::health::health_check))
        .nest("/api/v1", api_routes)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    if std::env::var("LOG_REQUEST_BODY")
        .map(|value| value == "1" || value.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
    {
        app = app.layer(middleware::from_fn(print_request_response));
    }

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
