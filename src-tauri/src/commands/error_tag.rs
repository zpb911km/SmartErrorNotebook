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

// 删除错因标签: 删除错因记录中标签等于给定名称的全部记录
#[tauri::command]
pub async fn delete_error_tag_by_name(
    state: State<'_, AppState>,
    tag_name: String,
) -> Result<u64, String> {
    let db = state.db.as_ref();
    let deleted_result = error_tag::Entity::delete_many()
        .filter(error_tag::Column::Name.eq(tag_name))
        .exec(db)
        .await
        .map_err(|e: sea_orm::DbErr| e.to_string())?;
    let deleted_count = deleted_result.rows_affected;
    Ok(deleted_count)
}

// 改动错因标签: 改动错因记录中标签等于给定名称的全部记录为新名称和颜色
#[tauri::command]
pub async fn update_error_tag_by_name(
    state: State<'_, AppState>,
    tag_name: String,
    new_tag_name: String,
    new_tag_color: Option<String>,
) -> Result<u64, String> {
    let db = state.db.as_ref();
    let mut update_query = error_tag::Entity::update_many();
    update_query = update_query.filter(error_tag::Column::Name.eq(tag_name));
    update_query = update_query.col_expr(
        error_tag::Column::Name,
        sea_orm::prelude::Expr::value(new_tag_name),
    );
    if let Some(color) = new_tag_color {
        update_query = update_query.col_expr(
            error_tag::Column::Color,
            sea_orm::prelude::Expr::value(color),
        );
    }
    let updated_result = update_query.exec(db).await;
    match updated_result {
        Ok(result) => Ok(result.rows_affected),
        Err(e) => Err(e.to_string()),
    }
}

// 删除错因记录,通过id定位
#[tauri::command]
pub async fn delete_error_tag_by_id(
    state: State<'_, AppState>,
    tag_id: String,
) -> Result<u64, String> {
    let db = state.db.as_ref();
    let deleted_result = error_tag::Entity::delete_many()
        .filter(error_tag::Column::Id.eq(tag_id))
        .exec(db)
        .await;
    match deleted_result {
        Ok(result) => Ok(result.rows_affected),
        Err(e) => Err(e.to_string()),
    }
}

// 修改错因记录,通过id定位,不可单独修改颜色,可以单独修改名字
#[tauri::command]
pub async fn update_error_tag_by_id(
    state: State<'_, AppState>,
    tag_id: String,
    new_tag_name: String,
    new_tag_color: Option<String>,
) -> Result<u64, String> {
    let db = state.db.as_ref();
    let mut update_query = error_tag::Entity::update_many();
    update_query = update_query.filter(error_tag::Column::Id.eq(tag_id));
    update_query = update_query.col_expr(
        error_tag::Column::Name,
        sea_orm::prelude::Expr::value(new_tag_name),
    );
    if let Some(color) = new_tag_color {
        update_query = update_query.col_expr(
            error_tag::Column::Color,
            sea_orm::prelude::Expr::value(color),
        );
    }

    let updated_result = update_query.exec(db).await;
    match updated_result {
        Ok(result) => Ok(result.rows_affected),
        Err(e) => Err(e.to_string()),
    }
}
