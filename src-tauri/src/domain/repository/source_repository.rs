use uuid::Uuid;

use crate::domain::model::Source;

use super::error::{RepositoryDeleteError, RepositoryFindError, RepositorySaveError};

#[async_trait::async_trait]
/// Collection results returned by `find_*` methods have no ordering guarantee.
pub(crate) trait SourceRepository: Send {
    async fn save(&self, source: &Source) -> Result<(), RepositorySaveError>;
    async fn delete_by_id(&self, source_id: &Uuid) -> Result<(), RepositoryDeleteError>;
    async fn find_all(&self, include_deleted: bool) -> Result<Vec<Source>, RepositoryFindError>;
    async fn find_by_id(
        &self,
        id: &Uuid,
        include_deleted: bool,
    ) -> Result<Option<Source>, RepositoryFindError>;
    async fn find_by_subject_id(
        &self,
        subject_id: &Uuid,
        include_deleted: bool,
    ) -> Result<Vec<Source>, RepositoryFindError>;
}
