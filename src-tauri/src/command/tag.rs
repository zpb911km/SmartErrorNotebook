use tauri::State;

use crate::application::query::ListTagsQuery;
use crate::application::{self, command::*};
use crate::util::parsing::parse_uuid;
use crate::AppState;

use super::error::ApiError;
use super::request::tag::{CreateTagRequest, DeleteTagRequest, UpdateTagRequest};
use super::response::tag::{
    CreateTagResponse, DeleteTagResponse, ListTagsResponse, UpdateTagResponse,
};

#[tauri::command]
pub async fn create_tag(
    state: State<'_, AppState>,
    request: CreateTagRequest,
) -> Result<CreateTagResponse, ApiError> {
    let tag = application::tag::create_tag(
        &state.repository_transaction_executor,
        CreateTagCommand {
            name: request.name,
            color: request.color,
        },
    )
    .await?;
    Ok(CreateTagResponse { tag: tag.into() })
}

#[tauri::command]
pub async fn update_tag(
    state: State<'_, AppState>,
    request: UpdateTagRequest,
) -> Result<UpdateTagResponse, ApiError> {
    let id = parse_uuid(&request.id, "id")?;
    let tag = application::tag::update_tag(
        &state.repository_transaction_executor,
        UpdateTagCommand {
            id,
            name: request.name,
            color: request.color,
        },
    )
    .await?;
    Ok(UpdateTagResponse { tag: tag.into() })
}

#[tauri::command]
pub async fn delete_tag(
    state: State<'_, AppState>,
    request: DeleteTagRequest,
) -> Result<DeleteTagResponse, ApiError> {
    let id = parse_uuid(&request.id, "id")?;
    application::tag::delete_tag(
        &state.repository_transaction_executor,
        DeleteTagCommand { id },
    )
    .await?;
    Ok(DeleteTagResponse { id: id.to_string() })
}

#[tauri::command]
pub async fn list_tags(state: State<'_, AppState>) -> Result<ListTagsResponse, ApiError> {
    Ok(ListTagsResponse {
        tags: application::tag::list_tags(
            &state.repository_transaction_executor,
            ListTagsQuery::All,
        )
        .await?
        .into_iter()
        .map(Into::into)
        .collect(),
    })
}
