use uuid::Uuid;

use crate::error::AppError;
use crate::repository::DataDeletionRepository;

pub struct DataDeletionService {
    repo: DataDeletionRepository,
}

impl DataDeletionService {
    pub fn new(repo: DataDeletionRepository) -> Self {
        Self { repo }
    }

    pub async fn nuke_user_data(&self, user_id: Uuid) -> Result<(), AppError> {
        self.repo.nuke_user_data(user_id).await
    }
}
