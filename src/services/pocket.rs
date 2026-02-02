use uuid::Uuid;

use crate::error::AppError;
use crate::repository::{PocketRepository, TransactionRepository};
use crate::schemas::{CreatePocket, Pocket, UpdatePocket};

pub struct PocketService {
    pocket_repo: PocketRepository,
    transaction_repo: TransactionRepository,
}

impl PocketService {
    pub fn new(pocket_repo: PocketRepository, transaction_repo: TransactionRepository) -> Self {
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
