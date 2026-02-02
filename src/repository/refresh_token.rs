use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;

pub struct RefreshTokenRepository {
    pool: PgPool,
}

impl RefreshTokenRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        user_id: Uuid,
        token_hash: &str,
        expires_at: DateTime<Utc>,
    ) -> Result<Uuid, AppError> {
        let id = sqlx::query_scalar!(
            "INSERT INTO refresh_tokens (user_id, token_hash, expires_at) VALUES ($1, $2, $3) RETURNING id",
            user_id,
            token_hash,
            expires_at
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(id)
    }

    pub async fn find_by_hash_and_user(
        &self,
        token_hash: &str,
    ) -> Result<Option<crate::schemas::RefreshTokenRow>, AppError> {
        // Need a schema for this
        let row = sqlx::query_as!(
            crate::schemas::RefreshTokenRow,
            r#"
            SELECT id, user_id, token_hash, expires_at, created_at, replaced_by, is_revoked
            FROM refresh_tokens
            WHERE token_hash = $1
            "#,
            token_hash
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(row)
    }

    /// Revoke a specific token by setting replaced_by (rotation)
    pub async fn rotate(&self, old_id: Uuid, new_hash: &str) -> Result<(), AppError> {
        sqlx::query!(
            "UPDATE refresh_tokens SET replaced_by = $1 WHERE id = $2",
            new_hash,
            old_id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Revoke all tokens for a user (security breach)
    pub async fn revoke_all_for_user(&self, user_id: Uuid) -> Result<(), AppError> {
        sqlx::query!(
            "UPDATE refresh_tokens SET is_revoked = TRUE WHERE user_id = $1",
            user_id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
