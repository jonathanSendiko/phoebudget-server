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
        sub_goal_id: Option<Uuid>,
        amount: Decimal,
        description: Option<String>,
        date: Option<DateTime<Utc>>,
    ) -> Result<Uuid, AppError> {
        let date = date.unwrap_or_else(Utc::now);
        let id = sqlx::query_scalar::<_, Uuid>(
            r#"
            INSERT INTO goal_entries (goal_id, sub_goal_id, amount, description, date)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id
            "#,
        )
        .bind(goal_id)
        .bind(sub_goal_id)
        .bind(amount)
        .bind(description)
        .bind(date)
        .fetch_one(&self.pool)
        .await?;
        Ok(id)
    }

    pub async fn get_by_goal(
        &self,
        goal_id: Uuid,
    ) -> Result<Vec<crate::schemas::GoalEntry>, AppError> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            goal_id: Uuid,
            sub_goal_id: Option<Uuid>,
            amount: Decimal,
            description: Option<String>,
            date: DateTime<Utc>,
        }

        let entries = sqlx::query_as::<_, Row>(
            r#"
            SELECT id, goal_id, sub_goal_id, amount, description, date
            FROM goal_entries
            WHERE goal_id = $1
            ORDER BY date DESC, created_at DESC
            "#,
        )
        .bind(goal_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(entries
            .into_iter()
            .map(|entry| crate::schemas::GoalEntry {
                id: entry.id,
                goal_id: entry.goal_id,
                sub_goal_id: entry.sub_goal_id,
                amount: entry.amount,
                description: entry.description,
                date: entry.date,
            })
            .collect())
    }
}
