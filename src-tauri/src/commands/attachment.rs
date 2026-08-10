use tauri::State;
use uuid::Uuid;

use crate::database::entities::attachment;
use crate::repository::attachment::{NewAttachment, SyncedAttachment};
use crate::AppState;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct CreateAttachmentInput {
    pub question_id: String,
    pub type_: String,
    pub file_type: String,
    pub base64_data: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct AttachmentInterface {
    pub id: String,
    pub question_id: String,
    pub type_: String,
    pub file_type: String,
    pub base64_data: String,
    pub hash: String,
}

impl From<attachment::Model> for AttachmentInterface {
    fn from(model: attachment::Model) -> Self {
        Self {
            id: model.id,
            question_id: model.question_id,
            type_: model.type_,
            file_type: model.file_type,
            base64_data: String::from_utf8(model.base64_data).unwrap_or_default(),
            hash: model.hash,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct UpsertAttachmentInput {
    pub id: String,
    pub version: i32,
    pub status: String,
    pub deleted_at: Option<i64>,
    pub question_id: String,
    pub type_: String,
    pub file_type: String,
    pub base64_data: Vec<u8>,
    pub hash: String,
}

#[tauri::command]
pub async fn create_attachment(
    state: State<'_, AppState>,
    input: CreateAttachmentInput,
) -> Result<AttachmentInterface, String> {
    let id = Uuid::new_v4().to_string();
    let model = state
        .repositories
        .attachments
        .create(NewAttachment {
            hash: id[..8].to_string(),
            id,
            question_id: input.question_id,
            type_: input.type_,
            file_type: input.file_type,
            base64_data: input.base64_data.into_bytes(),
            now: chrono::Utc::now().timestamp(),
        })
        .await?;
    Ok(model.into())
}

#[tauri::command]
pub async fn create_attachments_for_question(
    state: State<'_, AppState>,
    question_id: String,
    attachments: Vec<CreateAttachmentInput>,
) -> Result<Vec<AttachmentInterface>, String> {
    let mut created = Vec::with_capacity(attachments.len());
    for input in attachments {
        created.push(
            create_attachment(
                state.clone(),
                CreateAttachmentInput {
                    question_id: question_id.clone(),
                    ..input
                },
            )
            .await?,
        );
    }
    Ok(created)
}

#[tauri::command]
pub async fn get_attachments_by_question(
    state: State<'_, AppState>,
    question_id: String,
) -> Result<Vec<AttachmentInterface>, String> {
    Ok(state
        .repositories
        .attachments
        .list_active_by_question(question_id)
        .await?
        .into_iter()
        .map(Into::into)
        .collect())
}

#[tauri::command]
pub async fn delete_attachment(state: State<'_, AppState>, id: String) -> Result<(), String> {
    state
        .repositories
        .attachments
        .soft_delete(id, chrono::Utc::now().timestamp())
        .await
        .map_err(Into::into)
}

#[tauri::command]
pub async fn upsert_attachment(
    state: State<'_, AppState>,
    input: UpsertAttachmentInput,
) -> Result<(), String> {
    state
        .repositories
        .attachments
        .upsert_synced(SyncedAttachment {
            id: input.id,
            version: input.version,
            deleted_at: input.deleted_at,
            question_id: input.question_id,
            type_: input.type_,
            file_type: input.file_type,
            base64_data: input.base64_data,
            hash: input.hash,
            now: chrono::Utc::now().timestamp(),
        })
        .await
        .map_err(Into::into)
}
