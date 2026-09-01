use uuid::Uuid;

use crate::model::SrsData;

use super::error::{RepositoryFindError, RepositorySaveError};

#[async_trait::async_trait]
pub(crate) trait SrsDataRepository: Send {
    async fn save(&self, srs_data: &SrsData) -> Result<(), RepositorySaveError>;
    async fn find_all(&self, include_deleted: bool) -> Result<Vec<SrsData>, RepositoryFindError>;
    async fn find_by_question_id(
        &self,
        question_id: &Uuid,
    ) -> Result<Option<SrsData>, RepositoryFindError>;
}
