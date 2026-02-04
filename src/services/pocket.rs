use async_trait::async_trait;
use uuid::Uuid;

use crate::error::AppError;
use crate::repository::{PocketRepository, TransactionRepository};
use crate::schemas::{CreatePocket, Pocket, UpdatePocket};

#[async_trait]
pub trait PocketRepo: Send + Sync {
    async fn create(
        &self,
        user_id: Uuid,
        name: &str,
        description: Option<String>,
        icon: Option<String>,
    ) -> Result<Uuid, AppError>;
    async fn get_all(&self, user_id: Uuid) -> Result<Vec<Pocket>, AppError>;
    async fn get_by_id(&self, id: Uuid, user_id: Uuid) -> Result<Pocket, AppError>;
    async fn update(
        &self,
        id: Uuid,
        user_id: Uuid,
        name: Option<String>,
        description: Option<String>,
        icon: Option<String>,
    ) -> Result<(), AppError>;
    async fn delete(&self, id: Uuid, user_id: Uuid) -> Result<u64, AppError>;
}

#[async_trait]
pub trait PocketTransactionRepo: Send + Sync {
    async fn get_pocket_balance(
        &self,
        user_id: Uuid,
        pocket_id: Uuid,
    ) -> Result<rust_decimal::Decimal, AppError>;
}

#[async_trait]
impl PocketRepo for PocketRepository {
    async fn create(
        &self,
        user_id: Uuid,
        name: &str,
        description: Option<String>,
        icon: Option<String>,
    ) -> Result<Uuid, AppError> {
        self.create(user_id, name, description, icon).await
    }

    async fn get_all(&self, user_id: Uuid) -> Result<Vec<Pocket>, AppError> {
        self.get_all(user_id).await
    }

    async fn get_by_id(&self, id: Uuid, user_id: Uuid) -> Result<Pocket, AppError> {
        self.get_by_id(id, user_id).await
    }

    async fn update(
        &self,
        id: Uuid,
        user_id: Uuid,
        name: Option<String>,
        description: Option<String>,
        icon: Option<String>,
    ) -> Result<(), AppError> {
        self.update(id, user_id, name, description, icon).await
    }

    async fn delete(&self, id: Uuid, user_id: Uuid) -> Result<u64, AppError> {
        self.delete(id, user_id).await
    }
}

#[async_trait]
impl PocketTransactionRepo for TransactionRepository {
    async fn get_pocket_balance(
        &self,
        user_id: Uuid,
        pocket_id: Uuid,
    ) -> Result<rust_decimal::Decimal, AppError> {
        self.get_pocket_balance(user_id, pocket_id).await
    }
}

pub type PocketServiceImpl = PocketService<PocketRepository, TransactionRepository>;

pub struct PocketService<PRepo, TRepo> {
    pocket_repo: PRepo,
    transaction_repo: TRepo,
}

