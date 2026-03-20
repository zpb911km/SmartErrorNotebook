// SRS数据相关命令

use crate::AppState;
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use tauri::State;
use uuid::Uuid;

use crate::database::entities::{prelude::SrsData, srs_data};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CreateSRSDataInput {
    pub question_id: String,
    pub difficulty: f32,
    pub mastery: f32,
}

/// 为指定题目创建SRS数据
#[tauri::command]
pub async fn create_srs_data(
    state: State<'_, AppState>,
    input: CreateSRSDataInput,
) -> Result<srs_data::Model, String> {
    let db = state.db.as_ref();
    let now = chrono::Utc::now().timestamp();
    let id = Uuid::new_v4().to_string();

    let new_srs_data = srs_data::ActiveModel {
        id: Set(id.clone()),
        question_id: Set(input.question_id),
        difficulty: Set(input.difficulty),
        mastery: Set(input.mastery),
        lastreviewed_at: Set(now),
        review_count: Set(0),
        created_at: Set(now),
        updated_at: Set(now),
        version: Set(0),
        sync_status: Set("pending".to_string()),
        sync_hash: Set(None),
    };

    let _ = new_srs_data.clone().insert(db).await;

    // 将ActiveModel转换为Model
    let srs_model: srs_data::Model = SrsData::find_by_id(id)
        .one(db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or("srs添加不成功".to_string())?;

    Ok(srs_model)
}
