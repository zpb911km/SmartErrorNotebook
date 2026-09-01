use super::request::source::{
    CreateSourceInput, SourceFilter, UpdateSourceInput, UpsertSourceInput,
};
use super::response::source::SourceOutput;
use crate::domain::repository::legacy::repository_model::source::{
    NewSource, SourceChanges, SourceValues, SyncedSource,
};
use crate::domain::repository::legacy::SourceRepository;
use crate::domain::repository::{RepositoryFactory, RepositoryTransactionExecutor};
use crate::AppState;
use tauri::State;
use uuid::Uuid;

fn values(i: CreateSourceInput) -> SourceValues {
    SourceValues {
        subject_id: i.subject_id,
        book: i.book,
        chapter: i.chapter,
        knowledge: i.knowledge,
    }
}

#[tauri::command]
pub async fn get_sources(
    state: State<'_, AppState>,
    filter: Option<SourceFilter>,
) -> Result<Vec<SourceOutput>, String> {
    let subject_id = filter.unwrap_or_default().subject_id;
    state
        .repository_transaction_executor
        .execute(|factory, _| {
            Box::pin(async move {
                Ok(factory
                    .legacy_source_repository()
                    .list_active(subject_id.clone())
                    .await
                    .into_iter()
                    .map(|record| SourceOutput::new(record.source, record.question_id))
                    .collect())
            })
        })
        .await
}

#[tauri::command]
pub async fn get_source(state: State<'_, AppState>, id: String) -> Result<SourceOutput, String> {
    state
        .repository_transaction_executor
        .execute(|factory, _| {
            Box::pin(async move {
                let record = factory.legacy_source_repository().find_by_id(id).await;
                Ok(SourceOutput::new(record.source, record.question_id))
            })
        })
        .await
}

#[tauri::command]
pub async fn get_books(
    state: State<'_, AppState>,
    subject_id: Option<String>,
) -> Result<Vec<String>, String> {
    state
        .repository_transaction_executor
        .execute(|factory, _| {
            Box::pin(async move {
                Ok(factory
                    .legacy_source_repository()
                    .list_books(subject_id)
                    .await)
            })
        })
        .await
}

#[tauri::command]
pub async fn get_chapters(
    state: State<'_, AppState>,
    subject_id: Option<String>,
    book: String,
) -> Result<Vec<String>, String> {
    state
        .repository_transaction_executor
        .execute(|factory, _| {
            Box::pin(async move {
                Ok(factory
                    .legacy_source_repository()
                    .list_chapters(subject_id, book)
                    .await)
            })
        })
        .await
}

#[tauri::command]
pub async fn get_knowledges(
    state: State<'_, AppState>,
    subject_id: Option<String>,
    book: String,
    chapter: String,
) -> Result<Vec<String>, String> {
    state
        .repository_transaction_executor
        .execute(|factory, _| {
            Box::pin(async move {
                Ok(factory
                    .legacy_source_repository()
                    .list_knowledges(subject_id, book, chapter)
                    .await)
            })
        })
        .await
}

#[tauri::command]
pub async fn create_source(
    state: State<'_, AppState>,
    input: CreateSourceInput,
) -> Result<SourceOutput, String> {
    state
        .repository_transaction_executor
        .execute(|factory, _| {
            Box::pin(async move {
                Ok(SourceOutput::new(
                    factory
                        .legacy_source_repository()
                        .create(NewSource {
                            id: Uuid::new_v4().to_string(),
                            values: values(input),
                            now: chrono::Utc::now().timestamp(),
                        })
                        .await,
                    None,
                ))
            })
        })
        .await
}

#[tauri::command]
pub async fn update_source(
    state: State<'_, AppState>,
    input: UpdateSourceInput,
) -> Result<SourceOutput, String> {
    state
        .repository_transaction_executor
        .execute(|factory, _| {
            Box::pin(async move {
                Ok(SourceOutput::new(
                    factory
                        .legacy_source_repository()
                        .update(SourceChanges {
                            id: input.id,
                            values: SourceValues {
                                subject_id: input.subject_id,
                                book: input.book,
                                chapter: input.chapter,
                                knowledge: input.knowledge,
                            },
                            now: chrono::Utc::now().timestamp(),
                        })
                        .await,
                    None,
                ))
            })
        })
        .await
}

#[tauri::command]
pub async fn delete_source(state: State<'_, AppState>, id: String) -> Result<(), String> {
    state
        .repository_transaction_executor
        .execute(|factory, _| {
            Box::pin(async move {
                factory
                    .legacy_source_repository()
                    .soft_delete(id, chrono::Utc::now().timestamp())
                    .await;
                Ok(())
            })
        })
        .await
}

#[tauri::command]
pub async fn get_or_create_source_id(
    state: State<'_, AppState>,
    input: CreateSourceInput,
) -> Result<String, String> {
    state
        .repository_transaction_executor
        .execute(|factory, _| {
            Box::pin(async move {
                let repository = factory.legacy_source_repository();
                let values = values(input);
                if let Some(model) = repository.find_active_exact(&values).await {
                    return Ok(model.id.to_string());
                }
                Ok(repository
                    .create(NewSource {
                        id: Uuid::new_v4().to_string(),
                        values,
                        now: chrono::Utc::now().timestamp(),
                    })
                    .await
                    .id
                    .to_string())
            })
        })
        .await
}

#[tauri::command]
pub async fn upsert_source(
    state: State<'_, AppState>,
    input: UpsertSourceInput,
) -> Result<(), String> {
    state
        .repository_transaction_executor
        .execute(|factory, _| {
            Box::pin(async move {
                factory
                    .legacy_source_repository()
                    .upsert_synced(SyncedSource {
                        id: input.id,
                        version: input.version,
                        deleted_at: input.deleted_at,
                        question_id: input.question_id,
                        values: SourceValues {
                            subject_id: input.subject_id,
                            book: input.book,
                            chapter: input.chapter,
                            knowledge: input.knowledge,
                        },
                        now: chrono::Utc::now().timestamp(),
                    })
                    .await;
                Ok(())
            })
        })
        .await
}
