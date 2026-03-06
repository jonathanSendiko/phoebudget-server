use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::schemas::PocketSummary;

pub struct GoalRepository {
    pool: PgPool,
}

impl GoalRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        user_id: Uuid,
        pocket_id: Uuid,
        name: &str,
        description: Option<String>,
        target_amount: Decimal,
        current_amount: Option<Decimal>,
        icon: Option<String>,
    ) -> Result<Uuid, AppError> {
        let icon = icon.unwrap_or_else(|| "savings".to_string());
        let current_amount = current_amount.unwrap_or(Decimal::ZERO);
        let id = sqlx::query_scalar!(
            r#"
            INSERT INTO financial_goals (user_id, pocket_id, name, description, target_amount, current_amount, icon)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id
            "#,
            user_id,
            pocket_id,
            name,
            description,
            target_amount,
            current_amount,
            icon
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(id)
    }

    pub async fn get_all(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<crate::schemas::GoalSummary>, AppError> {
        struct Row {
            id: Uuid,
            name: String,
            icon: String,
            target_amount: Decimal,
            current_amount: Decimal,
        }

        let rows = sqlx::query_as!(
            Row,
            r#"
            SELECT 
                id, name, 
                COALESCE(icon, 'savings') as "icon!", 
                target_amount,
                current_amount
            FROM financial_goals
            WHERE user_id = $1
            ORDER BY created_at DESC
            "#,
            user_id
        )
        .fetch_all(&self.pool)
        .await?;

        let summaries = rows
            .into_iter()
            .map(|r| {
                let percentage = if r.target_amount.is_zero() {
                    Decimal::ZERO
                } else {
                    (r.current_amount / r.target_amount) * Decimal::from(100)
                };

                crate::schemas::GoalSummary {
                    id: r.id,
                    name: r.name,
                    icon: r.icon,
                    target_amount: r.target_amount,
                    current_amount: r.current_amount,
                    percentage,
                }
            })
            .collect();

        Ok(summaries)
    }

    pub async fn get_by_id(
        &self,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<crate::schemas::GoalDetail, AppError> {
        struct Row {
            id: Uuid,
            name: String,
            description: Option<String>,
            icon: String,
            target_amount: Decimal,
            current_amount: Decimal,
            created_at: Option<DateTime<Utc>>,
            pocket_id: Uuid,
            pocket_name: String,
            pocket_icon: String,
        }

        let row = sqlx::query_as!(
            Row,
            r#"
            SELECT 
                g.id, g.name, g.description,
                COALESCE(g.icon, 'savings') as "icon!", 
                g.target_amount, g.current_amount, g.created_at,
                p.id as "pocket_id!", p.name as "pocket_name!", COALESCE(p.icon, 'account_balance') as "pocket_icon!"
            FROM financial_goals g
            JOIN pockets p ON g.pocket_id = p.id
            WHERE g.id = $1 AND g.user_id = $2
            "#,
            id,
            user_id
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(AppError::NotFoundError("Goal not found".to_string()))?;

        let percentage = if row.target_amount.is_zero() {
            Decimal::ZERO
        } else {
            (row.current_amount / row.target_amount) * Decimal::from(100)
        };

        Ok(crate::schemas::GoalDetail {
            id: row.id,
            name: row.name,
            description: row.description,
            icon: row.icon,
            target_amount: row.target_amount,
            current_amount: row.current_amount,
            percentage,
            pocket: PocketSummary {
                id: row.pocket_id,
                name: row.pocket_name,
                icon: row.pocket_icon,
            },
            created_at: row.created_at,
        })
    }

    pub async fn update(
        &self,
        id: Uuid,
        user_id: Uuid,
        name: Option<String>,
        description: Option<String>,
        target_amount: Option<Decimal>,
        current_amount: Option<Decimal>,
        pocket_id: Option<Uuid>,
        icon: Option<String>,
    ) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            UPDATE financial_goals
            SET 
                name = COALESCE($3, name),
                description = COALESCE($4, description),
                target_amount = COALESCE($5, target_amount),
                current_amount = COALESCE($6, current_amount),
                pocket_id = COALESCE($7, pocket_id),
                icon = COALESCE($8, icon),
                updated_at = NOW()
            WHERE id = $1 AND user_id = $2
            "#,
            id,
            user_id,
            name,
            description,
            target_amount,
            current_amount,
            pocket_id,
            icon
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn delete(&self, id: Uuid, user_id: Uuid) -> Result<u64, AppError> {
        let result = sqlx::query!(
            "DELETE FROM financial_goals WHERE id = $1 AND user_id = $2",
            id,
            user_id
        )
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected())
    }
}
