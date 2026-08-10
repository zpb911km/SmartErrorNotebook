use crate::database::entities::error_question;
use crate::repository::error_question::{
    NewQuestion, QuestionChanges, QuestionQuery, SyncedQuestion,
};
use crate::AppState;
use tauri::State;
use uuid::Uuid;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CreateQuestionInput {
    pub user_id: String,
    pub subject_id: String,
    pub source_id: Option<String>,
    pub prompt: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub answer: Option<String>,
    pub analysis: Option<String>,
    pub error_note: Option<String>,
}
#[derive(serde::Serialize, serde::Deserialize)]
pub struct UpdateQuestionInput {
    pub id: String,
    pub subject_id: Option<String>,
    #[serde(alias = "source_id")]
    pub sourceid: Option<String>,
    pub prompt: Option<String>,
    #[serde(rename = "type")]
    pub type_: Option<String>,
    pub answer: Option<String>,
    pub analysis: Option<String>,
    pub error_note: Option<String>,
}
#[derive(serde::Serialize, serde::Deserialize)]
pub struct UpsertQuestionInput {
    pub id: String,
    pub version: i32,
    pub status: String,
    pub deleted_at: Option<i64>,
    pub userid: String,
    #[serde(alias = "subject_id")]
    pub subjectid: String,
    #[serde(alias = "source_id")]
    pub sourceid: Option<String>,
    pub prompt: String,
    #[serde(rename = "type_")]
    pub type_: String,
    pub answer: Option<String>,
    pub analysis: Option<String>,
    pub error_note: Option<String>,
    pub sync_hash: Option<String>,
}
#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct QuestionFilter {
    pub subject_id: Option<String>,
    pub search: Option<String>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}
#[derive(serde::Serialize)]
pub struct QuestionStats {
    pub total: u64,
}

#[tauri::command]
pub async fn get_questions(
    state: State<'_, AppState>,
    filter: Option<QuestionFilter>,
) -> Result<Vec<error_question::Model>, String> {
    let f = filter.unwrap_or_default();
    state
        .repositories
        .error_questions
        .list_active(QuestionQuery {
            subject_id: f.subject_id,
            search: f.search,
            limit: f.limit,
            offset: f.offset,
        })
        .await
        .map_err(Into::into)
}
#[tauri::command]
pub async fn get_question(
    state: State<'_, AppState>,
    id: String,
) -> Result<error_question::Model, String> {
    state
        .repositories
        .error_questions
        .find_by_id(id)
        .await
        .map_err(Into::into)
}
#[tauri::command]
pub async fn create_question(
    state: State<'_, AppState>,
    input: CreateQuestionInput,
) -> Result<error_question::Model, String> {
    state
        .repositories
        .error_questions
        .create(NewQuestion {
            id: Uuid::new_v4().to_string(),
            user_id: input.user_id,
            subject_id: input.subject_id,
            source_id: input.source_id,
            prompt: input.prompt,
            type_: input.type_,
            answer: input.answer,
            analysis: input.analysis,
            error_note: input.error_note,
            now: chrono::Utc::now().timestamp(),
        })
        .await
        .map_err(Into::into)
}
#[tauri::command]
pub async fn update_question(
    state: State<'_, AppState>,
    input: UpdateQuestionInput,
) -> Result<error_question::Model, String> {
    state
        .repositories
        .error_questions
        .update(QuestionChanges {
            id: input.id,
            subject_id: input.subject_id,
            source_id: input.sourceid,
            prompt: input.prompt,
            type_: input.type_,
            answer: input.answer,
            analysis: input.analysis,
            error_note: input.error_note,
            now: chrono::Utc::now().timestamp(),
        })
        .await
        .map_err(Into::into)
}
#[tauri::command]
pub async fn delete_question(state: State<'_, AppState>, id: String) -> Result<(), String> {
    state
        .repositories
        .error_questions
        .soft_delete_with_srs(id, chrono::Utc::now().timestamp())
        .await
        .map_err(Into::into)
}
#[tauri::command]
pub async fn get_question_stats(state: State<'_, AppState>) -> Result<QuestionStats, String> {
    Ok(QuestionStats {
        total: state.repositories.error_questions.count_active().await?,
    })
}
#[tauri::command]
pub async fn upsert_error_question(
    state: State<'_, AppState>,
    input: UpsertQuestionInput,
) -> Result<(), String> {
    state
        .repositories
        .error_questions
        .upsert_synced(SyncedQuestion {
            id: input.id,
            version: input.version,
            deleted_at: input.deleted_at,
            user_id: input.userid,
            subject_id: input.subjectid,
            source_id: input.sourceid,
            prompt: input.prompt,
            type_: input.type_,
            answer: input.answer,
            analysis: input.analysis,
            error_note: input.error_note,
            sync_hash: input.sync_hash,
            now: chrono::Utc::now().timestamp(),
        })
        .await
        .map_err(Into::into)
}
