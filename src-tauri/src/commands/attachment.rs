// 附件相关命令

use crate::AppState;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use tauri::State;
use uuid::Uuid;

use crate::database::entities::{attachment, prelude::Attachment};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CreateAttachmentInput {
    pub question_id: String,
    pub type_: String,
    pub file_type: String,
    pub base64_data: String,
}

/// 为指定题目创建附件
#[tauri::command]
pub async fn create_attachment(
    state: State<'_, AppState>,
    input: CreateAttachmentInput,
) -> Result<attachment::Model, String> {
    let db = state.db.as_ref();
    let now = chrono::Utc::now().timestamp();

    // 使用UUID的前8位作为简单的hash标识
    let id = Uuid::new_v4().to_string();
    let hash = id[..8].to_string();

    let new_attachment = attachment::ActiveModel {
        id: Set(id.clone()),
        question_id: Set(input.question_id),
        type_: Set(input.type_),
        file_type: Set(input.file_type),
        base64_data: Set(input.base64_data),
        hash: Set(hash),
        created_at: Set(now),
        updated_at: Set(now),
        deleted_at: Set(None),
        version: Set(0),
        sync_status: Set("pending".to_string()),
        sync_hash: Set(None),
    };

    let _ = new_attachment
        .insert(db)
        .await;

    let attachment_model = Attachment::find_by_id(id)
        .one(db)
        .await
        .map_err(|e: sea_orm::DbErr| e.to_string())?
        .ok_or("Attachment not found".to_string())?;
    Ok(attachment_model)
}

/// 批量为指定题目创建附件
#[tauri::command]
pub async fn create_attachments_for_question(
    state: State<'_, AppState>,
    question_id: String,
    attachments: Vec<CreateAttachmentInput>,
) -> Result<Vec<attachment::Model>, String> {
    let mut created_attachments = Vec::new();

    for mut input in attachments {
        input.question_id = question_id.clone();
        let attachment = create_attachment(state.clone(), input).await?;
        created_attachments.push(attachment);
    }

    Ok(created_attachments)
}

/// 获取指定题目的所有附件
#[tauri::command]
pub async fn get_attachments_by_question(
    state: State<'_, AppState>,
    question_id: String,
) -> Result<Vec<attachment::Model>, String> {
    let db = state.db.as_ref();

    let attachments = Attachment::find()
        .filter(attachment::Column::QuestionId.eq(question_id))
        .filter(attachment::Column::DeletedAt.is_null())
        .all(db)
        .await
        .map_err(|e: sea_orm::DbErr| e.to_string())?;

    Ok(attachments)
}

/// 删除附件（软删除）
#[tauri::command]
pub async fn delete_attachment(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let db = state.db.as_ref();
    let now = chrono::Utc::now().timestamp();

    let attachment = Attachment::find_by_id(id.clone())
        .one(db)
        .await
        .map_err(|e: sea_orm::DbErr| e.to_string())?
        .ok_or("Attachment not found")?;

    let mut attachment: attachment::ActiveModel = attachment.into();

    attachment.deleted_at = Set(Some(now));
    attachment.updated_at = Set(now);
    attachment.version = Set(attachment.version.unwrap() + 1);
    attachment.sync_status = Set("pending".to_string());

    attachment.update(db).await.map_err(|e: sea_orm::DbErr| e.to_string())?;

    Ok(())
}