use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::schemas::UserIdentityRow;

pub struct UserIdentityRepository {
    pool: PgPool,
}

impl UserIdentityRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_provider_subject(
        &self,
        provider: &str,
        provider_subject: &str,
    ) -> Result<Option<UserIdentityRow>, AppError> {
        let row = sqlx::query_as::<_, UserIdentityRow>(
            r#"
            SELECT id, user_id, provider, provider_subject, email, email_verified, name, picture_url,
                   created_at, updated_at
            FROM user_identities
            WHERE provider = $1 AND provider_subject = $2
            "#,
        )
        .bind(provider)
        .bind(provider_subject)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn create_identity(
        &self,
        user_id: Uuid,
        provider: &str,
        provider_subject: &str,
        email: Option<&str>,
        email_verified: Option<bool>,
        name: Option<&str>,
        picture_url: Option<&str>,
    ) -> Result<Uuid, AppError> {
        let id = sqlx::query_scalar::<_, Uuid>(
            r#"
            INSERT INTO user_identities (
                user_id, provider, provider_subject, email, email_verified, name, picture_url
            ) VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id
            "#,
        )
        .bind(user_id)
        .bind(provider)
        .bind(provider_subject)
        .bind(email)
        .bind(email_verified)
        .bind(name)
        .bind(picture_url)
        .fetch_one(&self.pool)
        .await?;
        Ok(id)
    }
}
