use crate::domain::model::legacy::SyncStatus;
use crate::domain::repository::legacy::repository_model::sync::{
    CascadeOrphanCheckResult, SyncRecordHeader, SyncRecordOutput,
};
use crate::domain::repository::legacy::SyncRepository;
use crate::domain::repository::{RepositoryFactory, RepositoryTransactionExecutor};
use crate::AppState;
use tauri::State;

#[tauri::command]
pub async fn legacy_get_all_records(
    state: State<'_, AppState>,
) -> Result<Vec<SyncRecordHeader>, String> {
    state
        .repository_transaction_executor
        .execute(|factory, _| {
            Box::pin(async move { Ok(factory.legacy_sync_repository().all_headers().await) })
        })
        .await
}

#[tauri::command]
pub async fn legacy_get_all_pending_records(
    state: State<'_, AppState>,
) -> Result<Vec<SyncRecordOutput>, String> {
    state
        .repository_transaction_executor
        .execute(|factory, _| {
            Box::pin(async move { Ok(factory.legacy_sync_repository().pending_records().await) })
        })
        .await
}

#[tauri::command]
pub async fn legacy_get_record_for_upload(
    state: State<'_, AppState>,
    table_name: Option<String>,
    record_id: String,
) -> Result<SyncRecordOutput, String> {
    state
        .repository_transaction_executor
        .execute(|factory, _| {
            Box::pin(async move {
                factory
                    .legacy_sync_repository()
                    .record_for_upload(table_name, record_id)
                    .await
            })
        })
        .await
}

#[tauri::command]
pub async fn legacy_set_record_sync_status_version(
    state: State<'_, AppState>,
    table_name: Option<String>,
    record_id: String,
    status: SyncStatus,
    version: i32,
) -> Result<String, String> {
    state
        .repository_transaction_executor
        .execute(|factory, _| {
            Box::pin(async move {
                factory
                    .legacy_sync_repository()
                    .set_status_version(table_name, record_id, status, version)
                    .await
            })
        })
        .await
}

#[tauri::command]
pub async fn legacy_purge_synced_deletions(
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    state
        .repository_transaction_executor
        .execute(|factory, _| {
            Box::pin(async move {
                Ok(factory
                    .legacy_sync_repository()
                    .purge_synced_deletions()
                    .await)
            })
        })
        .await
}

#[tauri::command]
pub async fn legacy_check_orphan_records(
    state: State<'_, AppState>,
) -> Result<CascadeOrphanCheckResult, String> {
    state
        .repository_transaction_executor
        .execute(|factory, _| {
            Box::pin(async move {
                Ok(factory
                    .legacy_sync_repository()
                    .check_orphans(chrono::Utc::now().timestamp())
                    .await)
            })
        })
        .await
}
