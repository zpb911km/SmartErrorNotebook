use super::repository_model::error_question::{
    NewQuestion, QuestionChanges, QuestionQuery, SyncedQuestion,
};
use crate::domain::model::Question;

#[async_trait::async_trait]
pub trait ErrorQuestionRepository: Send + Sync {
    async fn list_active(&self, query: QuestionQuery) -> Vec<Question>;
    async fn find_by_id(&self, id: String) -> Question;
    async fn create(&self, input: NewQuestion) -> Question;
    async fn update(&self, input: QuestionChanges) -> Question;
    async fn soft_delete_with_srs(&self, id: String, now: i64);
    async fn count_active(&self) -> u64;
    async fn upsert_synced(&self, input: SyncedQuestion);
}
