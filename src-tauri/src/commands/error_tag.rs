// 错因标签相关命令

use crate::AppState;
use sea_orm::ColumnTrait;
use sea_orm::QueryFilter;
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use tauri::State;
use uuid::Uuid;

use crate::database::entities::{error_tag, prelude::ErrorTag};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct TagInfo {
    pub name: String,
    pub color: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CreateErrorTagsForQuestionInput {
    pub question_id: String,
    pub tags: Vec<TagInfo>,
}

// 用于同步的 UPSERT 输入
#[derive(serde::Serialize, serde::Deserialize)]
pub struct UpsertErrorTagInput {
    pub id: String,
    pub version: i32,
    pub status: String,
    pub deleted_at: Option<i64>,
    pub question_id: String,
    pub name: String,
    pub color: String,
}

/// 为指定题目批量创建错因标签
#[tauri::command]
pub async fn create_error_tags_for_question(
    state: State<'_, AppState>,
    input: CreateErrorTagsForQuestionInput,
) -> Result<Vec<error_tag::Model>, String> {
    let db = state.db.as_ref();
    let now = chrono::Utc::now().timestamp();
    let mut created_tags = Vec::new();

    for tag_info in input.tags {
        let id = Uuid::new_v4().to_string();
        let new_tag = error_tag::ActiveModel {
            id: Set(id.clone()),
            question_id: Set(input.question_id.clone()),
            name: Set(tag_info.name),
            color: Set(tag_info.color),
            created_at: Set(now),
            updated_at: Set(now),
            deleted_at: Set(None),
            version: Set(0),
            sync_status: Set("pending".to_string()),
            sync_hash: Set(None),
        };

        let _ = new_tag.insert(db).await;

        // 将ActiveModel转换为Model
        let tag_model = ErrorTag::find_by_id(id)
            .one(db)
            .await
            .map_err(|e: sea_orm::DbErr| e.to_string())?
            .ok_or("没有成功添加".to_string())?;

        created_tags.push(tag_model);
    }

    Ok(created_tags)
}

/// 获取全部的错因标签
#[tauri::command]
pub async fn get_error_tags(state: State<'_, AppState>) -> Result<Vec<error_tag::Model>, String> {
    let db = state.db.as_ref();
    let tags = ErrorTag::find()
        .all(db)
        .await
        .map_err(|e: sea_orm::DbErr| e.to_string())?;

    // 按名称去重，只保留每个名称的第一个标签
    let unique_tags: Vec<error_tag::Model> = tags
        .into_iter()
        .fold(std::collections::HashMap::new(), |mut acc, tag| {
            acc.entry(tag.name.clone()).or_insert(tag);
            acc
        })
        .into_values()
        .collect();

    Ok(unique_tags)
}

#[tauri::command]
pub async fn get_full_error_tags(state: State<'_, AppState>) -> Result<Vec<error_tag::Model>, String> {
    let db = state.db.as_ref();
    let tags = ErrorTag::find()
        .all(db)
        .await
        .map_err(|e: sea_orm::DbErr| e.to_string())?;
    Ok(tags)
}

// 获取单一题目的错因标签
#[tauri::command]
pub async fn get_error_tags_for_question(
    state: State<'_, AppState>,
    question_id: String,
) -> Result<Vec<error_tag::Model>, String> {
    let db = state.db.as_ref();
    let tags = ErrorTag::find()
        .filter(error_tag::Column::QuestionId.eq(question_id))
        .all(db)
        .await
        .map_err(|e: sea_orm::DbErr| e.to_string())?;
    Ok(tags)
}


// 删除错因标签 by id, 软删除
#[tauri::command]
pub async fn delete_error_tag(state: State<'_, AppState>, tag_id: String) -> Result<(), String> {
    let db = state.db.as_ref();
    let tag = ErrorTag::find_by_id(tag_id)
        .one(db)
        .await
        .map_err(|e: sea_orm::DbErr| e.to_string())?;

    if let Some(tag) = tag {
        let mut active_model: error_tag::ActiveModel = tag.into();
        active_model.deleted_at = Set(Some(chrono::Utc::now().timestamp()));
        active_model.sync_status = Set("pending".to_string());
        active_model.update(db).await.map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// UPSERT: 根据 ID 插入或更新错因标签（用于同步）
#[tauri::command]
pub async fn upsert_error_tag(
    state: State<'_, AppState>,
    input: UpsertErrorTagInput,
) -> Result<(), String> {
    use crate::database::entities::error_tag as tag;

    let db = state.db.as_ref();
    let now = chrono::Utc::now().timestamp();

    // 检查记录是否存在
    let existing = tag::Entity::find_by_id(input.id.clone())
        .one(db)
        .await
        .map_err(|e| format!("Query failed: {}", e))?;

    if let Some(model) = existing {
        // 更新现有记录
        let mut active_model: error_tag::ActiveModel = model.into();
        active_model.question_id = Set(input.question_id);
        active_model.name = Set(input.name);
        active_model.color = Set(input.color);
        active_model.updated_at = Set(now);
        active_model.version = Set(input.version);
        active_model.sync_status = Set("synced".to_string());
        active_model.deleted_at = Set(input.deleted_at);

        active_model.update(db).await.map_err(|e| e.to_string())?;
    } else {
        // 插入新记录
        let new_tag = error_tag::ActiveModel {
            id: Set(input.id),
            question_id: Set(input.question_id),
            name: Set(input.name),
            color: Set(input.color),
            created_at: Set(now),
            updated_at: Set(now),
            deleted_at: Set(input.deleted_at),
            version: Set(input.version),
            sync_status: Set("synced".to_string()),
            sync_hash: Set(None),
        };

        let _ = new_tag.insert(db).await;
    }

    Ok(())
}

// 修改错因记录,通过id定位
#[tauri::command]
pub async fn update_error_tag_by_id(
    state: State<'_, AppState>,
    tag_id: String,
    new_tag_name: String,
    new_tag_color: Option<String>,
) -> Result<(), String> {
    let db = state.db.as_ref();
    let cur_model = ErrorTag::find_by_id(tag_id)
        .one(db)
        .await
        .map_err(|e: sea_orm::DbErr| e.to_string())?
        .ok_or("标签不存在".to_string())?;
    let mut active_model: error_tag::ActiveModel = cur_model.into();
    active_model.name = Set(new_tag_name);
    if let Some(color) = new_tag_color {
        active_model.color = Set(color);
    }
    active_model.updated_at = Set(chrono::Utc::now().timestamp());
    active_model.sync_status = Set("pending".to_string());
    let _ = active_model.update(db)
        .await
        .map_err(|e: sea_orm::DbErr| e.to_string());
    Ok(())
}

