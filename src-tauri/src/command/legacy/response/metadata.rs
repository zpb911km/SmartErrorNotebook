use crate::model::{Metadata, SyncStatus};
use serde::Serialize;

#[derive(Clone, Serialize)]
pub struct MetadataOutput {
    pub created_at: i64,
    pub updated_at: i64,
    pub deleted_at: Option<i64>,
    pub version: i32,
    pub sync_status: String,
    pub sync_hash: Option<String>,
}

impl From<&Metadata> for MetadataOutput {
    fn from(value: &Metadata) -> Self {
        Self {
            created_at: value.created_at.timestamp(),
            updated_at: value.updated_at.timestamp(),
            deleted_at: value.deleted_at.map(|date| date.timestamp()),
            version: value.sync_version.clamp(i32::MIN as i64, i32::MAX as i64) as i32,
            sync_status: match value.sync_status {
                SyncStatus::Pending => "pending",
                SyncStatus::Synced => "synced",
                SyncStatus::Conflict => "conflict",
            }
            .into(),
            sync_hash: None,
        }
    }
}
