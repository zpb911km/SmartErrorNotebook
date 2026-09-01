use uuid::Uuid;

use crate::domain::model::Attachment;

use super::error::{RepositoryDeleteError, RepositoryFindError, RepositorySaveError};

#[async_trait::async_trait]
pub(crate) trait AttachmentRepository: Send {
    async fn save(&self, attachment: &Attachment) -> Result<(), RepositorySaveError>;
    async fn delete_by_id(&self, attachment_id: &Uuid) -> Result<(), RepositoryDeleteError>;
    async fn find_by_id(&self, id: &Uuid) -> Result<Option<Attachment>, RepositoryFindError>;
    async fn find_by_question_id(
        &self,
        question_id: &Uuid,
    ) -> Result<Vec<Attachment>, RepositoryFindError>;
}
