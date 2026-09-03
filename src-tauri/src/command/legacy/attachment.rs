use super::request::attachment::{CreateAttachmentInput, UpsertAttachmentInput};
use super::response::attachment::AttachmentInterface;
use crate::domain::repository::legacy::repository_model::attachment::{
    NewAttachment, SyncedAttachment,
};
use crate::domain::repository::legacy::AttachmentRepository;
use crate::domain::repository::{RepositoryFactory, RepositoryTransactionExecutor};
use crate::AppState;
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub async fn legacy_create_attachment(
    state: State<'_, AppState>,
    input: CreateAttachmentInput,
) -> Result<AttachmentInterface, String> {
    let id = Uuid::new_v4().to_string();
    let question_id = input.question_id.clone();
    state
        .repository_transaction_executor
        .execute(|factory, _| {
            Box::pin(async move {
                let model = factory
                    .legacy_attachment_repository()
                    .create(NewAttachment {
                        id,
                        question_id: input.question_id,
                        type_: input.type_,
                        file_type: input.file_type,
                        base64_data: input.base64_data,
                        now: chrono::Utc::now().timestamp(),
                    })
                    .await?;
                Ok(AttachmentInterface::from_model(model, question_id))
            })
        })
        .await
}

#[tauri::command]
pub async fn legacy_create_attachments_for_question(
    state: State<'_, AppState>,
    question_id: String,
    attachments: Vec<CreateAttachmentInput>,
) -> Result<Vec<AttachmentInterface>, String> {
    state
        .repository_transaction_executor
        .execute(|factory, _| {
            Box::pin(async move {
                let repository = factory.legacy_attachment_repository();
                let mut created = Vec::with_capacity(attachments.len());
                for input in attachments {
                    let id = Uuid::new_v4().to_string();
                    let model = repository
                        .create(NewAttachment {
                            id,
                            question_id: question_id.clone(),
                            type_: input.type_,
                            file_type: input.file_type,
                            base64_data: input.base64_data,
                            now: chrono::Utc::now().timestamp(),
                        })
                        .await?;
                    created.push(AttachmentInterface::from_model(model, question_id.clone()));
                }
                Ok(created)
            })
        })
        .await
}

#[tauri::command]
pub async fn legacy_get_attachments_by_question(
    state: State<'_, AppState>,
    question_id: String,
) -> Result<Vec<AttachmentInterface>, String> {
    state
        .repository_transaction_executor
        .execute(|factory, _| {
            Box::pin(async move {
                Ok(factory
                    .legacy_attachment_repository()
                    .list_active_by_question(question_id.clone())
                    .await
                    .into_iter()
                    .map(|model| AttachmentInterface::from_model(model, question_id.clone()))
                    .collect())
            })
        })
        .await
}

#[tauri::command]
pub async fn legacy_delete_attachment(
    state: State<'_, AppState>,
    question_id: String,
    id: String,
) -> Result<(), String> {
    state
        .repository_transaction_executor
        .execute(|factory, _| {
            Box::pin(async move {
                factory
                    .legacy_attachment_repository()
                    .unlink(question_id, id, chrono::Utc::now().timestamp())
                    .await
            })
        })
        .await
}

#[tauri::command]
pub async fn legacy_upsert_attachment(
    state: State<'_, AppState>,
    input: UpsertAttachmentInput,
) -> Result<(), String> {
    let mut question_ids = input.question_ids;
    if let Some(question_id) = input.question_id {
        question_ids.push(question_id);
    }
    question_ids.sort();
    question_ids.dedup();
    state
        .repository_transaction_executor
        .execute(|factory, _| {
            Box::pin(async move {
                factory
                    .legacy_attachment_repository()
                    .upsert_synced(SyncedAttachment {
                        id: input.id,
                        version: input.version,
                        deleted_at: input.deleted_at,
                        question_ids,
                        type_: input.attachment_type,
                        file_type: input.file_type,
                        base64_data: input.base64_data,
                        now: chrono::Utc::now().timestamp(),
                    })
                    .await?;
                Ok(())
            })
        })
        .await
}
