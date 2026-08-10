use tauri::State;
use uuid::Uuid;

use crate::database::entities::subject;
use crate::repository::subject::{NewSubject, SubjectChanges, SyncedSubject};
use crate::AppState;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CreateSubjectInput {
    pub name: String,
    pub color: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct UpdateSubjectInput {
    pub id: String,
    pub name: Option<String>,
    pub color: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct UpsertSubjectInput {
    pub id: String,
    pub version: i32,
    pub status: String,
    pub deleted_at: Option<i64>,
    pub name: String,
    pub color: Option<String>,
}

#[tauri::command]
pub async fn get_subjects(state: State<'_, AppState>) -> Result<Vec<subject::Model>, String> {
    state
        .repositories
        .subjects
        .list_active()
        .await
        .map_err(Into::into)
}

#[tauri::command]
pub async fn create_subject(
    state: State<'_, AppState>,
    input: CreateSubjectInput,
) -> Result<subject::Model, String> {
    state
        .repositories
        .subjects
        .create(NewSubject {
            id: Uuid::new_v4().to_string(),
            name: input.name,
            color: input.color,
            now: chrono::Utc::now().timestamp(),
        })
        .await
        .map_err(Into::into)
}

#[tauri::command]
pub async fn update_subject(
    state: State<'_, AppState>,
    input: UpdateSubjectInput,
) -> Result<subject::Model, String> {
    state
        .repositories
        .subjects
        .update(SubjectChanges {
            id: input.id,
            name: input.name,
            color: input.color,
            now: chrono::Utc::now().timestamp(),
        })
        .await
        .map_err(Into::into)
}

#[tauri::command]
pub async fn delete_subject(state: State<'_, AppState>, id: String) -> Result<(), String> {
    state
        .repositories
        .subjects
        .soft_delete(id, chrono::Utc::now().timestamp())
        .await
        .map_err(Into::into)
}

#[tauri::command]
pub async fn upsert_subject(
    state: State<'_, AppState>,
    input: UpsertSubjectInput,
) -> Result<(), String> {
    state
        .repositories
        .subjects
        .upsert_synced(SyncedSubject {
            id: input.id,
            version: input.version,
            deleted_at: input.deleted_at,
            name: input.name,
            color: input.color,
            now: chrono::Utc::now().timestamp(),
        })
        .await
        .map_err(Into::into)
}
