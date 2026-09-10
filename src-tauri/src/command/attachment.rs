use tauri::State;

use crate::application::{self, command::CreateAttachmentCommand, query::GetAttachmentQuery};
use crate::util::{codec::decode_base64, parsing::parse_uuid};
use crate::AppState;

use super::error::ApiError;
use super::request::attachment::{
    CreateAttachmentRequest, DeleteAttachmentRequest, GetAttachmentRequest,
};
use super::response::attachment::{
    CreateAttachmentResponse, DeleteAttachmentResponse, GetAttachmentResponse,
};

const MAX_ATTACHMENT_BYTES: usize = 10 * 1024 * 1024;

#[tauri::command]
pub async fn create_attachment(
    state: State<'_, AppState>,
    request: CreateAttachmentRequest,
) -> Result<CreateAttachmentResponse, ApiError> {
    let data = decode_base64(&request.base64_data).map_err(ApiError::invalid)?;
    if data.len() > MAX_ATTACHMENT_BYTES {
        return Err(ApiError::invalid("attachment exceeds the 10 MiB limit"));
    }
    let detected_mime_type = infer::get(&data)
        .map(|kind| kind.mime_type())
        .ok_or_else(|| ApiError::invalid("attachment type could not be detected"))?;
    if !detected_mime_type.starts_with("image/") || detected_mime_type != request.mime_type {
        return Err(ApiError::invalid(format!(
            "attachment MIME type must match detected type {detected_mime_type}"
        )));
    }
    let id = application::attachment::create_attachment(
        &state.repository_transaction_executor,
        CreateAttachmentCommand {
            mime_type: detected_mime_type.into(),
            data,
        },
    )
    .await?;
    Ok(CreateAttachmentResponse { id: id.to_string() })
}

#[tauri::command]
pub async fn delete_attachment(
    state: State<'_, AppState>,
    request: DeleteAttachmentRequest,
) -> Result<DeleteAttachmentResponse, ApiError> {
    let id = parse_uuid(&request.id, "id")?;
    application::attachment::delete_attachment(
        &state.repository_transaction_executor,
        crate::application::command::DeleteAttachmentCommand { id },
    )
    .await?;
    Ok(DeleteAttachmentResponse { id: id.to_string() })
}

#[tauri::command]
pub async fn get_attachment(
    state: State<'_, AppState>,
    request: GetAttachmentRequest,
) -> Result<GetAttachmentResponse, ApiError> {
    Ok(GetAttachmentResponse {
        attachment: application::attachment::get_attachment(
            &state.repository_transaction_executor,
            GetAttachmentQuery::ById(parse_uuid(&request.id, "id")?),
        )
        .await?
        .into(),
    })
}
