use uuid::Uuid;

use crate::model::Source;

use super::error::{RepositoryDeleteError, RepositoryFindError, RepositorySaveError};

#[async_trait::async_trait]
pub(crate) trait SourceRepository: Send {
    async fn save(&self, source: &Source) -> Result<(), RepositorySaveError>;
    async fn delete_by_id(&self, source_id: &Uuid) -> Result<(), RepositoryDeleteError>;
    async fn find_by_id(&self, id: &Uuid) -> Result<Option<Source>, RepositoryFindError>;
    async fn find_by_subject_id(
        &self,
        subject_id: &Uuid,
    ) -> Result<Vec<Source>, RepositoryFindError>;
}
