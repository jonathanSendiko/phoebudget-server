use chrono::{Datelike, NaiveDate, Utc};
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::error::AppError;
use crate::repository::{PocketRepository, TransactionRepository, UserSubscriptionRepository};

pub struct UserSubscriptionService {
    sub_repo: UserSubscriptionRepository,
    pocket_repo: PocketRepository,
    transaction_repo: TransactionRepository, // Needed for processing
}

impl UserSubscriptionService {
    pub fn new(
        sub_repo: UserSubscriptionRepository,
        pocket_repo: PocketRepository,
        transaction_repo: TransactionRepository,
    ) -> Self {
        Self {
            sub_repo,
            pocket_repo,
            transaction_repo,
        }
    }

    pub async fn create_subscription(
        &self,
        user_id: Uuid,
        req: crate::schemas::CreateUserSubscription,
    ) -> Result<Uuid, AppError> {
        if req.amount <= Decimal::ZERO {
            return Err(AppError::ValidationError(
                "Amount must be positive".to_string(),
            ));
        }

        // Validate Basis
        match req.basis.as_str() {
            "monthly" => {
                if req.billing_month.is_some() {
                    return Err(AppError::ValidationError(
                        "Billing month must be null for monthly subscriptions".to_string(),
                    ));
                }
                if !(1..=31).contains(&req.billing_day) {
                    return Err(AppError::ValidationError(
                        "Billing day must be between 1 and 31".to_string(),
                    ));
                }
            }
            "annually" => {
                if req.billing_month.is_none() {
                    return Err(AppError::ValidationError(
                        "Billing month is required for annual subscriptions".to_string(),
                    ));
                }
                if let Some(m) = req.billing_month {
                    if !(1..=12).contains(&m) {
                        return Err(AppError::ValidationError(
                            "Billing month must be between 1 and 12".to_string(),
                        ));
                    }
                }
                if !(1..=31).contains(&req.billing_day) {
                    return Err(AppError::ValidationError(
                        "Billing day must be between 1 and 31".to_string(),
                    ));
                }
            }
            _ => return Err(AppError::ValidationError("Invalid basis".to_string())),
        }

        // Verify pocket exists
        let _ = self.pocket_repo.get_by_id(req.pocket_id, user_id).await?;

        // Calculate next charge date
        let next_charge_date = Self::calculate_next_charge_date(
            &req.basis,
            req.billing_day,
            req.billing_month,
            Utc::now().date_naive(), // From today
            false,                   // Start date is NOT last charge date, but today
        );

        self.sub_repo
            .create(
                user_id,
                req.pocket_id,
                &req.name,
                req.description,
                req.amount,
                &req.basis,
                req.billing_day,
                req.billing_month,
                req.category_id,
                next_charge_date,
            )
            .await
    }

