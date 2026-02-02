use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::schemas::{Category, CategorySummary, PocketSummary, Transaction, TransactionDetail};

pub struct TransactionRepository {
    pool: PgPool,
}

impl TransactionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_all_categories(&self) -> Result<Vec<Category>, AppError> {
        let categories = sqlx::query_as!(
            Category,
            r#"
            SELECT 
                id, 
                name, 
                COALESCE(is_income, FALSE) as "is_income!", 
                COALESCE(icon, 'help_outline') as "icon!",
                COALESCE(exclude_from_analysis, FALSE) as "exclude_from_analysis!"
            FROM categories
            ORDER BY name ASC
            "#
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(categories)
    }

    pub async fn get_category_by_name(&self, name: &str) -> Result<Category, AppError> {
        let category = sqlx::query_as!(
            Category,
            r#"
            SELECT 
                id, 
                name, 
                COALESCE(is_income, FALSE) as "is_income!", 
                COALESCE(icon, 'help_outline') as "icon!",
                COALESCE(exclude_from_analysis, FALSE) as "exclude_from_analysis!"
            FROM categories
            WHERE name = $1
            "#,
            name
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(AppError::NotFoundError(format!(
            "Category '{}' not found",
            name
        )))?;
        Ok(category)
    }

    pub async fn create(
        &self,
        user_id: Uuid,
        amount: Decimal,
        description: Option<String>,
        category_id: i32,
        occurred_at: DateTime<Utc>,
        original_currency: Option<String>,
        original_amount: Option<Decimal>,
        exchange_rate: Option<Decimal>,
        pocket_id: Uuid,
    ) -> Result<Uuid, AppError> {
        let id = sqlx::query_scalar!(
            r#"
            INSERT INTO transactions (
                amount, description, category_id, user_id, occurred_at,
                original_currency, original_amount, exchange_rate, pocket_id
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id
            "#,
            amount,
            description,
            category_id,
            user_id,
            occurred_at,
            original_currency,
            original_amount,
            exchange_rate,
            pocket_id
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(id)
    }

    pub async fn find_by_user_and_date(
        &self,
        user_id: Uuid,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
        pocket_id: Option<Uuid>,
        search: Option<String>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Transaction>, AppError> {
        // Prepare search pattern for ILIKE (case-insensitive)
        let search_pattern = search.map(|s| format!("%{}%", s));

        let transactions = sqlx::query!(
            r#"
            SELECT 
                t.id, t.amount, t.description, t.category_id, t.occurred_at, t.created_at,
                c.name as "category_name?", c.icon as category_icon, COALESCE(c.is_income, FALSE) as "category_is_income!",
                COALESCE(c.exclude_from_analysis, FALSE) as "category_exclude!",
                p.id as "pocket_id?", p.name as "pocket_name?", p.icon as "pocket_icon?"
            FROM transactions t
            LEFT JOIN categories c ON t.category_id = c.id
            LEFT JOIN pockets p ON t.pocket_id = p.id
            WHERE t.user_id = $3 
              AND t.deleted_at IS NULL
              AND ($1::timestamptz IS NULL OR t.occurred_at >= $1)
              AND ($2::timestamptz IS NULL OR t.occurred_at <= $2)
              AND ($4::uuid IS NULL OR t.pocket_id = $4)
              AND ($5::text IS NULL OR t.description ILIKE $5)
            ORDER BY t.occurred_at DESC
            LIMIT $6 OFFSET $7
            "#,
            start_date,
            end_date,
            user_id,
            pocket_id,
            search_pattern,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(|row| Transaction {
            id: row.id,
            amount: row.amount,
            description: row.description,
            category: row.category_id.map(|id| Category {
                id,
                name: row.category_name.unwrap_or_default(),
                is_income: row.category_is_income,
                icon: row.category_icon.unwrap_or_else(|| "help_outline".to_string()),
                exclude_from_analysis: row.category_exclude,
            }),
            pocket: row.pocket_id.map(|id| PocketSummary {
                id,
                name: row.pocket_name.unwrap_or_default(),
                icon: row
                    .pocket_icon
                    .unwrap_or_else(|| "account_balance_wallet".to_string()),
            }),
            occurred_at: row.occurred_at,
            created_at: row.created_at,
        })
        .collect();

        Ok(transactions)
    }

    pub async fn count_by_user_and_date(
        &self,
        user_id: Uuid,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
        pocket_id: Option<Uuid>,
        search: Option<String>,
    ) -> Result<i64, AppError> {
        // Prepare search pattern for ILIKE (case-insensitive)
        let search_pattern = search.map(|s| format!("%{}%", s));

        let result = sqlx::query!(
            r#"
            SELECT COUNT(*) as "count!"
            FROM transactions t
            WHERE t.user_id = $3 
              AND t.deleted_at IS NULL
              AND ($1::timestamptz IS NULL OR t.occurred_at >= $1)
              AND ($2::timestamptz IS NULL OR t.occurred_at <= $2)
              AND ($4::uuid IS NULL OR t.pocket_id = $4)
              AND ($5::text IS NULL OR t.description ILIKE $5)
            "#,
            start_date,
            end_date,
            user_id,
            pocket_id,
            search_pattern
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(result.count)
    }

    pub async fn get_transaction(
        &self,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<TransactionDetail, AppError> {
        let row = sqlx::query!(
            r#"
            SELECT 
                t.id, t.amount, t.description, t.category_id, t.occurred_at, t.created_at,
                t.original_currency, t.original_amount, t.exchange_rate,
                c.name as "category_name?", c.icon as category_icon, 
                COALESCE(c.is_income, FALSE) as "category_is_income!",
                COALESCE(c.exclude_from_analysis, FALSE) as "category_exclude!"
            FROM transactions t
            LEFT JOIN categories c ON t.category_id = c.id
            WHERE t.id = $1 AND t.user_id = $2 AND t.deleted_at IS NULL
            "#,
            id,
            user_id
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(AppError::NotFoundError("Transaction not found".to_string()))?;

        Ok(TransactionDetail {
            id: row.id,
            amount: row.amount,
            description: row.description,
            category: row.category_id.map(|id| Category {
                id,
                name: row.category_name.unwrap_or_default(),
                is_income: row.category_is_income,
                icon: row
                    .category_icon
                    .unwrap_or_else(|| "help_outline".to_string()),
                exclude_from_analysis: row.category_exclude,
            }),
            occurred_at: row.occurred_at,
            created_at: row.created_at,
            original_currency: row.original_currency,
            original_amount: row.original_amount,
            exchange_rate: row.exchange_rate,
        })
    }

    pub async fn get_spending_analysis(
        &self,
        user_id: Uuid,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<Vec<CategorySummary>, AppError> {
        let rows = sqlx::query_as!(
            CategorySummary,
            r#"
            SELECT 
                c.name as category, 
                COALESCE(SUM(t.amount), 0) as "total!",
                COALESCE(c.is_income, FALSE) as "is_income!",
                COALESCE(c.icon, 'help_outline') as "icon!"
            FROM transactions t
            JOIN categories c ON t.category_id = c.id
            WHERE t.user_id = $3 
              AND t.occurred_at BETWEEN $1 AND $2
              AND t.deleted_at IS NULL
              AND (c.exclude_from_analysis = FALSE OR c.exclude_from_analysis IS NULL)
            GROUP BY c.name, c.is_income, c.icon
            ORDER BY 2 DESC
            "#,
            start_date,
            end_date,
            user_id
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn update(
        &self,
        id: Uuid,
        user_id: Uuid,
        amount: Option<Decimal>,
        description: Option<String>,
        category_id: Option<i32>,
        occurred_at: Option<DateTime<Utc>>,
        original_currency: Option<String>,
        original_amount: Option<Decimal>,
        exchange_rate: Option<Decimal>,
    ) -> Result<(), AppError> {
        // Build dynamic query
        // simple way:
        sqlx::query!(
            r#"
            UPDATE transactions 
            SET 
                amount = COALESCE($3, amount),
                description = COALESCE($4, description),
                category_id = COALESCE($5, category_id),
                occurred_at = COALESCE($6, occurred_at),
                original_currency = COALESCE($7, original_currency),
                original_amount = COALESCE($8, original_amount),
                exchange_rate = COALESCE($9, exchange_rate)
            WHERE id = $1 AND user_id = $2
            "#,
            id,
            user_id,
            amount,
            description,
            category_id,
            occurred_at,
            original_currency,
            original_amount,
            exchange_rate
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn delete(&self, id: Uuid, user_id: Uuid) -> Result<u64, AppError> {
        let result = sqlx::query!(
            "UPDATE transactions SET deleted_at = NOW() WHERE id = $1 AND user_id = $2 AND deleted_at IS NULL",
            id,
            user_id
        )
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected())
    }

    pub async fn restore(&self, id: Uuid, user_id: Uuid) -> Result<u64, AppError> {
        let result = sqlx::query!(
            "UPDATE transactions SET deleted_at = NULL WHERE id = $1 AND user_id = $2",
            id,
            user_id
        )
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected())
    }

    pub async fn get_net_cash(&self, user_id: Uuid) -> Result<Decimal, AppError> {
        let result = sqlx::query!(
            r#"
            SELECT 
                COALESCE(SUM(
                    CASE WHEN c.is_income THEN t.amount 
                    ELSE -t.amount 
                    END
                ), 0) as "net_cash!"
            FROM transactions t
            JOIN categories c ON t.category_id = c.id
            WHERE t.user_id = $1 AND t.deleted_at IS NULL
            "#,
            user_id
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(result.net_cash)
    }

    pub async fn get_pocket_balance(
        &self,
        user_id: Uuid,
        pocket_id: Uuid,
    ) -> Result<Decimal, AppError> {
        let result = sqlx::query!(
            r#"
            SELECT 
                COALESCE(SUM(
                    CASE WHEN c.is_income THEN t.amount 
                    ELSE -t.amount 
                    END
                ), 0) as "balance!"
            FROM transactions t
            JOIN categories c ON t.category_id = c.id
            WHERE t.user_id = $1 AND t.pocket_id = $2 AND t.deleted_at IS NULL
            "#,
            user_id,
            pocket_id
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(result.balance)
    }
}
