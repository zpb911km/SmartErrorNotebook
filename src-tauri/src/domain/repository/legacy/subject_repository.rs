use super::repository_model::subject::{NewSubject, SubjectChanges, SyncedSubject};
use crate::domain::model::Subject;

#[async_trait::async_trait]
pub trait SubjectRepository: Send + Sync {
    async fn list_active(&self) -> Vec<Subject>;
    async fn create(&self, input: NewSubject) -> Subject;
    async fn update(&self, input: SubjectChanges) -> Result<Subject, String>;
    async fn soft_delete(&self, id: String, now: i64) -> Result<(), String>;
    async fn upsert_synced(&self, input: SyncedSubject);
}
