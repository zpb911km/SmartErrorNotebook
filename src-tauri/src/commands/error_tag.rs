// 错因标签相关命令

use crate::AppState;
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use serde::de::value::Error;
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

        let _ = new_tag
            .insert(db)
            .await;

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
pub async fn get_error_tags(
    state: State<'_, AppState>,
) -> Result<Vec<error_tag::Model>, String> {
    let db = state.db.as_ref();
    let tags = ErrorTag::find()
        .all(db)
        .await
        .map_err(|e: sea_orm::DbErr| e.to_string())?;
    Ok(tags)
}