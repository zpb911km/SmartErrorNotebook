use uuid::Uuid;

use crate::model::Subject;

use super::error::{RepositoryDeleteError, RepositoryFindError, RepositorySaveError};

#[async_trait::async_trait]
pub(crate) trait SubjectRepository: Send {
    async fn save(&self, subject: &Subject) -> Result<(), RepositorySaveError>;
    async fn delete_by_id(&self, subject_id: &Uuid) -> Result<(), RepositoryDeleteError>;
    async fn find_all(&self, include_deleted: bool) -> Result<Vec<Subject>, RepositoryFindError>;
    async fn find_by_id(&self, id: &Uuid) -> Result<Option<Subject>, RepositoryFindError>;
}
