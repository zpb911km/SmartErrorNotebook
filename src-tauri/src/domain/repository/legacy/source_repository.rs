use super::repository_model::source::{
    NewSource, SourceChanges, SourceValues, SourceWithContext, SyncedSource,
};
use crate::domain::model::Source;

#[async_trait::async_trait]
pub trait SourceRepository: Send + Sync {
    async fn list_active(&self, subject_id: Option<String>) -> Vec<SourceWithContext>;
    async fn find_by_id(&self, id: String) -> SourceWithContext;
    async fn list_books(&self, subject_id: Option<String>) -> Vec<String>;
    async fn list_chapters(&self, subject_id: Option<String>, book: String) -> Vec<String>;
    async fn list_knowledges(
        &self,
        subject_id: Option<String>,
        book: String,
        chapter: String,
    ) -> Vec<String>;
    async fn find_active_exact(&self, values: &SourceValues) -> Option<Source>;
    async fn create(&self, input: NewSource) -> Source;
    async fn update(&self, input: SourceChanges) -> Source;
    async fn soft_delete(&self, id: String, now: i64);
    async fn upsert_synced(&self, input: SyncedSource);
}
