use chrono::Utc;
use tauri::State;

use crate::application::{self, command::*};
use crate::util::parsing::parse_uuid;
use crate::AppState;

use super::error::ApiError;
use super::request::subject::{CreateSubjectRequest, DeleteSubjectRequest, UpdateSubjectRequest};
use super::response::subject::{
    CreateSubjectResponse, DeleteSubjectResponse, ListSubjectsResponse, UpdateSubjectResponse,
};

#[tauri::command]
pub async fn create_subject(
    state: State<'_, AppState>,
    request: CreateSubjectRequest,
) -> Result<CreateSubjectResponse, ApiError> {
    let subject = application::subject::create_subject(
        &state.repository_transaction_executor,
        CreateSubjectCommand {
            name: request.name,
            color: request.color,
        },
    )
    .await?;
    Ok(CreateSubjectResponse {
        subject: subject.into(),
    })
}

#[tauri::command]
pub async fn update_subject(
    state: State<'_, AppState>,
    request: UpdateSubjectRequest,
) -> Result<UpdateSubjectResponse, ApiError> {
    let id = parse_uuid(&request.id, "id")?;
    let subject = application::subject::update_subject(
        &state.repository_transaction_executor,
        UpdateSubjectCommand {
            id,
            name: request.name,
            color: request.color,
        },
    )
    .await?;
    Ok(UpdateSubjectResponse {
        subject: subject.into(),
    })
}

#[tauri::command]
pub async fn delete_subject(
    state: State<'_, AppState>,
    request: DeleteSubjectRequest,
) -> Result<DeleteSubjectResponse, ApiError> {
    let id = parse_uuid(&request.id, "id")?;
    application::subject::delete_subject(
        &state.repository_transaction_executor,
        DeleteSubjectCommand {
            id,
            deleted_at: Utc::now(),
        },
    )
    .await?;
    Ok(DeleteSubjectResponse { id: id.to_string() })
}

#[tauri::command]
pub async fn list_subjects(state: State<'_, AppState>) -> Result<ListSubjectsResponse, ApiError> {
    Ok(ListSubjectsResponse {
        subjects: application::subject::list_subjects(&state.repository_transaction_executor)
            .await?
            .into_iter()
            .map(Into::into)
            .collect(),
    })
}
