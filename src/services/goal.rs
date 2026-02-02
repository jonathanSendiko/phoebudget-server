use async_trait::async_trait;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::error::AppError;
use crate::repository::{GoalEntryRepository, GoalRepository, PocketRepository};

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
        icon: Option<String>,
    ) -> Result<(), AppError>;
    async fn delete(&self, id: Uuid, user_id: Uuid) -> Result<u64, AppError>;
}

#[async_trait]
pub trait GoalEntryRepo: Send + Sync {
    async fn create(
        &self,
        goal_id: Uuid,
        amount: Decimal,
        description: Option<String>,
        date: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<Uuid, AppError>;
    async fn get_by_goal(&self, goal_id: Uuid)
        -> Result<Vec<crate::schemas::GoalEntry>, AppError>;
}

#[async_trait]
pub trait GoalPocketRepo: Send + Sync {
    async fn get_by_id(
        &self,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<crate::schemas::Pocket, AppError>;
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
        icon: Option<String>,
    ) -> Result<(), AppError> {
        self.update(
            id,
            user_id,
            name,
            description,
            target_amount,
            current_amount,
            icon,
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
        amount: Decimal,
        description: Option<String>,
        date: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<Uuid, AppError> {
        self.create(goal_id, amount, description, date).await
    }

    async fn get_by_goal(
        &self,
        goal_id: Uuid,
    ) -> Result<Vec<crate::schemas::GoalEntry>, AppError> {
        self.get_by_goal(goal_id).await
    }
}

#[async_trait]
impl GoalPocketRepo for PocketRepository {
    async fn get_by_id(
        &self,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<crate::schemas::Pocket, AppError> {
        self.get_by_id(id, user_id).await
    }
}

pub type GoalServiceImpl = GoalService<GoalRepository, GoalEntryRepository, PocketRepository>;

pub struct GoalService<GRepo, ERepo, PRepo> {
    goal_repo: GRepo,
    entry_repo: ERepo,
    pocket_repo: PRepo,
}

impl<GRepo, ERepo, PRepo> GoalService<GRepo, ERepo, PRepo>
where
    GRepo: GoalRepo,
    ERepo: GoalEntryRepo,
    PRepo: GoalPocketRepo,
{
    pub fn new(goal_repo: GRepo, entry_repo: ERepo, pocket_repo: PRepo) -> Self {
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

#[cfg(test)]
mod tests {
    use super::{GoalEntryRepo, GoalPocketRepo, GoalRepo, GoalService};
    use crate::error::AppError;
    use crate::schemas::{
        CreateGoal, CreateGoalEntry, GoalDetail, GoalEntry, GoalSummary, Pocket, UpdateGoal,
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
        create_calls: Vec<(Uuid, Uuid, String, Option<String>, Decimal, Option<Decimal>, Option<String>)>,
        update_calls: Vec<(Uuid, Uuid, Option<String>, Option<String>, Option<Decimal>, Option<Decimal>, Option<String>)>,
        delete_result: u64,
    }

    impl Default for MockGoalState {
        fn default() -> Self {
            Self {
                goals: HashMap::new(),
                create_calls: Vec::new(),
                update_calls: Vec::new(),
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
            icon: Option<String>,
        ) -> Result<(), AppError> {
            let mut state = self.state.lock().unwrap();
            state
                .update_calls
                .push((id, user_id, name, description, target_amount, current_amount, icon));
            Ok(())
        }

        async fn delete(&self, _id: Uuid, _user_id: Uuid) -> Result<u64, AppError> {
            Ok(self.state.lock().unwrap().delete_result)
        }
    }

    #[derive(Clone, Default)]
    struct MockGoalEntryRepo {
        calls: Arc<Mutex<Vec<(Uuid, Decimal, Option<String>, Option<DateTime<Utc>>)>>>,
        entries: Arc<Mutex<Vec<GoalEntry>>>,
    }

    #[async_trait]
    impl GoalEntryRepo for MockGoalEntryRepo {
        async fn create(
            &self,
            goal_id: Uuid,
            amount: Decimal,
            description: Option<String>,
            date: Option<DateTime<Utc>>,
        ) -> Result<Uuid, AppError> {
            self.calls
                .lock()
                .unwrap()
                .push((goal_id, amount, description, date));
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
        async fn get_by_id(
            &self,
            id: Uuid,
            user_id: Uuid,
        ) -> Result<Pocket, AppError> {
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

    fn make_service(
        goal_repo: MockGoalRepo,
        entry_repo: MockGoalEntryRepo,
        pocket_repo: MockPocketRepo,
    ) -> GoalService<MockGoalRepo, MockGoalEntryRepo, MockPocketRepo> {
        GoalService::new(goal_repo, entry_repo, pocket_repo)
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
            created_at: detail.created_at,
        }
    }

    fn clone_goal_entry_ref(entry: &GoalEntry) -> GoalEntry {
        GoalEntry {
            id: entry.id,
            goal_id: entry.goal_id,
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
        };

        let err = service.create_goal(Uuid::new_v4(), req).await.unwrap_err();
        assert!(matches!(err, AppError::ValidationError(msg) if msg == "Target amount must be positive"));
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
        };

        let err = service.create_goal(Uuid::new_v4(), req).await.unwrap_err();
        assert!(matches!(err, AppError::ValidationError(msg) if msg == "Goal name cannot be empty"));
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
            icon: None,
        };

        let err = service
            .update_goal(Uuid::new_v4(), Uuid::new_v4(), req)
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::ValidationError(msg) if msg == "Target amount must be positive"));
    }

    #[tokio::test]
    async fn delete_goal_returns_not_found() {
        let goal_repo = MockGoalRepo {
            state: Arc::new(Mutex::new(MockGoalState {
                delete_result: 0,
                ..Default::default()
            })),
        };
        let service = make_service(goal_repo, MockGoalEntryRepo::default(), MockPocketRepo::default());

        let err = service
            .delete_goal(Uuid::new_v4(), Uuid::new_v4())
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::NotFoundError(msg) if msg == "Goal not found"));
    }

    #[tokio::test]
    async fn create_goal_entry_updates_goal_amount() {
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
        };

        let _ = service
            .create_goal_entry(goal_id, user_id, req)
            .await
            .unwrap();

        assert_eq!(entry_repo.calls.lock().unwrap().len(), 1);
        let updates = goal_repo.state.lock().unwrap().update_calls.clone();
        assert_eq!(updates.len(), 1);
        let update = &updates[0];
        assert_eq!(update.0, goal_id);
        assert_eq!(update.1, user_id);
        assert_eq!(update.5, Some(Decimal::new(12, 0)));
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
            entries: Arc::new(Mutex::new(vec![
                GoalEntry {
                    id: Uuid::new_v4(),
                    goal_id,
                    amount: Decimal::new(5, 0),
                    description: None,
                    date: Utc::now(),
                },
            ])),
        };
        let service = make_service(goal_repo, entry_repo.clone(), MockPocketRepo::default());

        let entries = service.get_goal_entries(goal_id, user_id).await.unwrap();
        assert_eq!(entries.len(), 1);
    }
}
