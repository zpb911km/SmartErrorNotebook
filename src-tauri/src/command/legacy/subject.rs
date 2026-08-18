use super::request::subject::{CreateSubjectInput, UpdateSubjectInput, UpsertSubjectInput};
use super::response::subject::SubjectOutput;
use crate::repository::legacy::repository_model::subject::{
    NewSubject, SubjectChanges, SyncedSubject,
};
use crate::repository::legacy::SubjectRepository;
use crate::repository::{RepositoryFactory, RepositoryTransactionExecutor};
use crate::AppState;
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub async fn get_subjects(state: State<'_, AppState>) -> Result<Vec<SubjectOutput>, String> {
    state
        .repository_transaction_executor
        .execute(|factory, _| {
            Box::pin(async move {
                Ok(factory
                    .legacy_subject_repository()
                    .list_active()
                    .await
                    .into_iter()
                    .map(Into::into)
                    .collect())
            })
        })
        .await
}

#[tauri::command]
pub async fn create_subject(
    state: State<'_, AppState>,
    input: CreateSubjectInput,
) -> Result<SubjectOutput, String> {
    state
        .repository_transaction_executor
        .execute(|factory, _| {
            Box::pin(async move {
                Ok(factory
                    .legacy_subject_repository()
                    .create(NewSubject {
                        id: Uuid::new_v4().to_string(),
                        name: input.name,
                        color: input.color,
                        now: chrono::Utc::now().timestamp(),
                    })
                    .await
                    .into())
            })
        })
        .await
}

#[tauri::command]
pub async fn update_subject(
    state: State<'_, AppState>,
    input: UpdateSubjectInput,
) -> Result<SubjectOutput, String> {
    state
        .repository_transaction_executor
        .execute(|factory, _| {
            Box::pin(async move {
                Ok(factory
                    .legacy_subject_repository()
                    .update(SubjectChanges {
                        id: input.id,
                        name: input.name,
                        color: input.color,
                        now: chrono::Utc::now().timestamp(),
                    })
                    .await?
                    .into())
            })
        })
        .await
}

#[tauri::command]
pub async fn delete_subject(state: State<'_, AppState>, id: String) -> Result<(), String> {
    state
        .repository_transaction_executor
        .execute(|factory, _| {
            Box::pin(async move {
                factory
                    .legacy_subject_repository()
                    .soft_delete(id, chrono::Utc::now().timestamp())
                    .await?;
                Ok(())
            })
        })
        .await
}

#[tauri::command]
pub async fn upsert_subject(
    state: State<'_, AppState>,
    input: UpsertSubjectInput,
) -> Result<(), String> {
    state
        .repository_transaction_executor
        .execute(|factory, _| {
            Box::pin(async move {
                factory
                    .legacy_subject_repository()
                    .upsert_synced(SyncedSubject {
                        id: input.id,
                        version: input.version,
                        deleted_at: input.deleted_at,
                        name: input.name,
                        color: input.color,
                        now: chrono::Utc::now().timestamp(),
                    })
                    .await;
                Ok(())
            })
        })
        .await
}
