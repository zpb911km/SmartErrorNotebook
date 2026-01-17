// 科目相关命令

use crate::AppState;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use tauri::State;
use uuid::Uuid;

use crate::database::entities::{prelude::Subject, subject};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CreateSubjectInput {
    pub name: String,
    pub color: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct UpdateSubjectInput {
    pub id: String,
    pub name: Option<String>,
    pub color: Option<String>,
}

/// 获取所有科目
#[tauri::command]
pub async fn get_subjects(state: State<'_, AppState>) -> Result<Vec<subject::Model>, String> {
    let db = state.db.as_ref();

    let subjects = Subject::find()
        .filter(subject::Column::DeletedAt.is_null())
        .all(db)
        .await
        .map_err(|e| e.to_string())?;

    Ok(subjects)
}

/// 创建科目
#[tauri::command]
pub async fn create_subject(
    state: State<'_, AppState>,
    input: CreateSubjectInput,
) -> Result<subject::Model, String> {
    let db = state.db.as_ref();
    let now = chrono::Utc::now().timestamp();

    let new_subject = subject::ActiveModel {
        id: Set(Uuid::new_v4().to_string()),
        name: Set(input.name),
        color: Set(input.color),
        created_at: Set(now),
        updated_at: Set(now),
        deleted_at: Set(None),
        version: Set(0),
        sync_status: Set("pending".to_string()),
        sync_hash: Set(None),
    };

    let subject = new_subject.insert(db).await.map_err(|e| e.to_string())?;

    Ok(subject)
}

/// 更新科目
#[tauri::command]
pub async fn update_subject(
    state: State<'_, AppState>,
    input: UpdateSubjectInput,
) -> Result<subject::Model, String> {
    let db = state.db.as_ref();
    let now = chrono::Utc::now().timestamp();

    let subject = Subject::find_by_id(input.id.clone())
        .one(db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or("Subject not found")?;

    let mut subject: subject::ActiveModel = subject.into();

    if let Some(name) = input.name {
        subject.name = Set(name);
    }
    if let Some(color) = input.color {
        subject.color = Set(Some(color));
    }

    subject.updated_at = Set(now);
    subject.version = Set(subject.version.unwrap() + 1);
    subject.sync_status = Set("pending".to_string());

    let subject = subject.update(db).await.map_err(|e| e.to_string())?;

    Ok(subject)
}

/// 删除科目（软删除）
#[tauri::command]
pub async fn delete_subject(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let db = state.db.as_ref();
    let now = chrono::Utc::now().timestamp();

    let subject = Subject::find_by_id(id.clone())
        .one(db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or("Subject not found")?;

    let mut subject: subject::ActiveModel = subject.into();

    subject.deleted_at = Set(Some(now));
    subject.updated_at = Set(now);
    subject.version = Set(subject.version.unwrap() + 1);
    subject.sync_status = Set("pending".to_string());

    subject.update(db).await.map_err(|e| e.to_string())?;

    Ok(())
}