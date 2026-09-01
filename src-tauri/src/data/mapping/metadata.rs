use std::str::FromStr;

use chrono::{DateTime, Utc};

use crate::model::{Metadata, SyncStatus};

use super::error::MappingError;

pub(super) fn try_from_values(
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    deleted_at: Option<DateTime<Utc>>,
    sync_status: String,
    sync_version: i64,
) -> Result<Metadata, MappingError> {
    let sync_status = SyncStatus::from_str(&sync_status).map_err(MappingError::StringToEnum)?;
    Ok(Metadata {
        created_at,
        updated_at,
        deleted_at,
        sync_status,
        sync_version,
    })
}
