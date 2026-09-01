use super::request::error_question::{
    CreateQuestionInput, QuestionFilter, UpdateQuestionInput, UpsertQuestionInput,
};
use super::response::error_question::{ErrorQuestionOutput, QuestionStats};
use crate::domain::model::Question;
use crate::domain::repository::legacy::repository_model::error_question::{
    NewQuestion, QuestionChanges, QuestionQuery, SyncedQuestion,
};
use crate::domain::repository::legacy::{ErrorQuestionRepository, SourceRepository};
use crate::domain::repository::{RepositoryFactory, RepositoryTransactionExecutor};
use crate::AppState;
use tauri::State;
use uuid::Uuid;

async fn output(factory: &impl RepositoryFactory, question: Question) -> ErrorQuestionOutput {
    let subject_id = if let Some(source_id) = question.source_id {
        factory
            .legacy_source_repository()
            .find_by_id(source_id.to_string())
            .await
            .source
            .subject_id
            .map(|id| id.to_string())
    } else {
        None
    };
    ErrorQuestionOutput::new(question, subject_id)
}

#[tauri::command]
pub async fn get_questions(
    state: State<'_, AppState>,
    filter: Option<QuestionFilter>,
) -> Result<Vec<ErrorQuestionOutput>, String> {
    let f = filter.unwrap_or_default();
    state
        .repository_transaction_executor
        .execute(|factory, _| {
            Box::pin(async move {
                let questions = factory
                    .legacy_error_question_repository()
                    .list_active(QuestionQuery {
                        subject_id: f.subject_id,
                        search: f.search,
                        limit: f.limit,
                        offset: f.offset,
                    })
                    .await;
                let mut outputs = Vec::with_capacity(questions.len());
                for question in questions {
                    outputs.push(output(&factory, question).await);
                }
                Ok(outputs)
            })
        })
        .await
}

#[tauri::command]
pub async fn get_question(
    state: State<'_, AppState>,
    id: String,
) -> Result<ErrorQuestionOutput, String> {
    state
        .repository_transaction_executor
        .execute(|factory, _| {
            Box::pin(async move {
                let question = factory
                    .legacy_error_question_repository()
                    .find_by_id(id)
                    .await;
                Ok(output(&factory, question).await)
            })
        })
        .await
}

#[tauri::command]
pub async fn create_question(
    state: State<'_, AppState>,
    input: CreateQuestionInput,
) -> Result<ErrorQuestionOutput, String> {
    state
        .repository_transaction_executor
        .execute(|factory, _| {
            Box::pin(async move {
                let question = factory
                    .legacy_error_question_repository()
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
                    .await;
                Ok(output(&factory, question).await)
            })
        })
        .await
}

#[tauri::command]
pub async fn update_question(
    state: State<'_, AppState>,
    input: UpdateQuestionInput,
) -> Result<ErrorQuestionOutput, String> {
    state
        .repository_transaction_executor
        .execute(|factory, _| {
            Box::pin(async move {
                let question = factory
                    .legacy_error_question_repository()
                    .update(QuestionChanges {
                        id: input.id,
                        subject_id: input.subject_id,
                        source_id: input.source_id,
                        prompt: input.prompt,
                        type_: input.type_,
                        answer: input.answer,
                        analysis: input.analysis,
                        error_note: input.error_note,
                        now: chrono::Utc::now().timestamp(),
                    })
                    .await;
                Ok(output(&factory, question).await)
            })
        })
        .await
}

#[tauri::command]
pub async fn delete_question(state: State<'_, AppState>, id: String) -> Result<(), String> {
    state
        .repository_transaction_executor
        .execute(|factory, _| {
            Box::pin(async move {
                factory
                    .legacy_error_question_repository()
                    .soft_delete_with_srs(id, chrono::Utc::now().timestamp())
                    .await;
                Ok(())
            })
        })
        .await
}

#[tauri::command]
pub async fn get_question_stats(state: State<'_, AppState>) -> Result<QuestionStats, String> {
    state
        .repository_transaction_executor
        .execute(|factory, _| {
            Box::pin(async move {
                Ok(QuestionStats {
                    total: factory
                        .legacy_error_question_repository()
                        .count_active()
                        .await,
                })
            })
        })
        .await
}

#[tauri::command]
pub async fn upsert_error_question(
    state: State<'_, AppState>,
    input: UpsertQuestionInput,
) -> Result<(), String> {
    state
        .repository_transaction_executor
        .execute(|factory, _| {
            Box::pin(async move {
                factory
                    .legacy_error_question_repository()
                    .upsert_synced(SyncedQuestion {
                        id: input.id,
                        version: input.version,
                        deleted_at: input.deleted_at,
                        user_id: input.user_id,
                        subject_id: input.subject_id,
                        source_id: input.source_id,
                        prompt: input.prompt,
                        type_: input.type_,
                        answer: input.answer,
                        analysis: input.analysis,
                        error_note: input.error_note,
                        sync_hash: input.sync_hash,
                        now: chrono::Utc::now().timestamp(),
                    })
                    .await;
                Ok(())
            })
        })
        .await
}
