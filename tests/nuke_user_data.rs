use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
    routing::delete,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{EncodingKey, Header, encode};
use phoebudget::{AppState, auth::Claims, handlers};
use sqlx::{PgPool, postgres::PgPoolOptions};
use tower::util::ServiceExt;
use uuid::Uuid;

async fn connect_test_db() -> Option<PgPool> {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .ok()
        .or_else(|| std::env::var("DATABASE_URL").ok())?;

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .ok()?;

    sqlx::migrate!().run(&pool).await.ok()?;
    Some(pool)
}

async fn insert_user_graph(
    pool: &PgPool,
    user_id: Uuid,
    category_id: i32,
    pocket_id: Uuid,
    goal_id: Uuid,
) {
    sqlx::query("INSERT INTO users (id, username, email, password_hash) VALUES ($1, $2, $3, $4)")
        .bind(user_id)
        .bind(format!("nuke_user_{}", user_id))
        .bind(format!("nuke_{}@example.com", user_id))
        .bind("pw")
        .execute(pool)
        .await
        .unwrap();

    sqlx::query("INSERT INTO user_settings (user_id, base_currency) VALUES ($1, 'USD')")
        .bind(user_id)
        .execute(pool)
        .await
        .unwrap();

    sqlx::query("INSERT INTO subscriptions (user_id, plan, status) VALUES ($1, 'free', 'active')")
        .bind(user_id)
        .execute(pool)
        .await
        .unwrap();

    sqlx::query(
        "INSERT INTO refresh_tokens (user_id, token_hash, expires_at, is_revoked) VALUES ($1, $2, $3, FALSE)",
    )
    .bind(user_id)
    .bind(format!("{:064x}", 1))
    .bind(Utc::now() + Duration::days(30))
    .execute(pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO user_identities (user_id, provider, provider_subject, email, email_verified) VALUES ($1, 'google', $2, $3, TRUE)",
    )
    .bind(user_id)
    .bind(format!("subject-{}", user_id))
    .bind(format!("nuke_{}@example.com", user_id))
    .execute(pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO pockets (id, user_id, name, is_default) VALUES ($1, $2, 'Main', TRUE)",
    )
    .bind(pocket_id)
    .bind(user_id)
    .execute(pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO transactions (user_id, amount, description, category_id, occurred_at, pocket_id) VALUES ($1, 10.0, 'test', $2, NOW(), $3)",
    )
    .bind(user_id)
    .bind(category_id)
    .bind(pocket_id)
    .execute(pool)
    .await
    .unwrap();

    let ticker = format!(
        "NUKE{}",
        user_id
            .to_string()
            .replace('-', "")
            .chars()
            .take(6)
            .collect::<String>()
    );
    sqlx::query("INSERT INTO assets (ticker, name, asset_type) VALUES ($1, 'Nuke Asset', 'Stock') ON CONFLICT (ticker) DO NOTHING")
        .bind(&ticker)
        .execute(pool)
        .await
        .unwrap();

    sqlx::query(
        "INSERT INTO portfolio (user_id, ticker, quantity, avg_buy_price) VALUES ($1, $2, 1.0, 10.0)",
    )
    .bind(user_id)
    .bind(&ticker)
    .execute(pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO financial_goals (id, user_id, pocket_id, name, target_amount) VALUES ($1, $2, $3, 'Goal', 100.0)",
    )
    .bind(goal_id)
    .bind(user_id)
    .bind(pocket_id)
    .execute(pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO goal_entries (goal_id, amount, description, date) VALUES ($1, 5.0, 'entry', NOW())",
    )
    .bind(goal_id)
    .execute(pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO user_subscriptions (user_id, pocket_id, name, amount, basis, billing_day, billing_month, category_id, next_charge_date) VALUES ($1, $2, 'Netflix', 9.99, 'annually', 1, 1, $3, CURRENT_DATE)",
    )
    .bind(user_id)
    .bind(pocket_id)
    .bind(category_id)
    .execute(pool)
    .await
    .unwrap();
}

async fn count_by_user(pool: &PgPool, table: &str, user_id: Uuid) -> i64 {
    let q = format!("SELECT COUNT(*) FROM {} WHERE user_id = $1", table);
    sqlx::query_scalar::<_, i64>(&q)
        .bind(user_id)
        .fetch_one(pool)
        .await
        .unwrap()
}

#[tokio::test]
async fn nuke_endpoint_deletes_all_user_data() {
    let Some(pool) = connect_test_db().await else {
        return;
    };

    unsafe { std::env::set_var("JWT_SECRET", "test-secret") };

    let redis_client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let state = AppState {
        db: pool.clone(),
        price_cache: moka::future::Cache::builder().build(),
        exchange_rate_cache: moka::future::Cache::builder().build(),
        http_client: reqwest::Client::new(),
        redis_client,
        itick_api_key: None,
    };

    let app = Router::new()
        .route("/api/v1/auth/nuke", delete(handlers::auth::nuke_user_data))
        .with_state(state);

    let user_id = Uuid::new_v4();
    let pocket_id = Uuid::new_v4();
    let goal_id = Uuid::new_v4();

    let category_id: i32 = sqlx::query_scalar(
        "INSERT INTO categories (name, is_income) VALUES ($1, FALSE) RETURNING id",
    )
    .bind(format!("Nuke Category {}", user_id))
    .fetch_one(&pool)
    .await
    .unwrap();

    insert_user_graph(&pool, user_id, category_id, pocket_id, goal_id).await;

    let claims = Claims {
        sub: user_id.to_string(),
        company: "Phoebudget".to_string(),
        exp: (Utc::now() + Duration::hours(1)).timestamp() as usize,
    };
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret("test-secret".as_bytes()),
    )
    .unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/auth/nuke")
                .method("DELETE")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    assert_eq!(count_by_user(&pool, "refresh_tokens", user_id).await, 0);
    assert_eq!(count_by_user(&pool, "user_identities", user_id).await, 0);
    assert_eq!(count_by_user(&pool, "subscriptions", user_id).await, 0);
    assert_eq!(count_by_user(&pool, "user_settings", user_id).await, 0);
    assert_eq!(count_by_user(&pool, "portfolio", user_id).await, 0);
    assert_eq!(count_by_user(&pool, "transactions", user_id).await, 0);
    assert_eq!(count_by_user(&pool, "user_subscriptions", user_id).await, 0);
    assert_eq!(count_by_user(&pool, "financial_goals", user_id).await, 0);
    assert_eq!(count_by_user(&pool, "pockets", user_id).await, 0);
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_one(&pool)
            .await
            .unwrap(),
        0
    );

    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM goal_entries WHERE goal_id = $1")
            .bind(goal_id)
            .fetch_one(&pool)
            .await
            .unwrap(),
        0
    );
}
