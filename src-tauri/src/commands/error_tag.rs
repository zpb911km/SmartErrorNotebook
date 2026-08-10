use crate::database::entities::error_tag;
use crate::repository::error_tag::{NewErrorTag, SyncedErrorTag};
use crate::AppState;
use std::collections::HashMap;
use tauri::State;
use uuid::Uuid;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct TagInfo {
    pub name: String,
    pub color: String,
}
#[derive(serde::Serialize, serde::Deserialize)]
pub struct CreateErrorTagsForQuestionInput {
    pub question_id: String,
    pub tags: Vec<TagInfo>,
}
#[derive(serde::Serialize, serde::Deserialize)]
pub struct UpsertErrorTagInput {
    pub id: String,
    pub version: i32,
    pub status: String,
    pub deleted_at: Option<i64>,
    pub question_id: String,
    pub name: String,
    pub color: String,
}

#[tauri::command]
pub async fn create_error_tags_for_question(
    state: State<'_, AppState>,
    input: CreateErrorTagsForQuestionInput,
) -> Result<Vec<error_tag::Model>, String> {
    let now = chrono::Utc::now().timestamp();
    state
        .repositories
        .error_tags
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
        .map_err(Into::into)
}
#[tauri::command]
pub async fn get_error_tags(state: State<'_, AppState>) -> Result<Vec<error_tag::Model>, String> {
    Ok(state
        .repositories
        .error_tags
        .list_active()
        .await?
        .into_iter()
        .fold(HashMap::new(), |mut map, tag| {
            map.entry(tag.name.clone()).or_insert(tag);
            map
        })
        .into_values()
        .collect())
}
#[tauri::command]
pub async fn get_full_error_tags(
    state: State<'_, AppState>,
) -> Result<Vec<error_tag::Model>, String> {
    state
        .repositories
        .error_tags
        .list_active()
        .await
        .map_err(Into::into)
}
#[tauri::command]
pub async fn get_error_tags_for_question(
    state: State<'_, AppState>,
    question_id: String,
) -> Result<Vec<error_tag::Model>, String> {
    state
        .repositories
        .error_tags
        .list_active_by_question(question_id)
        .await
        .map_err(Into::into)
}
#[tauri::command]
pub async fn delete_error_tag(state: State<'_, AppState>, tag_id: String) -> Result<(), String> {
    state
        .repositories
        .error_tags
        .soft_delete(tag_id, chrono::Utc::now().timestamp())
        .await
        .map_err(Into::into)
}
#[tauri::command]
pub async fn upsert_error_tag(
    state: State<'_, AppState>,
    input: UpsertErrorTagInput,
) -> Result<(), String> {
    state
        .repositories
        .error_tags
        .upsert_synced(SyncedErrorTag {
            id: input.id,
            version: input.version,
            deleted_at: input.deleted_at,
            question_id: input.question_id,
            name: input.name,
            color: input.color,
            now: chrono::Utc::now().timestamp(),
        })
        .await
        .map_err(Into::into)
}
#[tauri::command]
pub async fn update_error_tag_by_name(
    state: State<'_, AppState>,
    old_name: String,
    new_name: String,
    new_color: String,
) -> Result<(), String> {
    state
        .repositories
        .error_tags
        .update_by_name(
            old_name,
            new_name,
            new_color,
            chrono::Utc::now().timestamp(),
        )
        .await
        .map_err(Into::into)
}
#[tauri::command]
pub async fn update_error_tag_by_id(
    state: State<'_, AppState>,
    tag_id: String,
    new_tag_name: String,
    new_tag_color: Option<String>,
) -> Result<(), String> {
    state
        .repositories
        .error_tags
        .update_by_id(
            tag_id,
            new_tag_name,
            new_tag_color,
            chrono::Utc::now().timestamp(),
        )
        .await
        .map_err(Into::into)
}
