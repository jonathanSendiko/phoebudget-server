use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::schemas::CreatePortfolioItem;

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

    pub async fn count_by_user(&self, user_id: Uuid) -> Result<i64, AppError> {
        let result = sqlx::query!(
            r#"SELECT COUNT(*) as "count!" FROM portfolio WHERE user_id = $1"#,
            user_id
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(result.count)
    }
}
