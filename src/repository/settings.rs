use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;

pub struct SettingsRepository {
    pool: PgPool,
}

impl SettingsRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_base_currency(&self, user_id: Uuid) -> Result<String, AppError> {
        let settings = sqlx::query!(
            "SELECT base_currency FROM user_settings WHERE user_id = $1",
            user_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(settings
            .and_then(|r| r.base_currency)
            .unwrap_or_else(|| "SGD".to_string()))
    }

    pub async fn set_base_currency(&self, user_id: Uuid, currency: &str) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            INSERT INTO user_settings (user_id, base_currency)
            VALUES ($1, $2)
            ON CONFLICT (user_id) 
            DO UPDATE SET base_currency = EXCLUDED.base_currency
            "#,
            user_id,
            currency
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn validate_currency(&self, code: &str) -> Result<bool, AppError> {
        let result = sqlx::query!("SELECT 1 as exists FROM currencies WHERE code = $1", code)
            .fetch_optional(&self.pool)
            .await?;
        Ok(result.is_some())
    }

    pub async fn get_available_currencies(&self) -> Result<Vec<String>, AppError> {
        let rows = sqlx::query!("SELECT code FROM currencies ORDER BY code")
            .fetch_all(&self.pool)
            .await?;
        Ok(rows.into_iter().map(|r| r.code).collect())
    }
}
