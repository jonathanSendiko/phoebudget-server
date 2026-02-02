use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;

pub struct GoalEntryRepository {
    pool: PgPool,
}

impl GoalEntryRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        goal_id: Uuid,
        amount: Decimal,
        description: Option<String>,
        date: Option<DateTime<Utc>>,
    ) -> Result<Uuid, AppError> {
        let date = date.unwrap_or_else(Utc::now);
        let id = sqlx::query_scalar!(
            r#"
            INSERT INTO goal_entries (goal_id, amount, description, date)
            VALUES ($1, $2, $3, $4)
            RETURNING id
            "#,
            goal_id,
            amount,
            description,
            date
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(id)
    }

    pub async fn get_by_goal(
        &self,
        goal_id: Uuid,
    ) -> Result<Vec<crate::schemas::GoalEntry>, AppError> {
        let entries = sqlx::query_as!(
            crate::schemas::GoalEntry,
            r#"
            SELECT id, goal_id, amount, description, date
            FROM goal_entries
            WHERE goal_id = $1
            ORDER BY date DESC, created_at DESC
            "#,
            goal_id
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(entries)
    }
}
