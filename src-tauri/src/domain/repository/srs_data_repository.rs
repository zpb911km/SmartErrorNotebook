use uuid::Uuid;

use crate::domain::model::SrsData;

use super::error::{RepositoryFindError, RepositorySaveError};

#[async_trait::async_trait]
/// Collection results returned by `find_*` methods have no ordering guarantee.
pub(crate) trait SrsDataRepository: Send {
    async fn save(&self, srs_data: &SrsData) -> Result<(), RepositorySaveError>;
    async fn find_all(&self, include_deleted: bool) -> Result<Vec<SrsData>, RepositoryFindError>;
    async fn find_by_question_id(
        &self,
        question_id: &Uuid,
        include_deleted: bool,
    ) -> Result<Option<SrsData>, RepositoryFindError>;
}
