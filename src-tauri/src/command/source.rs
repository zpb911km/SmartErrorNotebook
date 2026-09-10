use chrono::Utc;
use tauri::State;

use crate::application::{self, command::*, query::*};
use crate::util::parsing::parse_uuid;
use crate::AppState;

use super::error::ApiError;
use super::request::source::{
    CreateSourceRequest, DeleteSourceRequest, DeleteSourcesRequest, GetSourceRequest,
    ListSourcesRequest, UpdateSourceRequest,
};
use super::response::source::{
    CreateSourceResponse, DeleteSourceResponse, DeleteSourcesResponse, GetSourceResponse,
    ListSourcesResponse, UpdateSourceResponse,
};

#[tauri::command]
pub async fn create_source(
    state: State<'_, AppState>,
    request: CreateSourceRequest,
) -> Result<CreateSourceResponse, ApiError> {
    let subject_id = request
        .subject_id
        .map(|value| parse_uuid(&value, "subjectId"))
        .transpose()?;
    let source = application::source::create_source(
        &state.repository_transaction_executor,
        CreateSourceCommand {
            subject_id,
            book: request.book,
            chapter: request.chapter,
            knowledge: request.knowledge,
        },
    )
    .await?;
    Ok(CreateSourceResponse {
        source: source.into(),
    })
}

#[tauri::command]
pub async fn update_source(
    state: State<'_, AppState>,
    request: UpdateSourceRequest,
) -> Result<UpdateSourceResponse, ApiError> {
    let id = parse_uuid(&request.id, "id")?;
    let subject_id = request
        .subject_id
        .map(|value| parse_uuid(&value, "subjectId"))
        .transpose()?;
    let source = application::source::update_source(
        &state.repository_transaction_executor,
        UpdateSourceCommand {
            id,
            subject_id,
            book: request.book,
            chapter: request.chapter,
            knowledge: request.knowledge,
        },
    )
    .await?;
    Ok(UpdateSourceResponse {
        source: source.into(),
    })
}

#[tauri::command]
pub async fn delete_source(
    state: State<'_, AppState>,
    request: DeleteSourceRequest,
) -> Result<DeleteSourceResponse, ApiError> {
    let id = parse_uuid(&request.id, "id")?;
    application::source::delete_source(
        &state.repository_transaction_executor,
        DeleteSourceCommand {
            id,
            deleted_at: Utc::now(),
        },
    )
    .await?;
    Ok(DeleteSourceResponse { id: id.to_string() })
}

#[tauri::command]
pub async fn delete_sources(
    state: State<'_, AppState>,
    request: DeleteSourcesRequest,
) -> Result<DeleteSourcesResponse, ApiError> {
    let mut ids = request
        .ids
        .into_iter()
        .map(|value| parse_uuid(&value, "ids"))
        .collect::<Result<Vec<_>, _>>()?;
    ids.sort_unstable();
    ids.dedup();
    application::source::delete_sources(
        &state.repository_transaction_executor,
        DeleteSourcesCommand {
            ids: ids.clone(),
            deleted_at: Utc::now(),
        },
    )
    .await?;
    Ok(DeleteSourcesResponse {
        ids: ids.into_iter().map(|id| id.to_string()).collect(),
    })
}

#[tauri::command]
pub async fn get_source(
    state: State<'_, AppState>,
    request: GetSourceRequest,
) -> Result<GetSourceResponse, ApiError> {
    Ok(GetSourceResponse {
        source: application::source::get_source(
            &state.repository_transaction_executor,
            GetSourceQuery::ById(parse_uuid(&request.id, "id")?),
        )
        .await?
        .into(),
    })
}

#[tauri::command]
pub async fn list_sources(
    state: State<'_, AppState>,
    request: ListSourcesRequest,
) -> Result<ListSourcesResponse, ApiError> {
    let query = match request
        .subject_id
        .map(|value| parse_uuid(&value, "subjectId"))
        .transpose()?
    {
        Some(subject_id) => ListSourcesQuery::BySubjectId(subject_id),
        None => ListSourcesQuery::All,
    };
    Ok(ListSourcesResponse {
        sources: application::source::list_sources(&state.repository_transaction_executor, query)
            .await?
            .into_iter()
            .map(Into::into)
            .collect(),
    })
}
