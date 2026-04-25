use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;

pub struct SubGoalRepository {
    pool: PgPool,
}

impl SubGoalRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn replace_for_goal(
        &self,
        goal_id: Uuid,
        sub_goals: &[crate::schemas::CreateSubGoal],
    ) -> Result<(), AppError> {
        let mut tx = self.pool.begin().await?;

        sqlx::query("DELETE FROM goal_sub_goals WHERE goal_id = $1")
            .bind(goal_id)
            .execute(&mut *tx)
            .await?;

        for (position, sub_goal) in sub_goals.iter().enumerate() {
            sqlx::query(
                r#"
                INSERT INTO goal_sub_goals (goal_id, name, target_amount, position)
                VALUES ($1, $2, $3, $4)
                "#,
            )
            .bind(goal_id)
            .bind(&sub_goal.name)
            .bind(sub_goal.target_amount)
            .bind(position as i32)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    pub async fn get_by_goal(
        &self,
        goal_id: Uuid,
    ) -> Result<Vec<crate::schemas::SubGoal>, AppError> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            goal_id: Uuid,
            name: String,
            target_amount: Decimal,
            position: i32,
            created_at: Option<chrono::DateTime<chrono::Utc>>,
        }

        let rows = sqlx::query_as::<_, Row>(
            r#"
            SELECT id, goal_id, name, target_amount, position, created_at
            FROM goal_sub_goals
            WHERE goal_id = $1
            ORDER BY position ASC
            "#,
        )
        .bind(goal_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| crate::schemas::SubGoal {
                id: row.id,
                goal_id: row.goal_id,
                name: row.name,
                target_amount: row.target_amount,
                position: row.position,
                created_at: row.created_at,
            })
            .collect())
    }
}
