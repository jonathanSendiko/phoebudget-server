use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::schemas::{Category, PocketSummary};

pub struct UserSubscriptionRepository {
    pool: PgPool,
}

impl UserSubscriptionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        user_id: Uuid,
        pocket_id: Uuid,
        name: &str,
        description: Option<String>,
        amount: Decimal,
        basis: &str,
        billing_day: i32,
        billing_month: Option<i32>,
        category_id: Option<i32>,
        next_charge_date: NaiveDate,
    ) -> Result<Uuid, AppError> {
        let id = sqlx::query_scalar!(
            r#"
            INSERT INTO user_subscriptions (
                user_id, pocket_id, name, description, amount, 
                basis, billing_day, billing_month, category_id, next_charge_date
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING id
            "#,
            user_id,
            pocket_id,
            name,
            description,
            amount,
            basis,
            billing_day,
            billing_month,
            category_id,
            next_charge_date
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(id)
    }

    pub async fn get_all(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<crate::schemas::UserSubscriptionSummary>, AppError> {
        let subs = sqlx::query!(
            r#"
            SELECT 
                s.id, s.name, s.amount, s.basis, s.next_charge_date, 
                COALESCE(s.is_active, TRUE) as "is_active!",
                c.icon as "category_icon?"
            FROM user_subscriptions s
            LEFT JOIN categories c ON s.category_id = c.id
            WHERE s.user_id = $1
            ORDER BY s.next_charge_date ASC
            "#,
            user_id
        )
        .fetch_all(&self.pool)
        .await?;

        let summaries = subs
            .into_iter()
            .map(|r| crate::schemas::UserSubscriptionSummary {
                id: r.id,
                name: r.name,
                amount: r.amount,
                basis: r.basis,
                next_charge_date: r.next_charge_date,
                is_active: r.is_active,
                icon: r
                    .category_icon
                    .unwrap_or_else(|| "subscriptions".to_string()),
            })
            .collect();

        Ok(summaries)
    }

    pub async fn get_by_id(
        &self,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<crate::schemas::UserSubscriptionDetail, AppError> {
        struct Row {
            id: Uuid,
            name: String,
            description: Option<String>,
            amount: Decimal,
            basis: String,
            billing_day: i32,
            billing_month: Option<i32>,
            next_charge_date: NaiveDate,
            is_active: bool,
            created_at: Option<DateTime<Utc>>,
            pocket_id: Uuid,
            pocket_name: String,
            pocket_icon: String,
            category_id: Option<i32>,
            category_name: Option<String>,
            category_icon: Option<String>,
            category_is_income: Option<bool>,
            category_exclude: Option<bool>,
        }

        let row = sqlx::query_as!(
            Row,
            r#"
            SELECT 
                s.id, s.name, s.description, s.amount, s.basis, 
                s.billing_day, s.billing_month, s.next_charge_date, 
                COALESCE(s.is_active, TRUE) as "is_active!", s.created_at,
                p.id as "pocket_id!", p.name as "pocket_name!", COALESCE(p.icon, 'account_balance') as "pocket_icon!",
                c.id as "category_id?", c.name as "category_name?", c.icon as "category_icon?", 
                c.is_income as "category_is_income?", c.exclude_from_analysis as "category_exclude?"
            FROM user_subscriptions s
            JOIN pockets p ON s.pocket_id = p.id
            LEFT JOIN categories c ON s.category_id = c.id
            WHERE s.id = $1 AND s.user_id = $2
            "#,
            id,
            user_id
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(AppError::NotFoundError("Subscription not found".to_string()))?;

        Ok(crate::schemas::UserSubscriptionDetail {
            id: row.id,
            name: row.name,
            description: row.description,
            amount: row.amount,
            basis: row.basis,
            billing_day: row.billing_day,
            billing_month: row.billing_month,
            next_charge_date: row.next_charge_date,
            is_active: row.is_active,
            pocket: PocketSummary {
                id: row.pocket_id,
                name: row.pocket_name,
                icon: row.pocket_icon,
            },
            category: row.category_id.map(|id| Category {
                id,
                name: row.category_name.unwrap_or_default(),
                is_income: row.category_is_income.unwrap_or(false),
                icon: row.category_icon.unwrap_or_default(),
                exclude_from_analysis: row.category_exclude.unwrap_or(false),
            }),
            created_at: row.created_at,
        })
    }

    pub async fn update(
        &self,
        id: Uuid,
        user_id: Uuid,
        name: Option<String>,
        description: Option<String>,
        amount: Option<Decimal>,
        basis: Option<String>,
        billing_day: Option<i32>,
        billing_month: Option<i32>,
        category_id: Option<i32>,
        is_active: Option<bool>,
        pocket_id: Option<Uuid>,
        next_charge_date: Option<NaiveDate>,
    ) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            UPDATE user_subscriptions
            SET 
                name = COALESCE($3, name),
                description = COALESCE($4, description),
                amount = COALESCE($5, amount),
                basis = COALESCE($6, basis),
                billing_day = COALESCE($7, billing_day),
                billing_month = COALESCE($8, billing_month),
                category_id = COALESCE($9, category_id),
                is_active = COALESCE($10, is_active),
                pocket_id = COALESCE($11, pocket_id),
                next_charge_date = COALESCE($12, next_charge_date),
                updated_at = NOW()
            WHERE id = $1 AND user_id = $2
            "#,
            id,
            user_id,
            name,
            description,
            amount,
            basis,
            billing_day,
            billing_month,
            category_id,
            is_active,
            pocket_id,
            next_charge_date
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn delete(&self, id: Uuid, user_id: Uuid) -> Result<u64, AppError> {
        let result = sqlx::query!(
            "DELETE FROM user_subscriptions WHERE id = $1 AND user_id = $2",
            id,
            user_id
        )
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected())
    }

    pub async fn get_due_subscriptions(
        &self,
    ) -> Result<Vec<crate::schemas::UserSubscriptionRow>, AppError> {
        let rows = sqlx::query_as!(
            crate::schemas::UserSubscriptionRow,
            r#"
            SELECT * FROM user_subscriptions 
            WHERE is_active = TRUE AND next_charge_date <= CURRENT_DATE
            "#
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn update_next_charge_date(
        &self,
        id: Uuid,
        next_date: NaiveDate,
    ) -> Result<(), AppError> {
        sqlx::query!(
            "UPDATE user_subscriptions SET next_charge_date = $1 WHERE id = $2",
            next_date,
            id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
