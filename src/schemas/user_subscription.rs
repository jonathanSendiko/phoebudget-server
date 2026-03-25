use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::i18n;

use super::common::round_currency;
use super::pocket::PocketSummary;
use super::transaction::Category;

// --- User Subscriptions DTOs ---

#[derive(Deserialize, Debug)]
pub struct CreateUserSubscription {
    pub name: String,
    pub description: Option<String>,
    pub amount: Decimal,
    pub basis: String, // 'monthly' or 'annually'
    pub billing_day: i32,
    pub billing_month: Option<i32>,
    pub category_id: Option<i32>,
    pub pocket_id: Uuid,
}

#[derive(Deserialize, Debug)]
pub struct UpdateUserSubscription {
    pub name: Option<String>,
    pub description: Option<String>,
    pub amount: Option<Decimal>,
    pub basis: Option<String>,
    pub billing_day: Option<i32>,
    pub billing_month: Option<i32>,
    pub category_id: Option<i32>,
    pub is_active: Option<bool>,
    // pocket_id cannot be changed easily as it affects future transactions? allowing it.
    pub pocket_id: Option<Uuid>,
}

#[derive(Serialize, Debug)]
pub struct UserSubscriptionSummary {
    pub id: Uuid,
    pub name: String,
    #[serde(serialize_with = "round_currency")]
    pub amount: Decimal,
    pub basis: String,
    pub next_charge_date: NaiveDate,
    pub is_active: bool,
    pub icon: String, // From category
}

impl UserSubscriptionSummary {
    pub fn localize(mut self) -> Self {
        self.basis = i18n::localize_basis(&self.basis);
        self
    }
}

#[derive(Serialize, Debug)]
pub struct UserSubscriptionDetail {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    #[serde(serialize_with = "round_currency")]
    pub amount: Decimal,
    pub basis: String,
    pub billing_day: i32,
    pub billing_month: Option<i32>,
    pub next_charge_date: NaiveDate,
    pub is_active: bool,
    pub pocket: PocketSummary,
    pub category: Option<Category>,
    pub created_at: Option<DateTime<Utc>>,
}

impl UserSubscriptionDetail {
    pub fn localize(mut self) -> Self {
        self.basis = i18n::localize_basis(&self.basis);
        self
    }
}

#[derive(Serialize)]
pub struct UserSubscriptionId {
    pub id: Uuid,
}

#[derive(Debug, sqlx::FromRow)]
pub struct UserSubscriptionRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub pocket_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub amount: Decimal,
    pub basis: String,
    pub billing_day: i32,
    pub billing_month: Option<i32>,
    pub category_id: Option<i32>,
    pub is_active: Option<bool>,
    pub next_charge_date: NaiveDate,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use super::{UserSubscriptionDetail, UserSubscriptionSummary};
    use crate::i18n::{Locale, run_with_locale};
    use crate::schemas::{Category, PocketSummary};
    use chrono::NaiveDate;
    use rust_decimal::Decimal;
    use uuid::Uuid;

    #[tokio::test]
    async fn localize_translates_basis_only() {
        run_with_locale(Locale::Indonesian, async {
            let summary = UserSubscriptionSummary {
                id: Uuid::new_v4(),
                name: "Netflix".to_string(),
                amount: Decimal::new(1599, 2),
                basis: "monthly".to_string(),
                next_charge_date: NaiveDate::from_ymd_opt(2026, 3, 31).unwrap(),
                is_active: true,
                icon: "subscriptions".to_string(),
            }
            .localize();
            assert_eq!(summary.basis, "bulanan");

            let detail = UserSubscriptionDetail {
                id: Uuid::new_v4(),
                name: "Netflix".to_string(),
                description: Some("video".to_string()),
                amount: Decimal::new(1599, 2),
                basis: "annually".to_string(),
                billing_day: 31,
                billing_month: Some(12),
                next_charge_date: NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
                is_active: true,
                pocket: PocketSummary {
                    id: Uuid::new_v4(),
                    name: "Main".to_string(),
                    icon: "account_balance".to_string(),
                },
                category: Some(Category {
                    id: 1,
                    name: "Subscriptions".to_string(),
                    is_income: false,
                    icon: "subscriptions".to_string(),
                    exclude_from_analysis: false,
                }),
                created_at: None,
            }
            .localize();

            assert_eq!(detail.basis, "tahunan");
            assert_eq!(detail.pocket.name, "Main");
            assert_eq!(detail.category.unwrap().name, "Subscriptions");
        })
        .await;
    }
}
