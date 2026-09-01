use uuid::Uuid;

use crate::domain::model::Tag;

use super::error::{RepositoryDeleteError, RepositoryFindError, RepositorySaveError};

#[async_trait::async_trait]
pub(crate) trait TagRepository: Send {
    async fn save(&self, tag: &Tag) -> Result<(), RepositorySaveError>;
    async fn delete_by_id(&self, tag_id: &Uuid) -> Result<(), RepositoryDeleteError>;
    async fn find_all(&self, include_deleted: bool) -> Result<Vec<Tag>, RepositoryFindError>;
    async fn find_by_id(&self, id: &Uuid) -> Result<Option<Tag>, RepositoryFindError>;
    async fn find_by_question_id(
        &self,
        question_id: &Uuid,
    ) -> Result<Vec<Tag>, RepositoryFindError>;
}
