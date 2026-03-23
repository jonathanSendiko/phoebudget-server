use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

use crate::i18n;

/// Subscription limits based on plan tier
#[derive(Serialize, Debug, Clone)]
pub struct SubscriptionLimits {
    pub max_investments: Option<i32>,
    pub max_pockets: Option<i32>,
    pub history_days: Option<i32>,
    pub multi_currency: bool,
    pub pocket_transfers: bool,
    pub advanced_analytics: bool,
    pub data_export: bool,
}

/// Response for GET /auth/subscription
#[derive(Serialize, Debug)]
pub struct SubscriptionResponse {
    pub plan: String,
    pub status: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub limits: SubscriptionLimits,
}

impl SubscriptionResponse {
    pub fn localize(mut self) -> Self {
        self.plan = i18n::localize_plan(&self.plan);
        self.status = i18n::localize_status(&self.status);
        self
    }
}

/// Internal row type for subscription repository
#[derive(Debug, sqlx::FromRow)]
pub struct SubscriptionRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub plan: String,
    pub status: String,
    pub started_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub payment_provider: Option<String>,
    pub external_subscription_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::{SubscriptionLimits, SubscriptionResponse};
    use crate::i18n::{Locale, run_with_locale};

    #[tokio::test]
    async fn localize_translates_plan_and_status() {
        run_with_locale(Locale::Indonesian, async {
            let response = SubscriptionResponse {
                plan: "free".to_string(),
                status: "active".to_string(),
                expires_at: None,
                limits: SubscriptionLimits {
                    max_investments: Some(3),
                    max_pockets: Some(2),
                    history_days: Some(90),
                    multi_currency: false,
                    pocket_transfers: false,
                    advanced_analytics: false,
                    data_export: false,
                },
            }
            .localize();

            assert_eq!(response.plan, "gratis");
            assert_eq!(response.status, "aktif");
        })
        .await;
    }
}
