use crate::error::AppError;
use crate::schemas::{
    Category, CategorySummary, CreatePortfolioItem, Pocket, PocketSummary, Transaction,
    TransactionDetail, User, UserProfile,
};
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
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Transaction>, AppError> {
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
              AND ($1::timestamptz IS NULL OR t.occurred_at >= $1)
              AND ($2::timestamptz IS NULL OR t.occurred_at <= $2)
              AND ($4::uuid IS NULL OR t.pocket_id = $4)
            ORDER BY t.occurred_at DESC
            LIMIT $5 OFFSET $6
            "#,
            start_date,
            end_date,
            user_id,
            pocket_id,
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
                icon: row.pocket_icon.unwrap_or_else(|| "account_balance_wallet".to_string()),
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
    ) -> Result<i64, AppError> {
        let result = sqlx::query!(
            r#"
            SELECT COUNT(*) as "count!"
            FROM transactions t
            WHERE t.user_id = $3 
              AND ($1::timestamptz IS NULL OR t.occurred_at >= $1)
              AND ($2::timestamptz IS NULL OR t.occurred_at <= $2)
              AND ($4::uuid IS NULL OR t.pocket_id = $4)
            "#,
            start_date,
            end_date,
            user_id,
            pocket_id
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
            WHERE t.id = $1 AND t.user_id = $2
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
            SELECT COALESCE(SUM(p.quantity * a.current_price), 0) as "total_invested!"
            FROM portfolio p
            JOIN assets a ON p.ticker = a.ticker
            WHERE p.user_id = $1
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

    pub async fn update_asset_price(
        &self,
        ticker: &str,
        price: Decimal,
        currency: &str,
    ) -> Result<(), AppError> {
        sqlx::query!(
            "UPDATE assets SET current_price = $1, currency = $2, last_updated = NOW() WHERE ticker = $3",
            price,
            currency,
            ticker
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn update_asset_icon(&self, ticker: &str, icon_url: &str) -> Result<(), AppError> {
        sqlx::query!(
            "UPDATE assets SET icon_url = $1 WHERE ticker = $2",
            icon_url,
            ticker
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_all_assets(&self) -> Result<Vec<crate::schemas::Asset>, AppError> {
        let rows = sqlx::query_as!(
            crate::schemas::Asset,
            r#"
            SELECT 
                ticker, 
                name, 
                asset_type,
                api_ticker,
                source,
                current_price,
                currency,
                icon_url
            FROM assets
            ORDER BY name
            "#
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn add_item(&self, user_id: Uuid, item: CreatePortfolioItem) -> Result<(), AppError> {
        // Ensure asset exists (in case user passes custom ticker not in DB)
        // For MVP, if ticker doesn't exist, we error out or insert basic one.
        // User wants "predetermined assets", so strict check is better,
        // BUT for now let's leniently insert if missing (defaulting source to YAHOO) or error.
        // Given the requirement "only allow user to use predetermined assets", we should probably Fail if not found.
        // But to keep it simple and safe:
        let asset_exists = sqlx::query!("SELECT ticker FROM assets WHERE ticker = $1", item.ticker)
            .fetch_optional(&self.pool)
            .await?
            .is_some();

        if !asset_exists {
            return Err(AppError::ValidationError(format!(
                "Asset '{}' not supported",
                item.ticker
            )));
        }

        sqlx::query!(
            "INSERT INTO portfolio (user_id, ticker, quantity, avg_buy_price) VALUES ($1, $2, $3, $4)",
            user_id,
            item.ticker,
            item.quantity,
            item.avg_buy_price
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_all_joined(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<crate::schemas::PortfolioJoinedRow>, AppError> {
        // Explicitly define the record type to satisfy the compiler
        struct Row {
            ticker: Option<String>,
            name: String,
            quantity: Decimal,
            avg_buy_price: Decimal,
            current_price: Option<Decimal>,
            source: Option<String>,
            api_ticker: Option<String>,
            currency: Option<String>,
            icon_url: Option<String>,
        }

        let rows = sqlx::query_as!(
            Row,
            r#"
            SELECT 
                p.ticker, 
                a.name, 
                p.quantity, 
                p.avg_buy_price, 
                a.current_price,
                a.source,
                a.api_ticker,
                a.currency,
                a.icon_url
            FROM portfolio p
            LEFT JOIN assets a ON p.ticker = a.ticker
            WHERE p.user_id = $1
            "#,
            user_id
        )
        .fetch_all(&self.pool)
        .await?;

        let result = rows
            .into_iter()
            .map(|r| crate::schemas::PortfolioJoinedRow {
                ticker: r.ticker.unwrap_or_default(),
                name: r.name,
                quantity: r.quantity,
                avg_buy_price: r.avg_buy_price,
                current_price: r.current_price.unwrap_or(Decimal::ZERO),
                source: r.source,
                api_ticker: r.api_ticker,
                currency: r.currency,
                icon_url: r.icon_url,
            })
            .collect();

        Ok(result)
    }

    pub async fn get_asset(&self, ticker: &str) -> Result<Option<crate::schemas::Asset>, AppError> {
        let asset = sqlx::query_as!(
            crate::schemas::Asset,
            r#"
            SELECT 
                ticker, 
                name, 
                asset_type,
                api_ticker,
                source,
                currency,
                current_price,
                icon_url
            FROM assets
            WHERE ticker = $1
            "#,
            ticker
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(asset)
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
                avg_buy_price = COALESCE($4, avg_buy_price)
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

    pub async fn get_available_currencies(&self) -> Result<Vec<String>, AppError> {
        let rows = sqlx::query!("SELECT code FROM currencies ORDER BY code")
            .fetch_all(&self.pool)
            .await?;
        Ok(rows.into_iter().map(|r| r.code).collect())
    }
}