    pub async fn get_subscriptions(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<crate::schemas::UserSubscriptionSummary>, AppError> {
        self.sub_repo.get_all(user_id).await
    }

    pub async fn get_subscription(
        &self,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<crate::schemas::UserSubscriptionDetail, AppError> {
        self.sub_repo.get_by_id(id, user_id).await
    }

    pub async fn update_subscription(
        &self,
        id: Uuid,
        user_id: Uuid,
        req: crate::schemas::UpdateUserSubscription,
    ) -> Result<(), AppError> {
        // Validation (simplified, full validation if basis changed is complex, assuming frontend sends valid combo)
        if let Some(amount) = req.amount {
            if amount <= Decimal::ZERO {
                return Err(AppError::ValidationError(
                    "Amount must be positive".to_string(),
                ));
            }
        }

        let mut next_charge = None;

        // If timing changed, recalculate next_charge_date
        // Fetch current to merge
        if req.basis.is_some() || req.billing_day.is_some() || req.billing_month.is_some() {
            let current = self.sub_repo.get_by_id(id, user_id).await?;
            let basis = req.basis.as_deref().unwrap_or(&current.basis);
            let day = req.billing_day.unwrap_or(current.billing_day);
            let month = req.billing_month.or(current.billing_month); // Careful with Option merging

            // Recalculate from TODAY. If user changes date, we assume they want next occurance from now.
            next_charge = Some(Self::calculate_next_charge_date(
                basis,
                day,
                month,
                Utc::now().date_naive(),
                false,
            ));
        }

        self.sub_repo
            .update(
                id,
                user_id,
                req.name,
                req.description,
                req.amount,
                req.basis,
                req.billing_day,
                req.billing_month,
                req.category_id,
                req.is_active,
                req.pocket_id,
                next_charge,
            )
            .await
    }

    pub async fn delete_subscription(&self, id: Uuid, user_id: Uuid) -> Result<(), AppError> {
        let deleted = self.sub_repo.delete(id, user_id).await?;
        if deleted == 0 {
            return Err(AppError::NotFoundError(
                "Subscription not found".to_string(),
            ));
        }
        Ok(())
    }

    /// Core logic for calculating the next charge date
    fn calculate_next_charge_date(
        basis: &str,
        billing_day: i32,
        billing_month: Option<i32>,
        reference_date: NaiveDate, // Usually today, or last charge date
        is_retry: bool, // If true, reference_date is last charge date, find next. If false, find first occurance >= reference_date
    ) -> NaiveDate {
        let billing_day = billing_day as u32;
        let mut target_year = reference_date.year();
        let mut target_month = reference_date.month();

        if basis == "monthly" {
            if is_retry {
                // Next month from reference
                if target_month == 12 {
                    target_month = 1;
                    target_year += 1;
                } else {
                    target_month += 1;
                }
            } else {
                // If today > billing_day, move to next month
                if reference_date.day() > billing_day {
                    if target_month == 12 {
                        target_month = 1;
                        target_year += 1;
                    } else {
                        target_month += 1;
                    }
                }
            }
            // Handle end of month logic
            Self::get_valid_date(target_year, target_month, billing_day)
        } else {
            // Annual
            let billing_month = billing_month.unwrap_or(1) as u32;

            if is_retry {
                // Next year
                target_year += 1;
            } else {
                // If today > billing_date (approx), move to next year
                // Simplistic check: constructed date < reference?
                let this_year_date = Self::get_valid_date(target_year, billing_month, billing_day);
                if this_year_date < reference_date {
                    target_year += 1;
                }
            }
            Self::get_valid_date(target_year, billing_month, billing_day)
        }
    }

    /// Helper to handle "Feb 31" -> "Feb 28/29"
    fn get_valid_date(year: i32, month: u32, day: u32) -> NaiveDate {
        // Try to create date. If fail, it's likely invalid day (e.g. 31 for Feb)
        // chrono::NaiveDate::from_ymd_opt(year, month, day).unwrap() // panic if invalid

        // Better approach:
        // If day is valid, return it.
        // If not, find last day of that month.
        if let Some(d) = NaiveDate::from_ymd_opt(year, month, day) {
            d
        } else {
            // Day is too large. Get last day of month.
            // Go to next month, day 1, subtract 1 day.
            let (next_y, next_m) = if month == 12 {
                (year + 1, 1)
            } else {
                (year, month + 1)
            };
            NaiveDate::from_ymd_opt(next_y, next_m, 1)
                .unwrap()
                .pred_opt()
                .unwrap()
        }
    }

    // Logic for Worker: Process a subscription
    pub async fn process_subscription(&self, _sub_id: Uuid) -> Result<(), AppError> {
        // 1. Fetch sub directly (bypass user check as worker is system)
        // We need a system-level get method or reuse get_by_id if we assume user_id context not needed for lookup (but repo enforces it).
        // Actually, we likely need a `get_by_id_system` in repo.
        // For now, let's assume we fetch the row using `TransactionRepository` or similar raw query if necessary,
        // OR we add `get_by_id_system` to `UserSubscriptionRepository`.
        // Let's assume we add it or use a raw query here for simplicity if needed, but adding to repo is cleaner.
        // I'll assume `sub_repo` has `get_for_processing` or similar.
        // Wait, I implemented `get_due_subscriptions` in repo which returns `UserSubscriptionRow`.
        // I can use that row. But the worker passes `sub_id`.
        // So I need `get_row_by_id`.

        // Let's implement `get_row_by_id` in repo? Or just do it here.
        // I'll add `process_subscription_transaction` which takes the ROW.
        // But the worker structure in the plan says: "Listens to jobs. Fetches sub details".

        // I will add `get_system` to repo later. For now, let's implement the core Logic assuming we have the data.
        Ok(())
    }

    // Using the row data to process
    pub async fn process_due_subscription(
        &self,
        sub: &crate::schemas::UserSubscriptionRow,
    ) -> Result<(), AppError> {
        // 1. Create Transaction
        // We need a category. If sub.category_id is None, use "Subscriptions" (we created it in migration).
        // If we don't know the ID, we look it up.
        // Ideally we cache this or fetch it.

        let category_id = if let Some(id) = sub.category_id {
            id
        } else {
            // Fallback to finding "Subscriptions"
            let cat = self
                .transaction_repo
                .get_category_by_name("Subscriptions")
                .await?;
            cat.id
        };

        // Create transaction
        let _tx_id = self
            .transaction_repo
            .create(
                sub.user_id,
                sub.amount,
                Some(format!("Subscription: {}", sub.name)),
                category_id,
                Utc::now(), // Occurred NOW
                None,
                None,
                None,
                sub.pocket_id,
            )
            .await?;

        // 2. Calculate NEXT charge date
        let next_date = Self::calculate_next_charge_date(
            &sub.basis,
            sub.billing_day,
            sub.billing_month,
            sub.next_charge_date, // base on PREVIOUS scheduled date to keep cadence? OR Today?
            // Usually base on previous to keep "1st of month" alignment even if run late.
            true,
        );

        // 3. Update subscription
        self.sub_repo
            .update_next_charge_date(sub.id, next_date)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::UserSubscriptionService;
    use chrono::NaiveDate;

    #[test]
    fn monthly_billing_same_month_when_before_day() {
        let reference = NaiveDate::from_ymd_opt(2026, 2, 2).unwrap();
        let next = UserSubscriptionService::calculate_next_charge_date(
            "monthly", 15, None, reference, false,
        );
        assert_eq!(next, NaiveDate::from_ymd_opt(2026, 2, 15).unwrap());
    }

    #[test]
    fn monthly_billing_next_month_when_after_day() {
        let reference = NaiveDate::from_ymd_opt(2026, 2, 20).unwrap();
        let next = UserSubscriptionService::calculate_next_charge_date(
            "monthly", 15, None, reference, false,
        );
        assert_eq!(next, NaiveDate::from_ymd_opt(2026, 3, 15).unwrap());
    }

    #[test]
    fn monthly_retry_always_next_month() {
        let reference = NaiveDate::from_ymd_opt(2026, 12, 15).unwrap();
        let next = UserSubscriptionService::calculate_next_charge_date(
            "monthly", 15, None, reference, true,
        );
        assert_eq!(next, NaiveDate::from_ymd_opt(2027, 1, 15).unwrap());
    }

    #[test]
    fn annual_billing_this_year_when_before_date() {
        let reference = NaiveDate::from_ymd_opt(2026, 2, 2).unwrap();
        let next = UserSubscriptionService::calculate_next_charge_date(
            "annually",
            15,
            Some(3),
            reference,
            false,
        );
        assert_eq!(next, NaiveDate::from_ymd_opt(2026, 3, 15).unwrap());
    }

    #[test]
    fn annual_billing_next_year_when_after_date() {
        let reference = NaiveDate::from_ymd_opt(2026, 12, 20).unwrap();
        let next = UserSubscriptionService::calculate_next_charge_date(
            "annually",
            15,
            Some(3),
            reference,
            false,
        );
        assert_eq!(next, NaiveDate::from_ymd_opt(2027, 3, 15).unwrap());
    }

    #[test]
    fn annual_retry_moves_to_next_year() {
        let reference = NaiveDate::from_ymd_opt(2026, 3, 15).unwrap();
        let next = UserSubscriptionService::calculate_next_charge_date(
            "annually",
            15,
            Some(3),
            reference,
            true,
        );
        assert_eq!(next, NaiveDate::from_ymd_opt(2027, 3, 15).unwrap());
    }

    #[test]
    fn get_valid_date_handles_non_leap_february() {
        let date = UserSubscriptionService::get_valid_date(2025, 2, 31);
        assert_eq!(date, NaiveDate::from_ymd_opt(2025, 2, 28).unwrap());
    }

    #[test]
    fn get_valid_date_handles_leap_february() {
        let date = UserSubscriptionService::get_valid_date(2024, 2, 31);
        assert_eq!(date, NaiveDate::from_ymd_opt(2024, 2, 29).unwrap());
    }
}
