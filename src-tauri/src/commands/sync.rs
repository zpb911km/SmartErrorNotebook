// 同步相关命令 - 聚合查询接口

use crate::AppState;
use sea_orm::{EntityTrait, ColumnTrait, QueryFilter};
use serde::{Serialize, Deserialize};
use tauri::State;

/// 通用记录输出格式（返回给前端）
#[derive(Debug, Clone, Serialize)]
pub struct SyncRecordOutput {
    pub id: String,
    pub table_name: String,
    pub version: i32,
    pub status: String,
    pub deleted_at: Option<i64>,
    pub updated_at: i64,
    #[serde(rename = "data")]
    pub data: serde_json::Value,
}

/// 获取所有 pending 状态的记录（按表分别查询并合并返回）
#[tauri::command]
pub async fn get_all_pending_records(
    state: State<'_, AppState>,
) -> Result<Vec<SyncRecordOutput>, String> {
    let db = state.db.as_ref();
    let mut results = Vec::new();

    // error_questions
    {
        use crate::database::entities::error_question as eq;
        let models = eq::Entity::find()
            .filter(eq::Column::SyncStatus.eq("pending"))
            .all(db)
            .await
            .map_err(|e| format!("Failed to query error_questions: {}", e))?;
        for model in models {
            results.push(model_to_sync_output(model, "error_questions".to_string()));
        }
    }

    // subjects
    {
        use crate::database::entities::subject as sub;
        let models = sub::Entity::find()
            .filter(sub::Column::SyncStatus.eq("pending"))
            .all(db)
            .await
            .map_err(|e| format!("Failed to query subjects: {}", e))?;
        for model in models {
            results.push(model_to_sync_output(model, "subjects".to_string()));
        }
    }

    // srs_data
    {
        use crate::database::entities::srs_data as srs;
        let models = srs::Entity::find()
            .filter(srs::Column::SyncStatus.eq("pending"))
            .all(db)
            .await
            .map_err(|e| format!("Failed to query srs_data: {}", e))?;
        for model in models {
            results.push(model_to_sync_output(model, "srs_data".to_string()));
        }
    }

    // attachments
    {
        use crate::database::entities::attachment as att;
        let models = att::Entity::find()
            .filter(att::Column::SyncStatus.eq("pending"))
            .all(db)
            .await
            .map_err(|e| format!("Failed to query attachments: {}", e))?;
        for model in models {
            results.push(model_to_sync_output(model, "attachments".to_string()));
        }
    }

    // error_tags
    {
        use crate::database::entities::error_tag as tag;
        let models = tag::Entity::find()
            .filter(tag::Column::SyncStatus.eq("pending"))
            .all(db)
            .await
            .map_err(|e| format!("Failed to query error_tags: {}", e))?;
        for model in models {
            results.push(model_to_sync_output(model, "error_tags".to_string()));
        }
    }

    // sources
    {
        use crate::database::entities::source as src;
        let models = src::Entity::find()
            .filter(src::Column::SyncStatus.eq("pending"))
            .all(db)
            .await
            .map_err(|e| format!("Failed to query sources: {}", e))?;
        for model in models {
            results.push(model_to_sync_output(model, "sources".to_string()));
        }
    }

    Ok(results)
}

/// 根据 ID 反查表名，获取单个完整记录用于上传
#[tauri::command]
pub async fn get_record_for_upload(
    state: State<'_, AppState>,
    record_id: String,
) -> Result<SyncRecordOutput, String> {
    let db = state.db.as_ref();

    // 依次在各个表中查找
    let table_names = [
        "error_questions",
        "subjects",
        "srs_data",
        "attachments",
        "error_tags",
        "sources",
    ];

    for &table_name in &table_names {
        match table_name {
            "error_questions" => {
                use crate::database::entities::error_question as eq;
                if let Some(model) = eq::Entity::find_by_id(record_id.clone())
                    .one(db)
                    .await
                    .map_err(|e| format!("Failed to query error_questions: {}", e))?
                {
                    return Ok(model_to_sync_output(model, "error_questions".to_string()));
                }
            }
            "subjects" => {
                use crate::database::entities::subject as sub;
                if let Some(model) = sub::Entity::find_by_id(record_id.clone())
                    .one(db)
                    .await
                    .map_err(|e| format!("Failed to query subjects: {}", e))?
                {
                    return Ok(model_to_sync_output(model, "subjects".to_string()));
                }
            }
            "srs_data" => {
                use crate::database::entities::srs_data as srs;
                if let Some(model) = srs::Entity::find_by_id(record_id.clone())
                    .one(db)
                    .await
                    .map_err(|e| format!("Failed to query srs_data: {}", e))?
                {
                    return Ok(model_to_sync_output(model, "srs_data".to_string()));
                }
            }
            "attachments" => {
                use crate::database::entities::attachment as att;
                if let Some(model) = att::Entity::find_by_id(record_id.clone())
                    .one(db)
                    .await
                    .map_err(|e| format!("Failed to query attachments: {}", e))?
                {
                    return Ok(model_to_sync_output(model, "attachments".to_string()));
                }
            }
            "error_tags" => {
                use crate::database::entities::error_tag as tag;
                if let Some(model) = tag::Entity::find_by_id(record_id.clone())
                    .one(db)
                    .await
                    .map_err(|e| format!("Failed to query error_tags: {}", e))?
                {
                    return Ok(model_to_sync_output(model, "error_tags".to_string()));
                }
            }
            "sources" => {
                use crate::database::entities::source as src;
                if let Some(model) = src::Entity::find_by_id(record_id.clone())
                    .one(db)
                    .await
                    .map_err(|e| format!("Failed to query sources: {}", e))?
                {
                    return Ok(model_to_sync_output(model, "sources".to_string()));
                }
            }
            _ => {}
        }
    }

    Err(format!("Record not found with id: {}", record_id))
}

/// 将 SeaORM Model 转换为通用输出格式（针对具体类型实现）
fn model_to_sync_output<M>(model: M, table_name: String) -> SyncRecordOutput
where
    M: sea_orm::ModelTrait + Serialize,
{
    let model_json: serde_json::Value = serde_json::to_value(&model).unwrap_or_default();
    let obj = model_json.as_object().unwrap().clone();

    // 移除同步字段，只保留业务数据
    let mut data = obj.clone();
    data.remove("version");
    data.remove("sync_status");
    data.remove("deleted_at");
    data.remove("updated_at");
    data.remove("created_at");

    // 从原始 model 提取需要同步的字段值
    let version = obj.get("version").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let status = obj.get("sync_status").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let deleted_at = obj.get("deleted_at").and_then(|v| v.as_i64());
    let updated_at = obj.get("updated_at").and_then(|v| v.as_i64()).unwrap_or(0);

    SyncRecordOutput {
        id: obj.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        table_name,
        version,
        status,
        deleted_at,
        updated_at,
        data: serde_json::Value::Object(data),
    }
}
