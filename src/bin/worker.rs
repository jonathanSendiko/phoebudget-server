use phoebudget::repository::{PocketRepository, TransactionRepository, UserSubscriptionRepository};
use phoebudget::services::UserSubscriptionService;
use redis::AsyncCommands;
use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::fmt::init();

    tracing::info!("Starting Worker...");

    let db_url = std::env::var("DATABASE_URL")
        .ok()
        .or_else(|| {
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
        .max_connections(5)
        .connect(&db_url)
        .await
        .expect("Failed to connect to DB");

    let client = redis::Client::open(redis_url).expect("Invalid Redis URL");
    let mut con = client
        .get_async_connection()
        .await
        .expect("Failed to connect to Redis");

    // Initialize Service (Need Repos)
    // We need to implement `get_row_by_id` or similar in repo to make this efficient,
    // but for now `UserSubscriptionService` needs basic repos.
    let sub_service = UserSubscriptionService::new(
        UserSubscriptionRepository::new(pool.clone()),
        PocketRepository::new(pool.clone()),
        TransactionRepository::new(pool.clone()),
    );

    // We also need access to the repo directly to fetch the Row by ID?
    // Or we can add `get_row` to service?
    // Current service has `get_subscription` returning Detail DTO (with joined data).
    // `process_due_subscription` takes `UserSubscriptionRow`.
    // We need a way to get `UserSubscriptionRow` from ID.
    // I'll cheat slightly and make a quick repo instance to get it, or SQL query here.
    // To keep it clean, I should really use the repo.
    // But `UserSubscriptionRepository` doesn't have `get_row_by_id`.
    // I'll assume I can just use `sqlx::query_as!` here since I have the pool.

    loop {
        // BLPOP: Block until item available. Timeout 0 = infinite.
        // Returns (key, value) tuple.
        let result: redis::RedisResult<(String, String)> =
            con.blpop("subscription_jobs", 0.0).await;

        match result {
            Ok((_key, sub_id_str)) => {
                tracing::info!("Processing job: {}", sub_id_str);

                if let Ok(sub_id) = Uuid::parse_str(&sub_id_str) {
                    // Fetch the Row
                    let row_result = sqlx::query_as!(
                        phoebudget::schemas::UserSubscriptionRow,
                        "SELECT * FROM user_subscriptions WHERE id = $1",
                        sub_id
                    )
                    .fetch_optional(&pool)
                    .await;

                    match row_result {
                        Ok(Some(row)) => {
                            if let Err(e) = sub_service.process_due_subscription(&row).await {
                                tracing::error!(
                                    "Failed to process subscription {}: {:?}",
                                    sub_id,
                                    e
                                );
                                // Retry logic? Push back to queue?
                                // For now, just log.
                            } else {
                                tracing::info!("Successfully processed subscription {}", sub_id);
                            }
                        }
                        Ok(None) => tracing::warn!("Subscription {} not found in DB", sub_id),
                        Err(e) => {
                            tracing::error!("DB error fetching subscription {}: {:?}", sub_id, e)
                        }
                    }
                } else {
                    tracing::error!("Invalid UUID in job: {}", sub_id_str);
                }
            }
            Err(e) => {
                tracing::error!("Redis BLPOP error: {:?}", e);
                // Sleep brief moment to avoid tight loop on connection error
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            }
        }
    }
}
