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

    pub(crate) fn touch(&mut self, now: DateTime<Utc>) {
        self.updated_at = now;
        self.sync_status = SyncStatus::Pending;
    }

    pub(crate) fn mark_as_deleted(&mut self, now: DateTime<Utc>) {
        self.touch(now);
        self.deleted_at = Some(now);
    }
}

pub_string_enum!(
    SyncStatus {
        Pending => "PENDING",
        Synced => "SYNCED",
        Conflict => "CONFLICT",
    }
);

#[cfg(test)]
mod tests {
    use chrono::Duration;

    use super::*;

    #[test]
    fn touch_updates_only_mutation_metadata() {
        let created_at = Utc::now();
        let updated_at = created_at + Duration::hours(1);
        let deleted_at = created_at + Duration::minutes(30);
        let mut metadata = Metadata {
            created_at,
            updated_at: created_at,
            deleted_at: Some(deleted_at),
            sync_status: SyncStatus::Synced,
            sync_version: 7,
        };

        metadata.touch(updated_at);

        assert_eq!(metadata.created_at, created_at);
        assert_eq!(metadata.updated_at, updated_at);
        assert_eq!(metadata.deleted_at, Some(deleted_at));
        assert_eq!(metadata.sync_status, SyncStatus::Pending);
        assert_eq!(metadata.sync_version, 7);
    }
}
