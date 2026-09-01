use super::repository_model::sync::{CascadeOrphanCheckResult, SyncRecordHeader, SyncRecordOutput};
use crate::domain::model::legacy;

#[async_trait::async_trait]
pub trait SyncRepository: Send + Sync {
    async fn all_headers(&self) -> Vec<SyncRecordHeader>;
    async fn pending_records(&self) -> Vec<SyncRecordOutput>;
    async fn record_for_upload(
        &self,
        table_name: Option<String>,
        id: String,
    ) -> Result<SyncRecordOutput, String>;
    async fn set_status_version(
        &self,
        table_name: Option<String>,
        id: String,
        status: legacy::SyncStatus,
        version: i32,
    ) -> Result<String, String>;
    async fn purge_synced_deletions(&self) -> serde_json::Value;
    async fn check_orphans(&self, now: i64) -> CascadeOrphanCheckResult;
}
