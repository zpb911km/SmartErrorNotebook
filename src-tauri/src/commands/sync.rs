pub use crate::repository::sync::{CascadeOrphanCheckResult, SyncRecordHeader, SyncRecordOutput};
use crate::AppState;
use tauri::State;

#[tauri::command]
pub async fn get_all_records(state: State<'_, AppState>) -> Result<Vec<SyncRecordHeader>, String> {
    state
        .repositories
        .sync
        .all_headers()
        .await
        .map_err(Into::into)
}
#[tauri::command]
pub async fn get_all_pending_records(
    state: State<'_, AppState>,
) -> Result<Vec<SyncRecordOutput>, String> {
    state
        .repositories
        .sync
        .pending_records()
        .await
        .map_err(Into::into)
}
#[tauri::command]
pub async fn get_record_for_upload(
    state: State<'_, AppState>,
    record_id: String,
) -> Result<SyncRecordOutput, String> {
    state
        .repositories
        .sync
        .record_for_upload(record_id)
        .await
        .map_err(Into::into)
}
#[tauri::command]
pub async fn set_record_sync_status_version(
    state: State<'_, AppState>,
    record_id: String,
    status: String,
    version: i32,
) -> Result<String, String> {
    state
        .repositories
        .sync
        .set_status_version(record_id, status, version)
        .await
        .map_err(Into::into)
}
#[tauri::command]
pub async fn purge_synced_deletions(
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    state
        .repositories
        .sync
        .purge_synced_deletions()
        .await
        .map_err(Into::into)
}
#[tauri::command]
pub async fn check_orphan_records(
    state: State<'_, AppState>,
) -> Result<CascadeOrphanCheckResult, String> {
    state
        .repositories
        .sync
        .check_orphans(chrono::Utc::now().timestamp())
        .await
        .map_err(Into::into)
}
