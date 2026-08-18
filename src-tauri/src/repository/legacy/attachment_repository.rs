use super::repository_model::attachment::{NewAttachment, SyncedAttachment};
use crate::model::Attachment;

#[async_trait::async_trait]
pub trait AttachmentRepository: Send + Sync {
    async fn create(&self, input: NewAttachment) -> Result<Attachment, String>;
    async fn list_active_by_question(&self, question_id: String) -> Vec<Attachment>;
    async fn unlink(&self, question_id: String, id: String, now: i64) -> Result<(), String>;
    async fn upsert_synced(&self, input: SyncedAttachment) -> Result<(), String>;
}
