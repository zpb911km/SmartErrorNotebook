use super::macros::pub_string_enum;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Metadata {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub sync_status: SyncStatus,
    pub sync_version: i64,
}
impl Metadata {
    pub fn new(now: DateTime<Utc>) -> Self {
        Metadata {
            created_at: now,
            updated_at: now,
            deleted_at: None,
            sync_status: SyncStatus::Pending,
            sync_version: 0,
        }
    }
}

pub_string_enum!(
    SyncStatus {
        Pending => "PENDING",
        Synced => "SYNCED",
        Conflict => "CONFLICT",
    }
);
