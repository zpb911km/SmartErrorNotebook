use super::repository_model::srs_data::{NewSrsData, SrsStateChanges, SyncedSrsData};
use crate::domain::model::SrsData;

#[async_trait::async_trait]
pub trait SrsDataRepository: Send + Sync {
    async fn find_by_question(&self, question_id: String, active_only: bool) -> Option<SrsData>;
    async fn list_active(&self) -> Vec<SrsData>;
    async fn create(&self, input: NewSrsData) -> SrsData;
    async fn update_state(&self, input: SrsStateChanges) -> SrsData;
    async fn upsert_synced(&self, input: SyncedSrsData);
}
