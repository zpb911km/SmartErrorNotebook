use super::request::error_tag::{CreateErrorTagsForQuestionInput, UpsertErrorTagInput};
use super::response::error_tag::TagOutput;
use crate::domain::repository::legacy::repository_model::error_tag::{NewErrorTag, SyncedErrorTag};
use crate::domain::repository::legacy::ErrorTagRepository;
use crate::domain::repository::{RepositoryFactory, RepositoryTransactionExecutor};
use crate::AppState;
use std::collections::HashMap;
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub async fn create_error_tags_for_question(
    state: State<'_, AppState>,
    input: CreateErrorTagsForQuestionInput,
) -> Result<Vec<TagOutput>, String> {
    let now = chrono::Utc::now().timestamp();
    let question_id = input.question_id.clone();
    state
        .repository_transaction_executor
        .execute(|factory, _| {
            Box::pin(async move {
                Ok(factory
                    .legacy_error_tag_repository()
                    .create_many(
                        input
                            .tags
                            .into_iter()
                            .map(|tag| NewErrorTag {
                                id: Uuid::new_v4().to_string(),
                                question_id: input.question_id.clone(),
                                name: tag.name,
                                color: tag.color,
                                now,
                            })
                            .collect(),
                    )
                    .await
                    .into_iter()
                    .map(|tag| TagOutput::new(tag, question_id.clone()))
                    .collect())
            })
        })
        .await
}

#[tauri::command]
pub async fn get_error_tags(state: State<'_, AppState>) -> Result<Vec<TagOutput>, String> {
    state
        .repository_transaction_executor
        .execute(|factory, _| {
            Box::pin(async move {
                Ok(factory
                    .legacy_error_tag_repository()
                    .list_active()
                    .await
                    .into_iter()
                    .fold(HashMap::new(), |mut map, tag| {
                        map.entry(tag.name.clone()).or_insert(tag);
                        map
                    })
                    .into_values()
                    .map(|tag| TagOutput::new(tag, String::new()))
                    .collect())
            })
        })
        .await
}

#[tauri::command]
pub async fn get_full_error_tags(state: State<'_, AppState>) -> Result<Vec<TagOutput>, String> {
    state
        .repository_transaction_executor
        .execute(|factory, _| {
            Box::pin(async move {
                Ok(factory
                    .legacy_error_tag_repository()
                    .list_active_with_questions()
                    .await
                    .into_iter()
                    .map(|record| {
                        TagOutput::new(record.tag, record.question_id.unwrap_or_default())
                    })
                    .collect())
            })
        })
        .await
}

#[tauri::command]
pub async fn get_error_tags_for_question(
    state: State<'_, AppState>,
    question_id: String,
) -> Result<Vec<TagOutput>, String> {
    let owner = question_id.clone();
    state
        .repository_transaction_executor
        .execute(|factory, _| {
            Box::pin(async move {
                Ok(factory
                    .legacy_error_tag_repository()
                    .list_active_by_question(question_id)
                    .await
                    .into_iter()
                    .map(|tag| TagOutput::new(tag, owner.clone()))
                    .collect())
            })
        })
        .await
}

#[tauri::command]
pub async fn delete_error_tag(
    state: State<'_, AppState>,
    question_id: String,
    tag_id: String,
) -> Result<(), String> {
    state
        .repository_transaction_executor
        .execute(|factory, _| {
            Box::pin(async move {
                factory
                    .legacy_error_tag_repository()
                    .unlink(question_id, tag_id, chrono::Utc::now().timestamp())
                    .await
            })
        })
        .await
}

#[tauri::command]
pub async fn upsert_error_tag(
    state: State<'_, AppState>,
    input: UpsertErrorTagInput,
) -> Result<(), String> {
    let mut question_ids = input.question_ids;
    if let Some(question_id) = input.question_id {
        question_ids.push(question_id);
    }
    question_ids.sort();
    question_ids.dedup();
    state
        .repository_transaction_executor
        .execute(|factory, _| {
            Box::pin(async move {
                factory
                    .legacy_error_tag_repository()
                    .upsert_synced(SyncedErrorTag {
                        id: input.id,
                        version: input.version,
                        deleted_at: input.deleted_at,
                        question_ids,
                        name: input.name,
                        color: input.color,
                        now: chrono::Utc::now().timestamp(),
                    })
                    .await?;
                Ok(())
            })
        })
        .await
}

#[tauri::command]
pub async fn update_error_tag_by_name(
    state: State<'_, AppState>,
    old_name: String,
    new_name: String,
    new_color: String,
) -> Result<(), String> {
    state
        .repository_transaction_executor
        .execute(|factory, _| {
            Box::pin(async move {
                factory
                    .legacy_error_tag_repository()
                    .update_by_name(
                        old_name,
                        new_name,
                        new_color,
                        chrono::Utc::now().timestamp(),
                    )
                    .await?;
                Ok(())
            })
        })
        .await
}

#[tauri::command]
pub async fn update_error_tag_by_id(
    state: State<'_, AppState>,
    tag_id: String,
    new_tag_name: String,
    new_tag_color: Option<String>,
) -> Result<(), String> {
    state
        .repository_transaction_executor
        .execute(|factory, _| {
            Box::pin(async move {
                factory
                    .legacy_error_tag_repository()
                    .update_by_id(
                        tag_id,
                        new_tag_name,
                        new_tag_color,
                        chrono::Utc::now().timestamp(),
                    )
                    .await?;
                Ok(())
            })
        })
        .await
}
