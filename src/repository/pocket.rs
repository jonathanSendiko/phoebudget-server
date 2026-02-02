use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::schemas::Pocket;

pub struct PocketRepository {
    pool: PgPool,
}

impl PocketRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        user_id: Uuid,
        name: &str,
        description: Option<String>,
        icon: Option<String>,
    ) -> Result<Uuid, AppError> {
        let icon = icon.unwrap_or_else(|| "account_balance_wallet".to_string());
        let id = sqlx::query_scalar!(
            r#"
            INSERT INTO pockets (user_id, name, description, icon)
            VALUES ($1, $2, $3, $4)
            RETURNING id
            "#,
            user_id,
            name,
            description,
            icon
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(id)
    }

    pub async fn create_default_for_user(&self, user_id: Uuid) -> Result<Uuid, AppError> {
        let id = sqlx::query_scalar!(
            r#"
            INSERT INTO pockets (user_id, name, is_default)
            VALUES ($1, 'Main', TRUE)
            RETURNING id
            "#,
            user_id
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(id)
    }

    pub async fn get_all(&self, user_id: Uuid) -> Result<Vec<Pocket>, AppError> {
        let pockets = sqlx::query_as!(
            Pocket,
            r#"
            SELECT 
                id, name, description, 
                COALESCE(icon, 'account_balance_wallet') as "icon!",
                COALESCE(is_default, FALSE) as "is_default!",
                created_at
            FROM pockets
            WHERE user_id = $1
            ORDER BY is_default DESC, name ASC
            "#,
            user_id
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(pockets)
    }

    pub async fn get_by_id(&self, id: Uuid, user_id: Uuid) -> Result<Pocket, AppError> {
        let pocket = sqlx::query_as!(
            Pocket,
            r#"
            SELECT 
                id, name, description, 
                COALESCE(icon, 'account_balance_wallet') as "icon!",
                COALESCE(is_default, FALSE) as "is_default!",
                created_at
            FROM pockets
            WHERE id = $1 AND user_id = $2
            "#,
            id,
            user_id
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(AppError::NotFoundError("Pocket not found".to_string()))?;
        Ok(pocket)
    }

    pub async fn get_default(&self, user_id: Uuid) -> Result<Pocket, AppError> {
        let pocket = sqlx::query_as!(
            Pocket,
            r#"
            SELECT 
                id, name, description, 
                COALESCE(icon, 'account_balance_wallet') as "icon!",
                COALESCE(is_default, FALSE) as "is_default!",
                created_at
            FROM pockets
            WHERE user_id = $1 AND is_default = TRUE
            "#,
            user_id
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(AppError::NotFoundError(
            "Default pocket not found".to_string(),
        ))?;
        Ok(pocket)
    }

    pub async fn update(
        &self,
        id: Uuid,
        user_id: Uuid,
        name: Option<String>,
        description: Option<String>,
        icon: Option<String>,
    ) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            UPDATE pockets 
            SET 
                name = COALESCE($3, name),
                description = COALESCE($4, description),
                icon = COALESCE($5, icon)
            WHERE id = $1 AND user_id = $2
            "#,
            id,
            user_id,
            name,
            description,
            icon
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn delete(&self, id: Uuid, user_id: Uuid) -> Result<u64, AppError> {
        // First check if this is a default pocket
        let pocket = self.get_by_id(id, user_id).await?;
        if pocket.is_default {
            return Err(AppError::ValidationError(
                "Cannot delete the default pocket".to_string(),
            ));
        }

        let result = sqlx::query!(
            "DELETE FROM pockets WHERE id = $1 AND user_id = $2",
            id,
            user_id
        )
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected())
    }

    pub async fn count_by_user(&self, user_id: Uuid) -> Result<i64, AppError> {
        let result = sqlx::query!(
            r#"SELECT COUNT(*) as "count!" FROM pockets WHERE user_id = $1"#,
            user_id
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(result.count)
    }
}
