use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::schemas::SubscriptionRow;

pub struct SubscriptionRepository {
    pool: PgPool,
}

impl SubscriptionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_by_user(&self, user_id: Uuid) -> Result<Option<SubscriptionRow>, AppError> {
        let row = sqlx::query_as!(
            SubscriptionRow,
            r#"
            SELECT 
                id, user_id, plan, status, started_at, expires_at, 
                payment_provider, external_subscription_id
            FROM subscriptions
            WHERE user_id = $1
            "#,
            user_id
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn create_default(&self, user_id: Uuid) -> Result<Uuid, AppError> {
        let id = sqlx::query_scalar!(
            r#"
            INSERT INTO subscriptions (user_id, plan, status)
            VALUES ($1, 'free', 'active')
            RETURNING id
            "#,
            user_id
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(id)
    }
}
