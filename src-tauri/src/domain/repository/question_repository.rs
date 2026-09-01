use uuid::Uuid;

use crate::domain::model::Question;

use super::error::{
    RepositoryCountError, RepositoryDeleteError, RepositoryFindError, RepositorySaveError,
};

#[async_trait::async_trait]
pub(crate) trait QuestionRepository: Send {
    async fn save(&self, question: &Question) -> Result<(), RepositorySaveError>;
    async fn delete_by_id(&self, question_id: &Uuid) -> Result<(), RepositoryDeleteError>;
    async fn find_all(&self, include_deleted: bool) -> Result<Vec<Question>, RepositoryFindError>;
    async fn find_by_id(&self, id: &Uuid) -> Result<Option<Question>, RepositoryFindError>;
    async fn count_all(&self, include_deleted: bool) -> Result<u64, RepositoryCountError>;
}
