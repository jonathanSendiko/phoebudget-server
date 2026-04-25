use async_trait::async_trait;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::error::AppError;
use crate::repository::{GoalEntryRepository, GoalRepository, PocketRepository, SubGoalRepository};

const MAX_SUB_GOALS: usize = 50;
const MAX_CURRENCY_SCALE: u32 = 2;

#[async_trait]
pub trait GoalRepo: Send + Sync {
    async fn create(
        &self,
        user_id: Uuid,
        pocket_id: Uuid,
        name: &str,
        description: Option<String>,
        target_amount: Decimal,
        current_amount: Option<Decimal>,
        icon: Option<String>,
    ) -> Result<Uuid, AppError>;
    async fn create_with_sub_goals(
        &self,
        user_id: Uuid,
        pocket_id: Uuid,
        name: &str,
        description: Option<String>,
        target_amount: Decimal,
        current_amount: Option<Decimal>,
        icon: Option<String>,
        sub_goals: &[crate::schemas::CreateSubGoal],
    ) -> Result<Uuid, AppError>;
    async fn get_all(&self, user_id: Uuid) -> Result<Vec<crate::schemas::GoalSummary>, AppError>;
    async fn get_by_id(
        &self,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<crate::schemas::GoalDetail, AppError>;
    async fn update(
        &self,
        id: Uuid,
        user_id: Uuid,
        name: Option<String>,
        description: Option<String>,
        target_amount: Option<Decimal>,
        current_amount: Option<Decimal>,
        pocket_id: Option<Uuid>,
        icon: Option<String>,
    ) -> Result<u64, AppError>;
    async fn update_with_sub_goals(
        &self,
        id: Uuid,
        user_id: Uuid,
        name: Option<String>,
        description: Option<String>,
        target_amount: Option<Decimal>,
        current_amount: Option<Decimal>,
        pocket_id: Option<Uuid>,
        icon: Option<String>,
        sub_goals: &[crate::schemas::CreateSubGoal],
    ) -> Result<u64, AppError>;
    async fn delete(&self, id: Uuid, user_id: Uuid) -> Result<u64, AppError>;
}

#[async_trait]
pub trait GoalEntryRepo: Send + Sync {
    async fn create(
        &self,
        goal_id: Uuid,
        sub_goal_id: Option<Uuid>,
        amount: Decimal,
        description: Option<String>,
        date: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<Uuid, AppError>;
    async fn create_and_update_goal_amount(
        &self,
        goal_id: Uuid,
        sub_goal_id: Option<Uuid>,
        amount: Decimal,
        description: Option<String>,
        date: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<Uuid, AppError>;
    async fn get_by_goal(&self, goal_id: Uuid) -> Result<Vec<crate::schemas::GoalEntry>, AppError>;
}

#[async_trait]
pub trait GoalPocketRepo: Send + Sync {
    async fn get_by_id(&self, id: Uuid, user_id: Uuid) -> Result<crate::schemas::Pocket, AppError>;
}

#[async_trait]
pub trait SubGoalRepo: Send + Sync {
    async fn replace_for_goal(
        &self,
        goal_id: Uuid,
        sub_goals: &[crate::schemas::CreateSubGoal],
    ) -> Result<(), AppError>;
    async fn get_by_goal(&self, goal_id: Uuid) -> Result<Vec<crate::schemas::SubGoal>, AppError>;
    async fn has_allocated_entries(&self, goal_id: Uuid) -> Result<bool, AppError>;
}

#[async_trait]
impl GoalRepo for GoalRepository {
    async fn create(
        &self,
        user_id: Uuid,
        pocket_id: Uuid,
        name: &str,
        description: Option<String>,
        target_amount: Decimal,
        current_amount: Option<Decimal>,
        icon: Option<String>,
    ) -> Result<Uuid, AppError> {
        self.create(
            user_id,
            pocket_id,
            name,
            description,
            target_amount,
            current_amount,
            icon,
        )
        .await
    }

    async fn create_with_sub_goals(
        &self,
        user_id: Uuid,
        pocket_id: Uuid,
        name: &str,
        description: Option<String>,
        target_amount: Decimal,
        current_amount: Option<Decimal>,
        icon: Option<String>,
        sub_goals: &[crate::schemas::CreateSubGoal],
    ) -> Result<Uuid, AppError> {
        self.create_with_sub_goals(
            user_id,
            pocket_id,
            name,
            description,
            target_amount,
            current_amount,
            icon,
            sub_goals,
        )
        .await
    }

    async fn get_all(&self, user_id: Uuid) -> Result<Vec<crate::schemas::GoalSummary>, AppError> {
        self.get_all(user_id).await
    }

    async fn get_by_id(
        &self,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<crate::schemas::GoalDetail, AppError> {
        self.get_by_id(id, user_id).await
    }

    async fn update(
        &self,
        id: Uuid,
        user_id: Uuid,
        name: Option<String>,
        description: Option<String>,
        target_amount: Option<Decimal>,
        current_amount: Option<Decimal>,
        pocket_id: Option<Uuid>,
        icon: Option<String>,
    ) -> Result<u64, AppError> {
        self.update(
            id,
            user_id,
            name,
            description,
            target_amount,
            current_amount,
            pocket_id,
            icon,
        )
        .await
    }

    async fn update_with_sub_goals(
        &self,
        id: Uuid,
        user_id: Uuid,
        name: Option<String>,
        description: Option<String>,
        target_amount: Option<Decimal>,
        current_amount: Option<Decimal>,
        pocket_id: Option<Uuid>,
        icon: Option<String>,
        sub_goals: &[crate::schemas::CreateSubGoal],
    ) -> Result<u64, AppError> {
        self.update_with_sub_goals(
            id,
            user_id,
            name,
            description,
            target_amount,
            current_amount,
            pocket_id,
            icon,
            sub_goals,
        )
        .await
    }

    async fn delete(&self, id: Uuid, user_id: Uuid) -> Result<u64, AppError> {
        self.delete(id, user_id).await
    }
}

#[async_trait]
impl GoalEntryRepo for GoalEntryRepository {
    async fn create(
        &self,
        goal_id: Uuid,
        sub_goal_id: Option<Uuid>,
        amount: Decimal,
        description: Option<String>,
        date: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<Uuid, AppError> {
        self.create(goal_id, sub_goal_id, amount, description, date)
            .await
    }

    async fn create_and_update_goal_amount(
        &self,
        goal_id: Uuid,
        sub_goal_id: Option<Uuid>,
        amount: Decimal,
        description: Option<String>,
        date: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<Uuid, AppError> {
        self.create_and_update_goal_amount(goal_id, sub_goal_id, amount, description, date)
            .await
    }

    async fn get_by_goal(&self, goal_id: Uuid) -> Result<Vec<crate::schemas::GoalEntry>, AppError> {
        self.get_by_goal(goal_id).await
    }
}

#[async_trait]
impl GoalPocketRepo for PocketRepository {
    async fn get_by_id(&self, id: Uuid, user_id: Uuid) -> Result<crate::schemas::Pocket, AppError> {
        self.get_by_id(id, user_id).await
    }
}

#[async_trait]
impl SubGoalRepo for SubGoalRepository {
    async fn replace_for_goal(
        &self,
        goal_id: Uuid,
        sub_goals: &[crate::schemas::CreateSubGoal],
    ) -> Result<(), AppError> {
        self.replace_for_goal(goal_id, sub_goals).await
    }

    async fn get_by_goal(&self, goal_id: Uuid) -> Result<Vec<crate::schemas::SubGoal>, AppError> {
        self.get_by_goal(goal_id).await
    }

    async fn has_allocated_entries(&self, goal_id: Uuid) -> Result<bool, AppError> {
        self.has_allocated_entries(goal_id).await
    }
}

pub type GoalServiceImpl =
    GoalService<GoalRepository, GoalEntryRepository, PocketRepository, SubGoalRepository>;

pub struct GoalService<GRepo, ERepo, PRepo, SRepo> {
    goal_repo: GRepo,
    entry_repo: ERepo,
    pocket_repo: PRepo,
    sub_goal_repo: SRepo,
}

impl<GRepo, ERepo, PRepo, SRepo> GoalService<GRepo, ERepo, PRepo, SRepo>
where
    GRepo: GoalRepo,
    ERepo: GoalEntryRepo,
    PRepo: GoalPocketRepo,
    SRepo: SubGoalRepo,
{
    pub fn new(
        goal_repo: GRepo,
        entry_repo: ERepo,
        pocket_repo: PRepo,
        sub_goal_repo: SRepo,
    ) -> Self {
        Self {
            goal_repo,
            entry_repo,
            pocket_repo,
            sub_goal_repo,
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
        validate_currency_scale(req.target_amount, "Target amount")?;
        if let Some(current_amount) = req.current_amount {
            validate_currency_scale(current_amount, "Current amount")?;
        }

        if req.name.trim().is_empty() {
            return Err(AppError::ValidationError(
                "Goal name cannot be empty".to_string(),
            ));
        }

        if let Some(sub_goals) = &req.sub_goals {
            if req.current_amount.is_some() && !sub_goals.is_empty() {
                return Err(AppError::ValidationError(
                    "Current amount cannot be set when sub goals are provided".to_string(),
                ));
            }
            validate_sub_goals(sub_goals, req.target_amount)?;
        }

        // Verify pocket exists and belongs to user
        let _ = self.pocket_repo.get_by_id(req.pocket_id, user_id).await?;

        let goal_id = if let Some(sub_goals) = &req.sub_goals {
            self.goal_repo
                .create_with_sub_goals(
                    user_id,
                    req.pocket_id,
                    &req.name,
                    req.description,
                    req.target_amount,
                    req.current_amount,
                    req.icon,
                    sub_goals,
                )
                .await?
        } else {
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
                .await?
        };

        Ok(goal_id)
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
        let mut goal = self.goal_repo.get_by_id(id, user_id).await?;
        goal.sub_goals = self.sub_goal_repo.get_by_goal(id).await?;
        Ok(goal)
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
            validate_currency_scale(target, "Target amount")?;
        }

        if let Some(current_amount) = req.current_amount {
            validate_currency_scale(current_amount, "Current amount")?;
        }

        if let Some(pocket_id) = req.pocket_id {
            self.pocket_repo.get_by_id(pocket_id, user_id).await?;
        }

        if req.sub_goals.is_some() || req.target_amount.is_some() {
            let current_goal = self.goal_repo.get_by_id(id, user_id).await?;
            let target_amount = req.target_amount.unwrap_or(current_goal.target_amount);
            if let Some(sub_goals) = &req.sub_goals {
                if !current_goal.current_amount.is_zero()
                    || self.sub_goal_repo.has_allocated_entries(id).await?
                {
                    return Err(AppError::ValidationError(
                        "Sub goals cannot be replaced after funds are allocated".to_string(),
                    ));
                }
                validate_sub_goals(sub_goals, target_amount)?;
            } else {
                let existing_sub_goals = self.sub_goal_repo.get_by_goal(id).await?;
                if !existing_sub_goals.is_empty() {
                    let total: Decimal = existing_sub_goals
                        .iter()
                        .map(|sub_goal| sub_goal.target_amount)
                        .sum();
                    if total != target_amount {
                        return Err(AppError::ValidationError(
                            "Sub goal total must equal goal target amount".to_string(),
                        ));
                    }
                }
            }
        }

        let updated = if let Some(sub_goals) = &req.sub_goals {
            self.goal_repo
                .update_with_sub_goals(
                    id,
                    user_id,
                    req.name,
                    req.description,
                    req.target_amount,
                    req.current_amount,
                    req.pocket_id,
                    req.icon,
                    sub_goals,
                )
                .await?
        } else {
            self.goal_repo
                .update(
                    id,
                    user_id,
                    req.name,
                    req.description,
                    req.target_amount,
                    req.current_amount,
                    req.pocket_id,
                    req.icon,
                )
                .await?
        };

        if updated == 0 {
            return Err(AppError::NotFoundError("Goal not found".to_string()));
        }

        Ok(())
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
        let _goal = self.goal_repo.get_by_id(goal_id, user_id).await?;
        let sub_goals = self.sub_goal_repo.get_by_goal(goal_id).await?;
        let has_sub_goals = !sub_goals.is_empty();
        match (has_sub_goals, req.sub_goal_id) {
            (true, None) => {
                return Err(AppError::ValidationError(
                    "Sub goal is required for this goal".to_string(),
                ));
            }
            (true, Some(sub_goal_id)) => {
                if !sub_goals.iter().any(|sub_goal| sub_goal.id == sub_goal_id) {
                    return Err(AppError::ValidationError(
                        "Sub goal does not belong to goal".to_string(),
                    ));
                }
            }
            (false, Some(_)) => {
                return Err(AppError::ValidationError(
                    "Goal has no sub goals".to_string(),
                ));
            }
            (false, None) => {}
        }

        let entry_id = self
            .entry_repo
            .create_and_update_goal_amount(
                goal_id,
                req.sub_goal_id,
                req.amount,
                req.description,
                req.date,
            )
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

fn validate_sub_goals(
    sub_goals: &[crate::schemas::CreateSubGoal],
    target_amount: Decimal,
) -> Result<(), AppError> {
    if sub_goals.len() > MAX_SUB_GOALS {
        return Err(AppError::ValidationError(
            "Sub goals cannot exceed 50 items".to_string(),
        ));
    }

    if sub_goals.is_empty() {
        return Ok(());
    }

    let mut total = Decimal::ZERO;
    for sub_goal in sub_goals {
        if sub_goal.name.trim().is_empty() {
            return Err(AppError::ValidationError(
                "Sub goal name cannot be empty".to_string(),
            ));
        }

        if sub_goal.name.len() > 100 {
            return Err(AppError::ValidationError(
                "Sub goal name cannot exceed 100 characters".to_string(),
            ));
        }

        if sub_goal.target_amount <= Decimal::ZERO {
            return Err(AppError::ValidationError(
                "Sub goal target amount must be positive".to_string(),
            ));
        }
        validate_currency_scale(sub_goal.target_amount, "Sub goal target amount")?;

        total += sub_goal.target_amount;
    }

    if total != target_amount {
        return Err(AppError::ValidationError(
            "Sub goal total must equal goal target amount".to_string(),
        ));
    }

    Ok(())
}

fn validate_currency_scale(amount: Decimal, field_name: &str) -> Result<(), AppError> {
    if amount.scale() > MAX_CURRENCY_SCALE {
        return Err(AppError::ValidationError(format!(
            "{field_name} cannot have more than 2 decimal places"
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{GoalEntryRepo, GoalPocketRepo, GoalRepo, GoalService, SubGoalRepo};
    use crate::error::AppError;
    use crate::schemas::{
        CreateGoal, CreateGoalEntry, CreateSubGoal, GoalDetail, GoalEntry, GoalSummary, Pocket,
        SubGoal, UpdateGoal,
    };
    use async_trait::async_trait;
    use chrono::{DateTime, Utc};
    use rust_decimal::Decimal;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use uuid::Uuid;

    #[derive(Clone, Default)]
    struct MockGoalRepo {
        state: Arc<Mutex<MockGoalState>>,
    }

    struct MockGoalState {
        goals: HashMap<Uuid, GoalDetail>,
        create_calls: Vec<(
            Uuid,
            Uuid,
            String,
            Option<String>,
            Decimal,
            Option<Decimal>,
            Option<String>,
        )>,
        create_with_sub_goal_calls: Vec<(
            Uuid,
            Uuid,
            String,
            Option<String>,
            Decimal,
            Option<Decimal>,
            Option<String>,
            Vec<CreateSubGoal>,
        )>,
        update_calls: Vec<(
            Uuid,
            Uuid,
            Option<String>,
            Option<String>,
            Option<Decimal>,
            Option<Decimal>,
            Option<Uuid>,
            Option<String>,
        )>,
        update_with_sub_goal_calls: Vec<(
            Uuid,
            Uuid,
            Option<String>,
            Option<String>,
            Option<Decimal>,
            Option<Decimal>,
            Option<Uuid>,
            Option<String>,
            Vec<CreateSubGoal>,
        )>,
        delete_result: u64,
    }

    impl Default for MockGoalState {
        fn default() -> Self {
            Self {
                goals: HashMap::new(),
                create_calls: Vec::new(),
                create_with_sub_goal_calls: Vec::new(),
                update_calls: Vec::new(),
                update_with_sub_goal_calls: Vec::new(),
                delete_result: 1,
            }
        }
    }

    #[async_trait]
    impl GoalRepo for MockGoalRepo {
        async fn create(
            &self,
            user_id: Uuid,
            pocket_id: Uuid,
            name: &str,
            description: Option<String>,
            target_amount: Decimal,
            current_amount: Option<Decimal>,
            icon: Option<String>,
        ) -> Result<Uuid, AppError> {
            let mut state = self.state.lock().unwrap();
            state.create_calls.push((
                user_id,
                pocket_id,
                name.to_string(),
                description,
                target_amount,
                current_amount,
                icon,
            ));
            Ok(Uuid::new_v4())
        }

        async fn create_with_sub_goals(
            &self,
            user_id: Uuid,
            pocket_id: Uuid,
            name: &str,
            description: Option<String>,
            target_amount: Decimal,
            current_amount: Option<Decimal>,
            icon: Option<String>,
            sub_goals: &[CreateSubGoal],
        ) -> Result<Uuid, AppError> {
            let mut state = self.state.lock().unwrap();
            state.create_with_sub_goal_calls.push((
                user_id,
                pocket_id,
                name.to_string(),
                description,
                target_amount,
                current_amount,
                icon,
                sub_goals.to_vec(),
            ));
            Ok(Uuid::new_v4())
        }

        async fn get_all(&self, _user_id: Uuid) -> Result<Vec<GoalSummary>, AppError> {
            Ok(Vec::new())
        }

        async fn get_by_id(&self, id: Uuid, _user_id: Uuid) -> Result<GoalDetail, AppError> {
            let state = self.state.lock().unwrap();
            state
                .goals
                .get(&id)
                .map(clone_goal_detail_ref)
                .ok_or_else(|| AppError::NotFoundError("Goal not found".to_string()))
        }

        async fn update(
            &self,
            id: Uuid,
            user_id: Uuid,
            name: Option<String>,
            description: Option<String>,
            target_amount: Option<Decimal>,
            current_amount: Option<Decimal>,
            pocket_id: Option<Uuid>,
            icon: Option<String>,
        ) -> Result<u64, AppError> {
            let mut state = self.state.lock().unwrap();
            state.update_calls.push((
                id,
                user_id,
                name,
                description,
                target_amount,
                current_amount,
                pocket_id,
                icon,
            ));
            Ok(state.delete_result)
        }

        async fn update_with_sub_goals(
            &self,
            id: Uuid,
            user_id: Uuid,
            name: Option<String>,
            description: Option<String>,
            target_amount: Option<Decimal>,
            current_amount: Option<Decimal>,
            pocket_id: Option<Uuid>,
            icon: Option<String>,
            sub_goals: &[CreateSubGoal],
        ) -> Result<u64, AppError> {
            let mut state = self.state.lock().unwrap();
            state.update_with_sub_goal_calls.push((
                id,
                user_id,
                name,
                description,
                target_amount,
                current_amount,
                pocket_id,
                icon,
                sub_goals.to_vec(),
            ));
            Ok(state.delete_result)
        }

        async fn delete(&self, _id: Uuid, _user_id: Uuid) -> Result<u64, AppError> {
            Ok(self.state.lock().unwrap().delete_result)
        }
    }

    #[derive(Clone, Default)]
    struct MockGoalEntryRepo {
        calls: Arc<
            Mutex<
                Vec<(
                    Uuid,
                    Option<Uuid>,
                    Decimal,
                    Option<String>,
                    Option<DateTime<Utc>>,
                )>,
            >,
        >,
        entries: Arc<Mutex<Vec<GoalEntry>>>,
    }

    #[async_trait]
    impl GoalEntryRepo for MockGoalEntryRepo {
        async fn create(
            &self,
            goal_id: Uuid,
            sub_goal_id: Option<Uuid>,
            amount: Decimal,
            description: Option<String>,
            date: Option<DateTime<Utc>>,
        ) -> Result<Uuid, AppError> {
            self.calls
                .lock()
                .unwrap()
                .push((goal_id, sub_goal_id, amount, description, date));
            Ok(Uuid::new_v4())
        }

        async fn create_and_update_goal_amount(
            &self,
            goal_id: Uuid,
            sub_goal_id: Option<Uuid>,
            amount: Decimal,
            description: Option<String>,
            date: Option<DateTime<Utc>>,
        ) -> Result<Uuid, AppError> {
            self.calls
                .lock()
                .unwrap()
                .push((goal_id, sub_goal_id, amount, description, date));
            Ok(Uuid::new_v4())
        }

        async fn get_by_goal(&self, _goal_id: Uuid) -> Result<Vec<GoalEntry>, AppError> {
            let entries = self.entries.lock().unwrap();
            Ok(entries.iter().map(clone_goal_entry_ref).collect())
        }
    }

    #[derive(Clone, Default)]
    struct MockPocketRepo {
        calls: Arc<Mutex<Vec<(Uuid, Uuid)>>>,
    }

    #[async_trait]
    impl GoalPocketRepo for MockPocketRepo {
        async fn get_by_id(&self, id: Uuid, user_id: Uuid) -> Result<Pocket, AppError> {
            self.calls.lock().unwrap().push((id, user_id));
            Ok(Pocket {
                id,
                name: "Pocket".to_string(),
                description: None,
                icon: "icon".to_string(),
                is_default: false,
                created_at: None,
            })
        }
    }

    #[derive(Clone, Default)]
    struct MockSubGoalRepo {
        replace_calls: Arc<Mutex<Vec<(Uuid, Vec<CreateSubGoal>)>>>,
        sub_goals: Arc<Mutex<Vec<SubGoal>>>,
        has_allocated_entries: bool,
    }

    #[async_trait]
    impl SubGoalRepo for MockSubGoalRepo {
        async fn replace_for_goal(
            &self,
            goal_id: Uuid,
            sub_goals: &[CreateSubGoal],
        ) -> Result<(), AppError> {
            self.replace_calls
                .lock()
                .unwrap()
                .push((goal_id, sub_goals.to_vec()));
            Ok(())
        }

        async fn get_by_goal(&self, _goal_id: Uuid) -> Result<Vec<SubGoal>, AppError> {
            let sub_goals = self.sub_goals.lock().unwrap();
            Ok(sub_goals.iter().map(clone_sub_goal_ref).collect())
        }

        async fn has_allocated_entries(&self, _goal_id: Uuid) -> Result<bool, AppError> {
            Ok(self.has_allocated_entries)
        }
    }

    fn make_service(
        goal_repo: MockGoalRepo,
        entry_repo: MockGoalEntryRepo,
        pocket_repo: MockPocketRepo,
    ) -> GoalService<MockGoalRepo, MockGoalEntryRepo, MockPocketRepo, MockSubGoalRepo> {
        GoalService::new(
            goal_repo,
            entry_repo,
            pocket_repo,
            MockSubGoalRepo::default(),
        )
    }

    fn make_service_with_sub_goals(
        goal_repo: MockGoalRepo,
        entry_repo: MockGoalEntryRepo,
        pocket_repo: MockPocketRepo,
        sub_goal_repo: MockSubGoalRepo,
    ) -> GoalService<MockGoalRepo, MockGoalEntryRepo, MockPocketRepo, MockSubGoalRepo> {
        GoalService::new(goal_repo, entry_repo, pocket_repo, sub_goal_repo)
    }

    fn sample_goal_detail(id: Uuid, current_amount: Decimal) -> GoalDetail {
        GoalDetail {
            id,
            name: "Goal".to_string(),
            description: None,
            icon: "savings".to_string(),
            target_amount: Decimal::new(100, 0),
            current_amount,
            percentage: Decimal::ZERO,
            pocket: crate::schemas::PocketSummary {
                id: Uuid::new_v4(),
                name: "Pocket".to_string(),
                icon: "icon".to_string(),
            },
            sub_goals: Vec::new(),
            created_at: None,
        }
    }

    fn clone_goal_detail_ref(detail: &GoalDetail) -> GoalDetail {
        GoalDetail {
            id: detail.id,
            name: detail.name.clone(),
            description: detail.description.clone(),
            icon: detail.icon.clone(),
            target_amount: detail.target_amount,
            current_amount: detail.current_amount,
            percentage: detail.percentage,
            pocket: crate::schemas::PocketSummary {
                id: detail.pocket.id,
                name: detail.pocket.name.clone(),
                icon: detail.pocket.icon.clone(),
            },
            sub_goals: detail.sub_goals.iter().map(clone_sub_goal_ref).collect(),
            created_at: detail.created_at,
        }
    }

    fn clone_sub_goal_ref(sub_goal: &SubGoal) -> SubGoal {
        SubGoal {
            id: sub_goal.id,
            goal_id: sub_goal.goal_id,
            name: sub_goal.name.clone(),
            target_amount: sub_goal.target_amount,
            current_amount: sub_goal.current_amount,
            percentage: sub_goal.percentage,
            position: sub_goal.position,
            created_at: sub_goal.created_at,
        }
    }

    fn clone_goal_entry_ref(entry: &GoalEntry) -> GoalEntry {
        GoalEntry {
            id: entry.id,
            goal_id: entry.goal_id,
            sub_goal_id: entry.sub_goal_id,
            amount: entry.amount,
            description: entry.description.clone(),
            date: entry.date,
        }
    }

    #[tokio::test]
    async fn create_goal_rejects_non_positive_target() {
        let service = make_service(
            MockGoalRepo::default(),
            MockGoalEntryRepo::default(),
            MockPocketRepo::default(),
        );

        let req = CreateGoal {
            name: "Goal".to_string(),
            description: None,
            target_amount: Decimal::ZERO,
            current_amount: None,
            pocket_id: Uuid::new_v4(),
            icon: None,
            sub_goals: None,
        };

        let err = service.create_goal(Uuid::new_v4(), req).await.unwrap_err();
        assert!(
            matches!(err, AppError::ValidationError(msg) if msg == "Target amount must be positive")
        );
    }

    #[tokio::test]
    async fn create_goal_rejects_empty_name() {
        let service = make_service(
            MockGoalRepo::default(),
            MockGoalEntryRepo::default(),
            MockPocketRepo::default(),
        );

        let req = CreateGoal {
            name: "  ".to_string(),
            description: None,
            target_amount: Decimal::new(10, 0),
            current_amount: None,
            pocket_id: Uuid::new_v4(),
            icon: None,
            sub_goals: None,
        };

        let err = service.create_goal(Uuid::new_v4(), req).await.unwrap_err();
        assert!(
            matches!(err, AppError::ValidationError(msg) if msg == "Goal name cannot be empty")
        );
    }

    #[tokio::test]
    async fn create_goal_requires_pocket_and_calls_repo() {
        let goal_repo = MockGoalRepo::default();
        let entry_repo = MockGoalEntryRepo::default();
        let pocket_repo = MockPocketRepo::default();
        let service = make_service(goal_repo.clone(), entry_repo, pocket_repo.clone());

        let user_id = Uuid::new_v4();
        let pocket_id = Uuid::new_v4();
        let req = CreateGoal {
            name: "Goal".to_string(),
            description: Some("desc".to_string()),
            target_amount: Decimal::new(100, 0),
            current_amount: Some(Decimal::new(20, 0)),
            pocket_id,
            icon: Some("icon".to_string()),
            sub_goals: None,
        };

        let _ = service.create_goal(user_id, req).await.unwrap();
        assert_eq!(pocket_repo.calls.lock().unwrap().len(), 1);

        let create_calls = goal_repo.state.lock().unwrap().create_calls.clone();
        assert_eq!(create_calls.len(), 1);
        let call = &create_calls[0];
        assert_eq!(call.0, user_id);
        assert_eq!(call.1, pocket_id);
        assert_eq!(call.2, "Goal");
        assert_eq!(call.3.as_deref(), Some("desc"));
        assert_eq!(call.4, Decimal::new(100, 0));
        assert_eq!(call.5, Some(Decimal::new(20, 0)));
        assert_eq!(call.6.as_deref(), Some("icon"));
    }

    #[tokio::test]
    async fn create_goal_rejects_current_amount_when_sub_goals_are_provided() {
        let service = make_service(
            MockGoalRepo::default(),
            MockGoalEntryRepo::default(),
            MockPocketRepo::default(),
        );

        let req = CreateGoal {
            name: "Goal".to_string(),
            description: None,
            target_amount: Decimal::new(100, 0),
            current_amount: Some(Decimal::new(10, 0)),
            pocket_id: Uuid::new_v4(),
            icon: None,
            sub_goals: Some(vec![CreateSubGoal {
                name: "Part".to_string(),
                target_amount: Decimal::new(100, 0),
            }]),
        };

        let err = service.create_goal(Uuid::new_v4(), req).await.unwrap_err();
        assert!(
            matches!(err, AppError::ValidationError(msg) if msg == "Current amount cannot be set when sub goals are provided")
        );
    }

    #[tokio::test]
    async fn create_goal_rejects_sub_goal_total_mismatch() {
        let service = make_service(
            MockGoalRepo::default(),
            MockGoalEntryRepo::default(),
            MockPocketRepo::default(),
        );

        let req = CreateGoal {
            name: "Goal".to_string(),
            description: None,
            target_amount: Decimal::new(100, 0),
            current_amount: None,
            pocket_id: Uuid::new_v4(),
            icon: None,
            sub_goals: Some(vec![
                CreateSubGoal {
                    name: "Part A".to_string(),
                    target_amount: Decimal::new(40, 0),
                },
                CreateSubGoal {
                    name: "Part B".to_string(),
                    target_amount: Decimal::new(50, 0),
                },
            ]),
        };

        let err = service.create_goal(Uuid::new_v4(), req).await.unwrap_err();
        assert!(
            matches!(err, AppError::ValidationError(msg) if msg == "Sub goal total must equal goal target amount")
        );
    }

    #[tokio::test]
    async fn create_goal_rejects_more_than_fifty_sub_goals() {
        let service = make_service(
            MockGoalRepo::default(),
            MockGoalEntryRepo::default(),
            MockPocketRepo::default(),
        );

        let sub_goals = (0..51)
            .map(|i| CreateSubGoal {
                name: format!("Part {i}"),
                target_amount: Decimal::ONE,
            })
            .collect();

        let req = CreateGoal {
            name: "Goal".to_string(),
            description: None,
            target_amount: Decimal::new(51, 0),
            current_amount: None,
            pocket_id: Uuid::new_v4(),
            icon: None,
            sub_goals: Some(sub_goals),
        };

        let err = service.create_goal(Uuid::new_v4(), req).await.unwrap_err();
        assert!(
            matches!(err, AppError::ValidationError(msg) if msg == "Sub goals cannot exceed 50 items")
        );
    }

    #[tokio::test]
    async fn create_goal_uses_atomic_repo_call_for_valid_sub_goals() {
        let goal_repo = MockGoalRepo::default();
        let service = make_service(
            goal_repo.clone(),
            MockGoalEntryRepo::default(),
            MockPocketRepo::default(),
        );

        let req = CreateGoal {
            name: "Goal".to_string(),
            description: None,
            target_amount: Decimal::new(100, 0),
            current_amount: None,
            pocket_id: Uuid::new_v4(),
            icon: None,
            sub_goals: Some(vec![
                CreateSubGoal {
                    name: "Part A".to_string(),
                    target_amount: Decimal::new(40, 0),
                },
                CreateSubGoal {
                    name: "Part B".to_string(),
                    target_amount: Decimal::new(60, 0),
                },
            ]),
        };

        service.create_goal(Uuid::new_v4(), req).await.unwrap();

        let state = goal_repo.state.lock().unwrap();
        assert_eq!(state.create_calls.len(), 0);
        assert_eq!(state.create_with_sub_goal_calls.len(), 1);
        assert_eq!(state.create_with_sub_goal_calls[0].7.len(), 2);
    }

    #[tokio::test]
    async fn create_goal_rejects_sub_goal_amount_with_more_than_two_decimals() {
        let service = make_service(
            MockGoalRepo::default(),
            MockGoalEntryRepo::default(),
            MockPocketRepo::default(),
        );

        let req = CreateGoal {
            name: "Goal".to_string(),
            description: None,
            target_amount: Decimal::new(100, 0),
            current_amount: None,
            pocket_id: Uuid::new_v4(),
            icon: None,
            sub_goals: Some(vec![CreateSubGoal {
                name: "Part".to_string(),
                target_amount: Decimal::new(100000, 3),
            }]),
        };

        let err = service.create_goal(Uuid::new_v4(), req).await.unwrap_err();
        assert!(
            matches!(err, AppError::ValidationError(msg) if msg == "Sub goal target amount cannot have more than 2 decimal places")
        );
    }

    #[tokio::test]
    async fn create_goal_rejects_sub_goal_name_longer_than_database_limit() {
        let service = make_service(
            MockGoalRepo::default(),
            MockGoalEntryRepo::default(),
            MockPocketRepo::default(),
        );

        let req = CreateGoal {
            name: "Goal".to_string(),
            description: None,
            target_amount: Decimal::new(100, 0),
            current_amount: None,
            pocket_id: Uuid::new_v4(),
            icon: None,
            sub_goals: Some(vec![CreateSubGoal {
                name: "x".repeat(101),
                target_amount: Decimal::new(100, 0),
            }]),
        };

        let err = service.create_goal(Uuid::new_v4(), req).await.unwrap_err();
        assert!(
            matches!(err, AppError::ValidationError(msg) if msg == "Sub goal name cannot exceed 100 characters")
        );
    }

    #[tokio::test]
    async fn update_goal_rejects_non_positive_target() {
        let service = make_service(
            MockGoalRepo::default(),
            MockGoalEntryRepo::default(),
            MockPocketRepo::default(),
        );

        let req = UpdateGoal {
            name: None,
            description: None,
            target_amount: Some(Decimal::ZERO),
            current_amount: None,
            pocket_id: None,
            icon: None,
            sub_goals: None,
        };

        let err = service
            .update_goal(Uuid::new_v4(), Uuid::new_v4(), req)
            .await
            .unwrap_err();
        assert!(
            matches!(err, AppError::ValidationError(msg) if msg == "Target amount must be positive")
        );
    }

    #[tokio::test]
    async fn delete_goal_returns_not_found() {
        let goal_repo = MockGoalRepo {
            state: Arc::new(Mutex::new(MockGoalState {
                delete_result: 0,
                ..Default::default()
            })),
        };
        let service = make_service(
            goal_repo,
            MockGoalEntryRepo::default(),
            MockPocketRepo::default(),
        );

        let err = service
            .delete_goal(Uuid::new_v4(), Uuid::new_v4())
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::NotFoundError(msg) if msg == "Goal not found"));
    }

    #[tokio::test]
    async fn update_goal_requires_pocket_and_passes_it_to_repo() {
        let goal_repo = MockGoalRepo::default();
        let pocket_repo = MockPocketRepo::default();
        let service = make_service(
            goal_repo.clone(),
            MockGoalEntryRepo::default(),
            pocket_repo.clone(),
        );

        let goal_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let pocket_id = Uuid::new_v4();
        let req = UpdateGoal {
            name: Some("Renamed".to_string()),
            description: None,
            target_amount: None,
            current_amount: None,
            pocket_id: Some(pocket_id),
            icon: None,
            sub_goals: None,
        };

        service.update_goal(goal_id, user_id, req).await.unwrap();

        let pocket_calls = pocket_repo.calls.lock().unwrap().clone();
        assert_eq!(pocket_calls, vec![(pocket_id, user_id)]);

        let update_calls = goal_repo.state.lock().unwrap().update_calls.clone();
        assert_eq!(update_calls.len(), 1);
        let update = &update_calls[0];
        assert_eq!(update.0, goal_id);
        assert_eq!(update.1, user_id);
        assert_eq!(update.2.as_deref(), Some("Renamed"));
        assert_eq!(update.6, Some(pocket_id));
    }

    #[tokio::test]
    async fn update_goal_uses_atomic_repo_call_when_replacing_sub_goals() {
        let goal_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let mut goal_repo_state = MockGoalState::default();
        goal_repo_state
            .goals
            .insert(goal_id, sample_goal_detail(goal_id, Decimal::ZERO));
        let goal_repo = MockGoalRepo {
            state: Arc::new(Mutex::new(goal_repo_state)),
        };
        let service = make_service(
            goal_repo.clone(),
            MockGoalEntryRepo::default(),
            MockPocketRepo::default(),
        );

        service
            .update_goal(
                goal_id,
                user_id,
                UpdateGoal {
                    name: None,
                    description: None,
                    target_amount: Some(Decimal::new(100, 0)),
                    current_amount: None,
                    pocket_id: None,
                    icon: None,
                    sub_goals: Some(vec![CreateSubGoal {
                        name: "Part".to_string(),
                        target_amount: Decimal::new(100, 0),
                    }]),
                },
            )
            .await
            .unwrap();

        let state = goal_repo.state.lock().unwrap();
        assert_eq!(state.update_calls.len(), 0);
        assert_eq!(state.update_with_sub_goal_calls.len(), 1);
        assert_eq!(state.update_with_sub_goal_calls[0].8.len(), 1);
    }

    #[tokio::test]
    async fn update_goal_rejects_replacing_sub_goals_after_funds_are_allocated() {
        let goal_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let mut goal_repo_state = MockGoalState::default();
        goal_repo_state
            .goals
            .insert(goal_id, sample_goal_detail(goal_id, Decimal::ZERO));
        let goal_repo = MockGoalRepo {
            state: Arc::new(Mutex::new(goal_repo_state)),
        };
        let sub_goal_repo = MockSubGoalRepo {
            replace_calls: Arc::new(Mutex::new(Vec::new())),
            sub_goals: Arc::new(Mutex::new(Vec::new())),
            has_allocated_entries: true,
        };
        let service = make_service_with_sub_goals(
            goal_repo,
            MockGoalEntryRepo::default(),
            MockPocketRepo::default(),
            sub_goal_repo,
        );

        let err = service
            .update_goal(
                goal_id,
                user_id,
                UpdateGoal {
                    name: None,
                    description: None,
                    target_amount: None,
                    current_amount: None,
                    pocket_id: None,
                    icon: None,
                    sub_goals: Some(vec![CreateSubGoal {
                        name: "Part".to_string(),
                        target_amount: Decimal::new(100, 0),
                    }]),
                },
            )
            .await
            .unwrap_err();

        assert!(
            matches!(err, AppError::ValidationError(msg) if msg == "Sub goals cannot be replaced after funds are allocated")
        );
    }

    #[tokio::test]
    async fn update_goal_rejects_adding_sub_goals_when_goal_has_existing_progress() {
        let goal_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let mut goal_repo_state = MockGoalState::default();
        goal_repo_state
            .goals
            .insert(goal_id, sample_goal_detail(goal_id, Decimal::new(5, 0)));
        let goal_repo = MockGoalRepo {
            state: Arc::new(Mutex::new(goal_repo_state)),
        };
        let service = make_service(
            goal_repo,
            MockGoalEntryRepo::default(),
            MockPocketRepo::default(),
        );

        let err = service
            .update_goal(
                goal_id,
                user_id,
                UpdateGoal {
                    name: None,
                    description: None,
                    target_amount: None,
                    current_amount: None,
                    pocket_id: None,
                    icon: None,
                    sub_goals: Some(vec![CreateSubGoal {
                        name: "Part".to_string(),
                        target_amount: Decimal::new(100, 0),
                    }]),
                },
            )
            .await
            .unwrap_err();

        assert!(
            matches!(err, AppError::ValidationError(msg) if msg == "Sub goals cannot be replaced after funds are allocated")
        );
    }

    #[tokio::test]
    async fn update_goal_returns_not_found_when_repo_updates_nothing() {
        let goal_repo = MockGoalRepo {
            state: Arc::new(Mutex::new(MockGoalState {
                delete_result: 0,
                ..Default::default()
            })),
        };
        let service = make_service(
            goal_repo,
            MockGoalEntryRepo::default(),
            MockPocketRepo::default(),
        );

        let err = service
            .update_goal(
                Uuid::new_v4(),
                Uuid::new_v4(),
                UpdateGoal {
                    name: Some("Renamed".to_string()),
                    description: None,
                    target_amount: None,
                    current_amount: None,
                    pocket_id: None,
                    icon: None,
                    sub_goals: None,
                },
            )
            .await
            .unwrap_err();

        assert!(matches!(err, AppError::NotFoundError(msg) if msg == "Goal not found"));
    }

    #[tokio::test]
    async fn update_goal_rejects_target_when_existing_sub_goals_would_not_match() {
        let goal_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let mut goal_repo_state = MockGoalState::default();
        goal_repo_state
            .goals
            .insert(goal_id, sample_goal_detail(goal_id, Decimal::ZERO));
        let goal_repo = MockGoalRepo {
            state: Arc::new(Mutex::new(goal_repo_state)),
        };
        let sub_goal_repo = MockSubGoalRepo {
            replace_calls: Arc::new(Mutex::new(Vec::new())),
            sub_goals: Arc::new(Mutex::new(vec![SubGoal {
                id: Uuid::new_v4(),
                goal_id,
                name: "Part".to_string(),
                target_amount: Decimal::new(100, 0),
                current_amount: Decimal::ZERO,
                percentage: Decimal::ZERO,
                position: 0,
                created_at: None,
            }])),
            has_allocated_entries: false,
        };
        let service = make_service_with_sub_goals(
            goal_repo,
            MockGoalEntryRepo::default(),
            MockPocketRepo::default(),
            sub_goal_repo,
        );

        let err = service
            .update_goal(
                goal_id,
                user_id,
                UpdateGoal {
                    name: None,
                    description: None,
                    target_amount: Some(Decimal::new(120, 0)),
                    current_amount: None,
                    pocket_id: None,
                    icon: None,
                    sub_goals: None,
                },
            )
            .await
            .unwrap_err();

        assert!(
            matches!(err, AppError::ValidationError(msg) if msg == "Sub goal total must equal goal target amount")
        );
    }

    #[tokio::test]
    async fn create_goal_entry_uses_atomic_entry_repo_call() {
        let goal_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let mut goal_repo_state = MockGoalState::default();
        goal_repo_state
            .goals
            .insert(goal_id, sample_goal_detail(goal_id, Decimal::new(5, 0)));
        let goal_repo = MockGoalRepo {
            state: Arc::new(Mutex::new(goal_repo_state)),
        };
        let entry_repo = MockGoalEntryRepo::default();
        let pocket_repo = MockPocketRepo::default();
        let service = make_service(goal_repo.clone(), entry_repo.clone(), pocket_repo);

        let req = CreateGoalEntry {
            amount: Decimal::new(7, 0),
            description: Some("deposit".to_string()),
            date: None,
            sub_goal_id: None,
        };

        let _ = service
            .create_goal_entry(goal_id, user_id, req)
            .await
            .unwrap();

        assert_eq!(entry_repo.calls.lock().unwrap().len(), 1);
        assert_eq!(entry_repo.calls.lock().unwrap()[0].1, None);
        let updates = goal_repo.state.lock().unwrap().update_calls.clone();
        assert_eq!(updates.len(), 0);
    }

    #[tokio::test]
    async fn create_goal_entry_requires_sub_goal_when_goal_has_sub_goals() {
        let goal_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let mut goal_repo_state = MockGoalState::default();
        goal_repo_state
            .goals
            .insert(goal_id, sample_goal_detail(goal_id, Decimal::new(0, 0)));
        let goal_repo = MockGoalRepo {
            state: Arc::new(Mutex::new(goal_repo_state)),
        };
        let sub_goal_repo = MockSubGoalRepo {
            replace_calls: Arc::new(Mutex::new(Vec::new())),
            sub_goals: Arc::new(Mutex::new(vec![SubGoal {
                id: Uuid::new_v4(),
                goal_id,
                name: "Part".to_string(),
                target_amount: Decimal::new(100, 0),
                current_amount: Decimal::ZERO,
                percentage: Decimal::ZERO,
                position: 0,
                created_at: None,
            }])),
            has_allocated_entries: false,
        };
        let service = make_service_with_sub_goals(
            goal_repo,
            MockGoalEntryRepo::default(),
            MockPocketRepo::default(),
            sub_goal_repo,
        );

        let err = service
            .create_goal_entry(
                goal_id,
                user_id,
                CreateGoalEntry {
                    amount: Decimal::new(7, 0),
                    description: None,
                    date: None,
                    sub_goal_id: None,
                },
            )
            .await
            .unwrap_err();

        assert!(
            matches!(err, AppError::ValidationError(msg) if msg == "Sub goal is required for this goal")
        );
    }

    #[tokio::test]
    async fn create_goal_entry_passes_selected_sub_goal_to_repo() {
        let goal_id = Uuid::new_v4();
        let sub_goal_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let mut goal_repo_state = MockGoalState::default();
        goal_repo_state
            .goals
            .insert(goal_id, sample_goal_detail(goal_id, Decimal::new(0, 0)));
        let goal_repo = MockGoalRepo {
            state: Arc::new(Mutex::new(goal_repo_state)),
        };
        let entry_repo = MockGoalEntryRepo::default();
        let sub_goal_repo = MockSubGoalRepo {
            replace_calls: Arc::new(Mutex::new(Vec::new())),
            sub_goals: Arc::new(Mutex::new(vec![SubGoal {
                id: sub_goal_id,
                goal_id,
                name: "Part".to_string(),
                target_amount: Decimal::new(100, 0),
                current_amount: Decimal::ZERO,
                percentage: Decimal::ZERO,
                position: 0,
                created_at: None,
            }])),
            has_allocated_entries: false,
        };
        let service = make_service_with_sub_goals(
            goal_repo,
            entry_repo.clone(),
            MockPocketRepo::default(),
            sub_goal_repo,
        );

        service
            .create_goal_entry(
                goal_id,
                user_id,
                CreateGoalEntry {
                    amount: Decimal::new(-7, 0),
                    description: None,
                    date: None,
                    sub_goal_id: Some(sub_goal_id),
                },
            )
            .await
            .unwrap();

        let calls = entry_repo.calls.lock().unwrap().clone();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].1, Some(sub_goal_id));
        assert_eq!(calls[0].2, Decimal::new(-7, 0));
    }

    #[tokio::test]
    async fn create_goal_entry_rejects_sub_goal_for_goal_without_sub_goals() {
        let goal_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let mut goal_repo_state = MockGoalState::default();
        goal_repo_state
            .goals
            .insert(goal_id, sample_goal_detail(goal_id, Decimal::new(0, 0)));
        let goal_repo = MockGoalRepo {
            state: Arc::new(Mutex::new(goal_repo_state)),
        };
        let service = make_service(
            goal_repo,
            MockGoalEntryRepo::default(),
            MockPocketRepo::default(),
        );

        let err = service
            .create_goal_entry(
                goal_id,
                user_id,
                CreateGoalEntry {
                    amount: Decimal::new(7, 0),
                    description: None,
                    date: None,
                    sub_goal_id: Some(Uuid::new_v4()),
                },
            )
            .await
            .unwrap_err();

        assert!(matches!(err, AppError::ValidationError(msg) if msg == "Goal has no sub goals"));
    }

    #[tokio::test]
    async fn get_goal_entries_requires_ownership() {
        let goal_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let mut goal_repo_state = MockGoalState::default();
        goal_repo_state
            .goals
            .insert(goal_id, sample_goal_detail(goal_id, Decimal::new(0, 0)));
        let goal_repo = MockGoalRepo {
            state: Arc::new(Mutex::new(goal_repo_state)),
        };
        let entry_repo = MockGoalEntryRepo {
            calls: Arc::new(Mutex::new(Vec::new())),
            entries: Arc::new(Mutex::new(vec![GoalEntry {
                id: Uuid::new_v4(),
                goal_id,
                sub_goal_id: None,
                amount: Decimal::new(5, 0),
                description: None,
                date: Utc::now(),
            }])),
        };
        let service = make_service(goal_repo, entry_repo.clone(), MockPocketRepo::default());

        let entries = service.get_goal_entries(goal_id, user_id).await.unwrap();
        assert_eq!(entries.len(), 1);
    }
}
