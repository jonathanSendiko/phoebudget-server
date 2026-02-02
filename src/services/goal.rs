use rust_decimal::Decimal;
use uuid::Uuid;

use crate::error::AppError;
use crate::repository::{GoalEntryRepository, GoalRepository, PocketRepository};

pub struct GoalService {
    goal_repo: GoalRepository,
    entry_repo: GoalEntryRepository,
    pocket_repo: PocketRepository,
}

impl GoalService {
    pub fn new(
        goal_repo: GoalRepository,
        entry_repo: GoalEntryRepository,
        pocket_repo: PocketRepository,
    ) -> Self {
        Self {
            goal_repo,
            entry_repo,
            pocket_repo,
        }
    }

    pub async fn create_goal(
        &self,
        user_id: Uuid,
        req: crate::schemas::CreateGoal,
    ) -> Result<Uuid, AppError> {
        if req.target_amount <= Decimal::ZERO {
            return Err(AppError::ValidationError(
                "Target amount must be positive".to_string(),
            ));
        }

        if req.name.trim().is_empty() {
            return Err(AppError::ValidationError(
                "Goal name cannot be empty".to_string(),
            ));
        }

        // Verify pocket exists and belongs to user
        let _ = self.pocket_repo.get_by_id(req.pocket_id, user_id).await?;

        self.goal_repo
            .create(
                user_id,
                req.pocket_id,
                &req.name,
                req.description,
                req.target_amount,
                req.current_amount,
                req.icon,
            )
            .await
    }

    pub async fn get_goals(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<crate::schemas::GoalSummary>, AppError> {
        self.goal_repo.get_all(user_id).await
    }

    pub async fn get_goal(
        &self,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<crate::schemas::GoalDetail, AppError> {
        self.goal_repo.get_by_id(id, user_id).await
    }

    pub async fn update_goal(
        &self,
        id: Uuid,
        user_id: Uuid,
        req: crate::schemas::UpdateGoal,
    ) -> Result<(), AppError> {
        if let Some(target) = req.target_amount {
            if target <= Decimal::ZERO {
                return Err(AppError::ValidationError(
                    "Target amount must be positive".to_string(),
                ));
            }
        }

        self.goal_repo
            .update(
                id,
                user_id,
                req.name,
                req.description,
                req.target_amount,
                req.current_amount,
                req.icon,
            )
            .await
    }

    pub async fn delete_goal(&self, id: Uuid, user_id: Uuid) -> Result<(), AppError> {
        let deleted = self.goal_repo.delete(id, user_id).await?;
        if deleted == 0 {
            return Err(AppError::NotFoundError("Goal not found".to_string()));
        }
        Ok(())
    }

    pub async fn create_goal_entry(
        &self,
        goal_id: Uuid,
        user_id: Uuid,
        req: crate::schemas::CreateGoalEntry,
    ) -> Result<Uuid, AppError> {
        // 1. Verify goal ownership and get current amount
        let goal = self.goal_repo.get_by_id(goal_id, user_id).await?;

        // 2. Create entry
        let entry_id = self
            .entry_repo
            .create(goal_id, req.amount, req.description, req.date)
            .await?;

        // 3. Update goal current_amount
        let new_amount = goal.current_amount + req.amount;
        self.goal_repo
            .update(goal_id, user_id, None, None, None, Some(new_amount), None)
            .await?;

        Ok(entry_id)
    }

    pub async fn get_goal_entries(
        &self,
        goal_id: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<crate::schemas::GoalEntry>, AppError> {
        // Verify ownership
        let _ = self.goal_repo.get_by_id(goal_id, user_id).await?;
        self.entry_repo.get_by_goal(goal_id).await
    }
}
