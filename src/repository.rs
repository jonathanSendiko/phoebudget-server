use crate::error::AppError;
use crate::schemas::{CategorySummary, CreatePortfolioItem, Transaction, User, UserProfile};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>, AppError> {
        let user = sqlx::query_as!(
            User,
            "SELECT id, username, email, password_hash, created_at FROM users WHERE email = $1",
            email
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(user)
    }

    pub async fn check_exists(&self, email: &str, username: &str) -> Result<bool, AppError> {
        let existing = sqlx::query!(
            "SELECT id FROM users WHERE email = $1 OR username = $2",
            email,
            username
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(existing.is_some())
    }

    pub async fn create(
        &self,
        username: &str,
        email: &str,
        password_hash: &str,
    ) -> Result<Uuid, AppError> {
        let user_id = sqlx::query_scalar!(
            "INSERT INTO users (username, email, password_hash) VALUES ($1, $2, $3) RETURNING id",
            username,
            email,
            password_hash
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(user_id)
    }
    pub async fn get_profile(&self, user_id: Uuid) -> Result<UserProfile, AppError> {
        let profile = sqlx::query_as!(
            UserProfile,
            r#"
            SELECT 
                u.id, 
                u.username, 
                u.email, 
                COALESCE(s.base_currency, 'SGD') as "base_currency!",
                u.created_at as "joined_at!"
            FROM users u
            LEFT JOIN user_settings s ON u.id = s.user_id
            WHERE u.id = $1
            "#,
            user_id
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(AppError::NotFoundError("User not found".to_string()))?;

        Ok(profile)
    }
}

pub struct TransactionRepository {
    pool: PgPool,
}

impl TransactionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        user_id: Uuid,
        amount: Decimal,
        description: String,
        category_id: i32,
        occurred_at: DateTime<Utc>,
    ) -> Result<Uuid, AppError> {
        let id = sqlx::query_scalar!(
            r#"
            INSERT INTO transactions (amount, description, category_id, user_id, occurred_at)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id
            "#,
            amount,
            description,
            category_id,
            user_id,
            occurred_at
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(id)
    }

    pub async fn find_by_user_and_date(
        &self,
        user_id: Uuid,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<Vec<Transaction>, AppError> {
        let rows = sqlx::query_as!(
            Transaction,
            r#"
            SELECT id, amount, description, category_id, occurred_at, created_at 
            FROM transactions
            WHERE user_id = $3 AND occurred_at BETWEEN $1 AND $2
            ORDER BY occurred_at DESC
            "#,
            start_date,
            end_date,
            user_id
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
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
                COALESCE(SUM(t.amount), 0) as "total!"
            FROM transactions t
            JOIN categories c ON t.category_id = c.id
            WHERE t.user_id = $3 AND t.occurred_at BETWEEN $1 AND $2
            GROUP BY c.name
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
                occurred_at = COALESCE($6, occurred_at)
            WHERE id = $1 AND user_id = $2
            "#,
            id,
            user_id,
            amount,
            description,
            category_id,
            occurred_at
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn delete(&self, id: Uuid, user_id: Uuid) -> Result<u64, AppError> {
        let result = sqlx::query!(
            "DELETE FROM transactions WHERE id = $1 AND user_id = $2",
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
            WHERE t.user_id = $1
            "#,
            user_id
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(result.net_cash)
    }
}

pub struct PortfolioRepository {
    pool: PgPool,
}

impl PortfolioRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_total_invested(&self, user_id: Uuid) -> Result<Decimal, AppError> {
        let result = sqlx::query!(
            r#"
            SELECT COALESCE(SUM(quantity * current_price), 0) as "total_invested!"
            FROM portfolio
            WHERE user_id = $1
            "#,
            user_id
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(result.total_invested)
    }

    pub async fn get_tickers(&self, user_id: Uuid) -> Result<Vec<String>, AppError> {
        let rows = sqlx::query!(
            "SELECT DISTINCT ticker FROM portfolio WHERE user_id = $1",
            user_id
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().filter_map(|r| r.ticker).collect())
    }

    pub async fn update_price(&self, ticker: &str, price: Decimal) -> Result<(), AppError> {
        sqlx::query!(
            "UPDATE portfolio SET current_price = $1, last_updated = NOW() WHERE ticker = $2",
            price,
            ticker
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn ensure_asset_exists(&self, ticker: &str) -> Result<(), AppError> {
        sqlx::query!(
            "INSERT INTO assets (ticker, name, asset_type) VALUES ($1, $1, 'Stock') ON CONFLICT (ticker) DO NOTHING",
            ticker
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn add_item(
        &self,
        user_id: Uuid,
        item: CreatePortfolioItem,
        current_price: Decimal,
    ) -> Result<(), AppError> {
        sqlx::query!(
            "INSERT INTO portfolio (user_id, ticker, quantity, avg_buy_price, current_price) VALUES ($1, $2, $3, $4, $5)",
            user_id,
            item.ticker,
            item.quantity,
            item.avg_buy_price,
            current_price
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
    pub async fn get_all_joined(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<(String, String, Decimal, Decimal, Decimal)>, AppError> {
        let rows = sqlx::query!(
            r#"
            SELECT 
                p.ticker, 
                a.name, 
                p.quantity, 
                p.avg_buy_price, 
                p.current_price
            FROM portfolio p
            LEFT JOIN assets a ON p.ticker = a.ticker
            WHERE p.user_id = $1
            "#,
            user_id
        )
        .fetch_all(&self.pool)
        .await?;

        // Map results to a tuple.
        let result = rows
            .into_iter()
            .map(|r| {
                (
                    r.ticker.unwrap_or_else(String::new),
                    r.name,
                    r.quantity,
                    r.avg_buy_price,
                    r.current_price.unwrap_or(Decimal::ZERO),
                )
            })
            .collect();

        Ok(result)
    }
    pub async fn delete(&self, user_id: Uuid, ticker: &str) -> Result<u64, AppError> {
        let result = sqlx::query!(
            "DELETE FROM portfolio WHERE user_id = $1 AND ticker = $2",
            user_id,
            ticker
        )
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected())
    }

    pub async fn update(
        &self,
        user_id: Uuid,
        ticker: &str,
        quantity: Option<Decimal>,
        avg_buy_price: Option<Decimal>,
    ) -> Result<(), AppError> {
        // Simple dynamic query via COALESCE
        // Since we're dealing with "if null dont change", COALESCE works if we pass NULL for None.
        sqlx::query!(
            r#"
            UPDATE portfolio 
            SET 
                quantity = COALESCE($3, quantity), 
                avg_buy_price = COALESCE($4, avg_buy_price),
                last_updated = NOW()
            WHERE user_id = $1 AND ticker = $2
            "#,
            user_id,
            ticker,
            quantity,
            avg_buy_price
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

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
}
