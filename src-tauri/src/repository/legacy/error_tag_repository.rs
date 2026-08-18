use super::repository_model::error_tag::{NewErrorTag, SyncedErrorTag, TagWithQuestion};
use crate::model::Tag;

#[async_trait::async_trait]
pub trait ErrorTagRepository: Send + Sync {
    async fn create_many(&self, tags: Vec<NewErrorTag>) -> Vec<Tag>;
    async fn list_active(&self) -> Vec<Tag>;
    async fn list_active_with_questions(&self) -> Vec<TagWithQuestion>;
    async fn list_active_by_question(&self, question_id: String) -> Vec<Tag>;
    async fn unlink(&self, question_id: String, id: String, now: i64) -> Result<(), String>;
    async fn upsert_synced(&self, input: SyncedErrorTag) -> Result<(), String>;
    async fn update_by_name(
        &self,
        old_name: String,
        new_name: String,
        new_color: String,
        now: i64,
    ) -> Result<(), String>;
    async fn update_by_id(
        &self,
        id: String,
        name: String,
        color: Option<String>,
        now: i64,
    ) -> Result<(), String>;
}
