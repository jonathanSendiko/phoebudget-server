use phoebudget::repository::UserSubscriptionRepository;
use redis::Commands;
use sqlx::postgres::PgPoolOptions;
use std::time::Duration;
use tokio::time;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::fmt::init();

    tracing::info!("Starting Scheduler...");

    let db_url = std::env::var("DATABASE_URL")
        .ok()
        .or_else(|| {
            // Construct from parts if DATABASE_URL not set (likely based on docker-compose)
            let user = std::env::var("DB_USERNAME").ok()?;
            let pass = std::env::var("DB_PASSWORD").ok()?;
            let host = std::env::var("DB_HOST").ok()?;
            let port = std::env::var("DB_PORT").ok()?;
            let name = std::env::var("DB_NAME").ok()?;
            Some(format!(
                "postgres://{}:{}@{}:{}/{}",
                user, pass, host, port, name
            ))
        })
        .expect("Database config not found");

    let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&db_url)
        .await
        .expect("Failed to connect to DB");

    let client = redis::Client::open(redis_url).expect("Invalid Redis URL");
    let mut con = client.get_connection().expect("Failed to connect to Redis");

    let repo = UserSubscriptionRepository::new(pool.clone());
    let mut interval = time::interval(Duration::from_secs(3600));

    loop {
        interval.tick().await;
        tracing::info!("Checking for due subscriptions...");

        match repo.get_due_subscriptions().await {
            Ok(due_subs) => {
                if due_subs.is_empty() {
                    tracing::info!("No due subscriptions found.");
                } else {
                    tracing::info!("Found {} due subscriptions.", due_subs.len());
                    for sub in due_subs {
                        let _: () = con
                            .rpush("subscription_jobs", sub.id.to_string())
                            .expect("Redis push failed");
                    }
                    tracing::info!("Jobs pushed to Redis.");
                }
            }
            Err(e) => {
                tracing::error!("Failed to fetch due subscriptions: {:?}", e);
            }
        }
    }
}
