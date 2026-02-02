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

#[cfg(test)]
mod tests {
    use super::SubscriptionService;
    use crate::schemas::SubscriptionRow;
    use chrono::{DateTime, Utc};
    use uuid::Uuid;

    fn sample_subscription(plan: &str) -> SubscriptionRow {
        SubscriptionRow {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            plan: plan.to_string(),
            status: "active".to_string(),
            started_at: DateTime::<Utc>::from_timestamp(1_700_000_000, 0).unwrap(),
            expires_at: None,
            payment_provider: None,
            external_subscription_id: None,
        }
    }

    #[test]
    fn compute_limits_free_defaults() {
        let limits = SubscriptionService::compute_limits(&None);
        assert_eq!(limits.max_investments, Some(3));
        assert_eq!(limits.max_pockets, Some(2));
        assert_eq!(limits.history_days, Some(90));
        assert!(!limits.multi_currency);
        assert!(!limits.pocket_transfers);
        assert!(!limits.advanced_analytics);
        assert!(!limits.data_export);
    }

    #[test]
    fn compute_limits_premium_unlimited() {
        let sub = Some(sample_subscription("premium"));
        let limits = SubscriptionService::compute_limits(&sub);
        assert_eq!(limits.max_investments, None);
        assert_eq!(limits.max_pockets, None);
        assert_eq!(limits.history_days, None);
        assert!(limits.multi_currency);
        assert!(limits.pocket_transfers);
        assert!(limits.advanced_analytics);
        assert!(limits.data_export);
    }

    #[test]
    fn compute_limits_lifetime_unlimited() {
        let sub = Some(sample_subscription("lifetime"));
        let limits = SubscriptionService::compute_limits(&sub);
        assert_eq!(limits.max_investments, None);
        assert_eq!(limits.max_pockets, None);
        assert_eq!(limits.history_days, None);
        assert!(limits.multi_currency);
        assert!(limits.pocket_transfers);
        assert!(limits.advanced_analytics);
        assert!(limits.data_export);
    }
}
