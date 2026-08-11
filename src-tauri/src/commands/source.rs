use crate::domain;
use crate::repository::source::{NewSource, SourceChanges, SourceValues, SyncedSource};
use crate::AppState;
use tauri::State;
use uuid::Uuid;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct CreateSourceInput {
    pub subject_id: Option<String>,
    pub book: Option<String>,
    pub chapter: Option<String>,
    pub knowledge: Option<String>,
}
#[derive(serde::Serialize, serde::Deserialize)]
pub struct UpdateSourceInput {
    pub id: String,
    pub subject_id: Option<String>,
    pub book: Option<String>,
    pub chapter: Option<String>,
    pub knowledge: Option<String>,
}
#[derive(serde::Serialize, serde::Deserialize)]
pub struct UpsertSourceInput {
    pub id: String,
    pub version: i32,
    pub status: String,
    pub deleted_at: Option<i64>,
    pub question_id: Option<String>,
    pub subject_id: Option<String>,
    pub book: Option<String>,
    pub chapter: Option<String>,
    pub knowledge: Option<String>,
}
#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct SourceFilter {
    pub subject_id: Option<String>,
}
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
) -> Result<Vec<domain::Source>, String> {
    state
        .repositories
        .sources
        .list_active(filter.unwrap_or_default().subject_id)
        .await
        .map_err(Into::into)
}
#[tauri::command]
pub async fn get_source(state: State<'_, AppState>, id: String) -> Result<domain::Source, String> {
    state
        .repositories
        .sources
        .find_by_id(id)
        .await
        .map_err(Into::into)
}
#[tauri::command]
pub async fn get_books(
    state: State<'_, AppState>,
    subject_id: Option<String>,
) -> Result<Vec<String>, String> {
    state
        .repositories
        .sources
        .list_books(subject_id)
        .await
        .map_err(Into::into)
}
#[tauri::command]
pub async fn get_chapters(
    state: State<'_, AppState>,
    subject_id: Option<String>,
    book: String,
) -> Result<Vec<String>, String> {
    state
        .repositories
        .sources
        .list_chapters(subject_id, book)
        .await
        .map_err(Into::into)
}
#[tauri::command]
pub async fn get_knowledges(
    state: State<'_, AppState>,
    subject_id: Option<String>,
    book: String,
    chapter: String,
) -> Result<Vec<String>, String> {
    state
        .repositories
        .sources
        .list_knowledges(subject_id, book, chapter)
        .await
        .map_err(Into::into)
}
#[tauri::command]
pub async fn create_source(
    state: State<'_, AppState>,
    input: CreateSourceInput,
) -> Result<domain::Source, String> {
    state
        .repositories
        .sources
        .create(NewSource {
            id: Uuid::new_v4().to_string(),
            values: values(input),
            now: chrono::Utc::now().timestamp(),
        })
        .await
        .map_err(Into::into)
}
#[tauri::command]
pub async fn update_source(
    state: State<'_, AppState>,
    input: UpdateSourceInput,
) -> Result<domain::Source, String> {
    state
        .repositories
        .sources
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
        .await
        .map_err(Into::into)
}
#[tauri::command]
pub async fn delete_source(state: State<'_, AppState>, id: String) -> Result<(), String> {
    state
        .repositories
        .sources
        .soft_delete(id, chrono::Utc::now().timestamp())
        .await
        .map_err(Into::into)
}
#[tauri::command]
pub async fn get_or_create_source_id(
    state: State<'_, AppState>,
    input: CreateSourceInput,
) -> Result<String, String> {
    let values = values(input);
    if let Some(model) = state
        .repositories
        .sources
        .find_active_exact(&values)
        .await?
    {
        return Ok(model.id);
    }
    Ok(state
        .repositories
        .sources
        .create(NewSource {
            id: Uuid::new_v4().to_string(),
            values,
            now: chrono::Utc::now().timestamp(),
        })
        .await?
        .id)
}
#[tauri::command]
pub async fn upsert_source(
    state: State<'_, AppState>,
    input: UpsertSourceInput,
) -> Result<(), String> {
    state
        .repositories
        .sources
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
        .await
        .map_err(Into::into)
}
