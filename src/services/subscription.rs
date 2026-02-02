use uuid::Uuid;

use crate::error::AppError;
use crate::repository::SubscriptionRepository;
use crate::schemas::{SubscriptionLimits, SubscriptionResponse};

pub struct SubscriptionService {
    subscription_repo: SubscriptionRepository,
}

impl SubscriptionService {
    pub fn new(subscription_repo: SubscriptionRepository) -> Self {
        Self { subscription_repo }
    }

    pub async fn get_subscription(&self, user_id: Uuid) -> Result<SubscriptionResponse, AppError> {
        let sub = self.subscription_repo.get_by_user(user_id).await?;
        let limits = Self::compute_limits(&sub);

        Ok(SubscriptionResponse {
            plan: sub
                .as_ref()
                .map(|s| s.plan.clone())
                .unwrap_or_else(|| "free".to_string()),
            status: sub
                .as_ref()
                .map(|s| s.status.clone())
                .unwrap_or_else(|| "active".to_string()),
            expires_at: sub.as_ref().and_then(|s| s.expires_at),
            limits,
        })
    }

    pub fn compute_limits(sub: &Option<crate::schemas::SubscriptionRow>) -> SubscriptionLimits {
        let plan = sub.as_ref().map(|s| s.plan.as_str()).unwrap_or("free");

        match plan {
            "premium" | "lifetime" => SubscriptionLimits {
                max_investments: None,
                max_pockets: None,
                history_days: None,
                multi_currency: true,
                pocket_transfers: true,
                advanced_analytics: true,
                data_export: true,
            },
            _ => SubscriptionLimits {
                max_investments: Some(3),
                max_pockets: Some(2),
                history_days: Some(90),
                multi_currency: false,
                pocket_transfers: false,
                advanced_analytics: false,
                data_export: false,
            },
        }
    }
}