impl<PRepo, TRepo> PocketService<PRepo, TRepo>
where
    PRepo: PocketRepo,
    TRepo: PocketTransactionRepo,
{
    pub fn new(pocket_repo: PRepo, transaction_repo: TRepo) -> Self {
        Self {
            pocket_repo,
            transaction_repo,
        }
    }

    pub async fn create_pocket(&self, user_id: Uuid, req: CreatePocket) -> Result<Uuid, AppError> {
        if req.name.trim().is_empty() {
            return Err(AppError::ValidationError(
                "Pocket name cannot be empty".to_string(),
            ));
        }

        self.pocket_repo
            .create(user_id, &req.name, req.description, req.icon)
            .await
    }

    pub async fn get_pockets(&self, user_id: Uuid) -> Result<Vec<Pocket>, AppError> {
        self.pocket_repo.get_all(user_id).await
    }

    pub async fn get_pocket(
        &self,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<crate::schemas::PocketDetail, AppError> {
        let pocket = self.pocket_repo.get_by_id(id, user_id).await?;
        let balance = self
            .transaction_repo
            .get_pocket_balance(user_id, id)
            .await?;

        Ok(crate::schemas::PocketDetail {
            id: pocket.id,
            name: pocket.name,
            description: pocket.description,
            icon: pocket.icon,
            is_default: pocket.is_default,
            created_at: pocket.created_at,
            balance,
        })
    }

    pub async fn update_pocket(
        &self,
        id: Uuid,
        user_id: Uuid,
        req: UpdatePocket,
    ) -> Result<(), AppError> {
        // Validate name if provided
        if let Some(ref name) = req.name {
            if name.trim().is_empty() {
                return Err(AppError::ValidationError(
                    "Pocket name cannot be empty".to_string(),
                ));
            }
        }

        self.pocket_repo
            .update(id, user_id, req.name, req.description, req.icon)
            .await
    }

    pub async fn delete_pocket(&self, id: Uuid, user_id: Uuid) -> Result<(), AppError> {
        let deleted = self.pocket_repo.delete(id, user_id).await?;
        if deleted == 0 {
            return Err(AppError::NotFoundError("Pocket not found".to_string()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{PocketRepo, PocketService, PocketTransactionRepo};
    use crate::error::AppError;
    use crate::schemas::{CreatePocket, Pocket, UpdatePocket};
    use async_trait::async_trait;
    use rust_decimal::Decimal;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use uuid::Uuid;

    #[derive(Clone, Default)]
    struct MockPocketRepo {
        state: Arc<Mutex<MockPocketState>>,
    }

    struct MockPocketState {
        pockets: HashMap<Uuid, Pocket>,
        create_calls: Vec<(Uuid, String, Option<String>, Option<String>)>,
        update_calls: Vec<(Uuid, Uuid, Option<String>, Option<String>, Option<String>)>,
        delete_result: u64,
    }

    impl Default for MockPocketState {
        fn default() -> Self {
            Self {
                pockets: HashMap::new(),
                create_calls: Vec::new(),
                update_calls: Vec::new(),
                delete_result: 1,
            }
        }
    }

    #[async_trait]
    impl PocketRepo for MockPocketRepo {
        async fn create(
            &self,
            user_id: Uuid,
            name: &str,
            description: Option<String>,
            icon: Option<String>,
        ) -> Result<Uuid, AppError> {
            let mut state = self.state.lock().unwrap();
            state
                .create_calls
                .push((user_id, name.to_string(), description, icon));
            Ok(Uuid::new_v4())
        }

        async fn get_all(&self, _user_id: Uuid) -> Result<Vec<Pocket>, AppError> {
            let state = self.state.lock().unwrap();
            Ok(state.pockets.values().map(clone_pocket_ref).collect())
        }

        async fn get_by_id(&self, id: Uuid, _user_id: Uuid) -> Result<Pocket, AppError> {
            let state = self.state.lock().unwrap();
            state
                .pockets
                .get(&id)
                .map(clone_pocket_ref)
                .ok_or_else(|| AppError::NotFoundError("Pocket not found".to_string()))
        }

        async fn update(
            &self,
            id: Uuid,
            user_id: Uuid,
            name: Option<String>,
            description: Option<String>,
            icon: Option<String>,
        ) -> Result<(), AppError> {
            let mut state = self.state.lock().unwrap();
            state
                .update_calls
                .push((id, user_id, name, description, icon));
            Ok(())
        }

        async fn delete(&self, _id: Uuid, _user_id: Uuid) -> Result<u64, AppError> {
            Ok(self.state.lock().unwrap().delete_result)
        }
    }

    #[derive(Clone, Default)]
    struct MockTransactionRepo {
        balance: Decimal,
    }

    #[async_trait]
    impl PocketTransactionRepo for MockTransactionRepo {
        async fn get_pocket_balance(
            &self,
            _user_id: Uuid,
            _pocket_id: Uuid,
        ) -> Result<Decimal, AppError> {
            Ok(self.balance)
        }
    }

    fn make_pocket(id: Uuid) -> Pocket {
        Pocket {
            id,
            name: "Pocket".to_string(),
            description: Some("desc".to_string()),
            icon: "icon".to_string(),
            is_default: false,
            created_at: None,
        }
    }

    fn clone_pocket_ref(pocket: &Pocket) -> Pocket {
        Pocket {
            id: pocket.id,
            name: pocket.name.clone(),
            description: pocket.description.clone(),
            icon: pocket.icon.clone(),
            is_default: pocket.is_default,
            created_at: pocket.created_at,
        }
    }

    fn make_service(
        pocket_repo: MockPocketRepo,
        transaction_repo: MockTransactionRepo,
    ) -> PocketService<MockPocketRepo, MockTransactionRepo> {
        PocketService::new(pocket_repo, transaction_repo)
    }

    #[tokio::test]
    async fn create_pocket_rejects_empty_name() {
        let service = make_service(MockPocketRepo::default(), MockTransactionRepo::default());
        let req = CreatePocket {
            name: "  ".to_string(),
            description: None,
            icon: None,
        };

        let err = service
            .create_pocket(Uuid::new_v4(), req)
            .await
            .unwrap_err();
        assert!(
            matches!(err, AppError::ValidationError(msg) if msg == "Pocket name cannot be empty")
        );
    }

    #[tokio::test]
    async fn create_pocket_calls_repo() {
        let pocket_repo = MockPocketRepo::default();
        let service = make_service(pocket_repo.clone(), MockTransactionRepo::default());
        let user_id = Uuid::new_v4();
        let req = CreatePocket {
            name: "Pocket".to_string(),
            description: Some("desc".to_string()),
            icon: Some("icon".to_string()),
        };

        let _ = service.create_pocket(user_id, req).await.unwrap();
        let calls = pocket_repo.state.lock().unwrap().create_calls.clone();
        assert_eq!(calls.len(), 1);
        let call = &calls[0];
        assert_eq!(call.0, user_id);
        assert_eq!(call.1, "Pocket");
        assert_eq!(call.2.as_deref(), Some("desc"));
        assert_eq!(call.3.as_deref(), Some("icon"));
    }

    #[tokio::test]
    async fn update_pocket_rejects_empty_name() {
        let service = make_service(MockPocketRepo::default(), MockTransactionRepo::default());
        let req = UpdatePocket {
            name: Some("  ".to_string()),
            description: None,
            icon: None,
        };

        let err = service
            .update_pocket(Uuid::new_v4(), Uuid::new_v4(), req)
            .await
            .unwrap_err();
        assert!(
            matches!(err, AppError::ValidationError(msg) if msg == "Pocket name cannot be empty")
        );
    }

    #[tokio::test]
    async fn delete_pocket_returns_not_found() {
        let pocket_repo = MockPocketRepo {
            state: Arc::new(Mutex::new(MockPocketState {
                delete_result: 0,
                ..Default::default()
            })),
        };
        let service = make_service(pocket_repo, MockTransactionRepo::default());
        let err = service
            .delete_pocket(Uuid::new_v4(), Uuid::new_v4())
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::NotFoundError(msg) if msg == "Pocket not found"));
    }

    #[tokio::test]
    async fn get_pocket_includes_balance() {
        let pocket_id = Uuid::new_v4();
        let mut state = MockPocketState::default();
        state.pockets.insert(pocket_id, make_pocket(pocket_id));
        let pocket_repo = MockPocketRepo {
            state: Arc::new(Mutex::new(state)),
        };
        let transaction_repo = MockTransactionRepo {
            balance: Decimal::new(25, 0),
        };
        let service = make_service(pocket_repo, transaction_repo);

        let detail = service.get_pocket(pocket_id, Uuid::new_v4()).await.unwrap();
        assert_eq!(detail.id, pocket_id);
        assert_eq!(detail.balance, Decimal::new(25, 0));
    }
}
