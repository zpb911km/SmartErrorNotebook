use uuid::Uuid;

use crate::domain::model::Attachment;

use super::error::{RepositoryDeleteError, RepositoryFindError, RepositorySaveError};

#[async_trait::async_trait]
/// Collection results returned by `find_*` methods have no ordering guarantee.
pub(crate) trait AttachmentRepository: Send {
    async fn save(&self, attachment: &Attachment) -> Result<(), RepositorySaveError>;
    async fn delete_by_id(&self, attachment_id: &Uuid) -> Result<(), RepositoryDeleteError>;
    async fn find_by_id(
        &self,
        id: &Uuid,
        include_deleted: bool,
    ) -> Result<Option<Attachment>, RepositoryFindError>;
    async fn find_by_question_id(
        &self,
        question_id: &Uuid,
        include_deleted: bool,
    ) -> Result<Vec<Attachment>, RepositoryFindError>;
    async fn is_referenced_by_question(
        &self,
        attachment_id: &Uuid,
        include_deleted: bool,
    ) -> Result<bool, RepositoryFindError>;
}
