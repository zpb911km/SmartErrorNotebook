// 同步相关命令 - 聚合查询接口

use crate::AppState;
use sea_orm::{ActiveValue, ColumnTrait, EntityTrait, QueryFilter};
use serde::Serialize;
use tauri::State;

/// 级联检查孤儿记录结果
#[derive(Debug, Serialize)]
pub struct CascadeOrphanCheckResult {
    /// 被标记为软删除的记录 (格式："table_name:id")
    pub orphan_records_soft_deleted: Vec<String>,
    pub total_checked: usize,
}

/// 轻量记录头（握手用，不含 data 负载）
#[derive(Debug, Clone, Serialize)]
pub struct SyncRecordHeader {
    pub id: String,
    pub table_name: String,
    pub version: i32,
    pub status: String,
    pub deleted_at: Option<i64>,
    pub updated_at: i64,
    pub created_at: i64,
}

/// 获取所有记录（无视 status），仅返回握手所需的字段，不含 data
#[tauri::command]
pub async fn get_all_records(state: State<'_, AppState>) -> Result<Vec<SyncRecordHeader>, String> {
    let db = state.db.as_ref();
    let mut results = Vec::new();

    // error_questions
    {
        use crate::database::entities::error_question as eq;
        let models = eq::Entity::find()
            .all(db)
            .await
            .map_err(|e| format!("Failed to query error_questions: {}", e))?;
        for model in models {
            results.push(model_to_header(model, "error_questions".to_string()));
        }
    }

    // subjects
    {
        use crate::database::entities::subject as sub;
        let models = sub::Entity::find()
            .all(db)
            .await
            .map_err(|e| format!("Failed to query subjects: {}", e))?;
        for model in models {
            results.push(model_to_header(model, "subjects".to_string()));
        }
    }

    // srs_data
    {
        use crate::database::entities::srs_data as srs;
        let models = srs::Entity::find()
            .all(db)
            .await
            .map_err(|e| format!("Failed to query srs_data: {}", e))?;
        for model in models {
            results.push(model_to_header(model, "srs_data".to_string()));
        }
    }

    // attachments
    {
        use crate::database::entities::attachment as att;
        let models = att::Entity::find()
            .all(db)
            .await
            .map_err(|e| format!("Failed to query attachments: {}", e))?;
        for model in models {
            results.push(model_to_header(model, "attachments".to_string()));
        }
    }

    // error_tags
    {
        use crate::database::entities::error_tag as tag;
        let models = tag::Entity::find()
            .all(db)
            .await
            .map_err(|e| format!("Failed to query error_tags: {}", e))?;
        for model in models {
            results.push(model_to_header(model, "error_tags".to_string()));
        }
    }

    // sources
    {
        use crate::database::entities::source as src;
        let models = src::Entity::find()
            .all(db)
            .await
            .map_err(|e| format!("Failed to query sources: {}", e))?;
        for model in models {
            results.push(model_to_header(model, "sources".to_string()));
        }
    }

    Ok(results)
}

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

#[tauri::command]
pub async fn set_record_sync_status_version(
    state: State<'_, AppState>,
    record_id: String,
    status: String,
    version: i32,
) -> Result<String, String> {
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
                use sea_orm::ActiveModelTrait;
                if let Some(model) = eq::Entity::find_by_id(record_id.clone())
                    .one(db)
                    .await
                    .map_err(|e| format!("Failed to query error_questions: {}", e))?
                {
                    let mut model: eq::ActiveModel = model.into();
                    model.sync_status = ActiveValue::Set(status.clone());
                    model.version = ActiveValue::Set(version);
                    let rst = model
                        .update(db)
                        .await
                        .map_err(|e| format!("Failed to update error_questions: {}", e))?;
                    return Ok(format!("Record updated with id: {:?}", rst.id));
                }
            }
            "subjects" => {
                use crate::database::entities::subject as sub;
                use sea_orm::ActiveModelTrait;
                if let Some(model) = sub::Entity::find_by_id(record_id.clone())
                    .one(db)
                    .await
                    .map_err(|e| format!("Failed to query subjects: {}", e))?
                {
                    let mut model: sub::ActiveModel = model.into();
                    model.sync_status = ActiveValue::Set(status.clone());
                    model.version = ActiveValue::Set(version);
                    let rst = model
                        .update(db)
                        .await
                        .map_err(|e| format!("Failed to update subjects: {}", e))?;
                    return Ok(format!("Record updated with id: {:?}", rst.id));
                }
            }
            "srs_data" => {
                use crate::database::entities::srs_data as srs;
                use sea_orm::{ActiveModelTrait, ActiveValue};
                if let Some(model) = srs::Entity::find_by_id(record_id.clone())
                    .one(db)
                    .await
                    .map_err(|e| format!("Failed to query srs_data: {}", e))?
                {
                    let mut model: srs::ActiveModel = model.into();
                    model.sync_status = ActiveValue::Set(status.clone());
                    model.version = ActiveValue::Set(version);
                    model.deleted_at = ActiveValue::Set(None);
                    let rst = model
                        .update(db)
                        .await
                        .map_err(|e| format!("Failed to update srs_data: {}", e))?;
                    return Ok(format!("Record updated with id: {:?}", rst.id));
                }
            }
            "attachments" => {
                use crate::database::entities::attachment as att;
                use sea_orm::ActiveModelTrait;
                if let Some(model) = att::Entity::find_by_id(record_id.clone())
                    .one(db)
                    .await
                    .map_err(|e| format!("Failed to query attachments: {}", e))?
                {
                    let mut model: att::ActiveModel = model.into();
                    model.sync_status = ActiveValue::Set(status.clone());
                    model.version = ActiveValue::Set(version);
                    let rst = model
                        .update(db)
                        .await
                        .map_err(|e| format!("Failed to update attachments: {}", e))?;
                    return Ok(format!("Record updated with id: {:?}", rst.id));
                }
            }
            "error_tags" => {
                use crate::database::entities::error_tag as tag;
                use sea_orm::ActiveModelTrait;
                if let Some(model) = tag::Entity::find_by_id(record_id.clone())
                    .one(db)
                    .await
                    .map_err(|e| format!("Failed to query error_tags: {}", e))?
                {
                    let mut model: tag::ActiveModel = model.into();
                    model.sync_status = ActiveValue::Set(status.clone());
                    model.version = ActiveValue::Set(version);
                    let rst = model
                        .update(db)
                        .await
                        .map_err(|e| format!("Failed to update error_tags: {}", e))?;
                    return Ok(format!("Record updated with id: {:?}", rst.id));
                }
            }
            "sources" => {
                use crate::database::entities::source as src;
                use sea_orm::ActiveModelTrait;
                if let Some(model) = src::Entity::find_by_id(record_id.clone())
                    .one(db)
                    .await
                    .map_err(|e| format!("Failed to query sources: {}", e))?
                {
                    let mut model: src::ActiveModel = model.into();
                    model.sync_status = ActiveValue::Set(status.clone());
                    model.version = ActiveValue::Set(version);
                    let rst = model
                        .update(db)
                        .await
                        .map_err(|e| format!("Failed to update sources: {}", e))?;
                    return Ok(format!("Record updated with id: {:?}", rst.id));
                }
            }
            _ => {}
        }
    }

    Err(format!("Record not found with id: {}", record_id))
}

/// 清理所有已同步(synced)且已软删除(deleted_at 不空)的记录，执行真删除
#[tauri::command]
pub async fn purge_synced_deletions(
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let db = state.db.as_ref();
    let mut results = serde_json::Map::new();

    // error_questions
    {
        use crate::database::entities::error_question as eq;
        let deleted = eq::Entity::delete_many()
            .filter(eq::Column::SyncStatus.eq("synced"))
            .filter(eq::Column::DeletedAt.is_not_null())
            .exec(db)
            .await
            .map_err(|e| format!("Failed to purge error_questions: {}", e))?;
        results.insert(
            "error_questions".to_string(),
            serde_json::json!({ "deleted": deleted.rows_affected }),
        );
    }

    // subjects
    {
        use crate::database::entities::subject as sub;
        let deleted = sub::Entity::delete_many()
            .filter(sub::Column::SyncStatus.eq("synced"))
            .filter(sub::Column::DeletedAt.is_not_null())
            .exec(db)
            .await
            .map_err(|e| format!("Failed to purge subjects: {}", e))?;
        results.insert(
            "subjects".to_string(),
            serde_json::json!({ "deleted": deleted.rows_affected }),
        );
    }

    // attachments
    {
        use crate::database::entities::attachment as att;
        let deleted = att::Entity::delete_many()
            .filter(att::Column::SyncStatus.eq("synced"))
            .filter(att::Column::DeletedAt.is_not_null())
            .exec(db)
            .await
            .map_err(|e| format!("Failed to purge attachments: {}", e))?;
        results.insert(
            "attachments".to_string(),
            serde_json::json!({ "deleted": deleted.rows_affected }),
        );
    }

    // error_tags
    {
        use crate::database::entities::error_tag as tag;
        let deleted = tag::Entity::delete_many()
            .filter(tag::Column::SyncStatus.eq("synced"))
            .filter(tag::Column::DeletedAt.is_not_null())
            .exec(db)
            .await
            .map_err(|e| format!("Failed to purge error_tags: {}", e))?;
        results.insert(
            "error_tags".to_string(),
            serde_json::json!({ "deleted": deleted.rows_affected }),
        );
    }

    // sources
    {
        use crate::database::entities::source as src;
        let deleted = src::Entity::delete_many()
            .filter(src::Column::SyncStatus.eq("synced"))
            .filter(src::Column::DeletedAt.is_not_null())
            .exec(db)
            .await
            .map_err(|e| format!("Failed to purge sources: {}", e))?;
        results.insert(
            "sources".to_string(),
            serde_json::json!({ "deleted": deleted.rows_affected }),
        );
    }
    let rst = check_orphan_records(state).await?;
    results.insert(
        "orphan_records".to_string(),
        serde_json::json!({ "checked": rst.total_checked }),
    );
    Ok(serde_json::Value::Object(results))
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
    let status = obj
        .get("sync_status")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let deleted_at = obj.get("deleted_at").and_then(|v| v.as_i64());
    let updated_at = obj.get("updated_at").and_then(|v| v.as_i64()).unwrap_or(0);

    SyncRecordOutput {
        id: obj
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        table_name,
        version,
        status,
        deleted_at,
        updated_at,
        data: serde_json::Value::Object(data),
    }
}

/// 将 SeaORM Model 转换为轻量记录头（不含 data）
fn model_to_header<M>(model: M, table_name: String) -> SyncRecordHeader
where
    M: sea_orm::ModelTrait + Serialize,
{
    let model_json: serde_json::Value = serde_json::to_value(&model).unwrap_or_default();
    let obj = model_json.as_object().unwrap().clone();

    SyncRecordHeader {
        id: obj
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        table_name,
        version: obj.get("version").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
        status: obj
            .get("sync_status")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        deleted_at: obj.get("deleted_at").and_then(|v| v.as_i64()),
        updated_at: obj.get("updated_at").and_then(|v| v.as_i64()).unwrap_or(0),
        created_at: obj.get("created_at").and_then(|v| v.as_i64()).unwrap_or(0),
    }
}

pub async fn check_orphan_records(
    state: State<'_, AppState>,
) -> Result<CascadeOrphanCheckResult, String> {
    use sea_orm::{ActiveModelTrait, ActiveValue};

    let db = state.db.as_ref();
    let now = chrono::Utc::now().timestamp();
    let mut orphan_records_soft_deleted = Vec::new();
    let mut total_checked = 0;

    // ========== 第 1 层：检查依赖 subjects 的记录 ==========

    // 1.1 检查 error_questions 依赖的 subject 是否存在且未软删除
    {
        use crate::database::entities::{error_question as eq, subject as sub};
        let all_questions = eq::Entity::find()
            .filter(eq::Column::DeletedAt.is_null())
            .all(db)
            .await
            .map_err(|e| format!("Failed to query error_questions: {}", e))?;
        total_checked += all_questions.len();

        for q in all_questions {
            let subject_exists = sub::Entity::find_by_id(q.subjectid.clone())
                .filter(sub::Column::DeletedAt.is_null())
                .one(db)
                .await
                .map_err(|e| format!("Failed to query subject: {}", e))?
                .is_some();

            if !subject_exists {
                let mut active_model: eq::ActiveModel = q.clone().into();
                active_model.deleted_at = ActiveValue::Set(Some(now));
                active_model.updated_at = ActiveValue::Set(now);
                active_model.version = ActiveValue::Set(q.version);
                active_model.sync_status = ActiveValue::Set("pending".to_string());
                active_model
                    .update(db)
                    .await
                    .map_err(|e| format!("Failed to soft delete error_question: {}", e))?;
                orphan_records_soft_deleted.push(format!("error_question:{}", q.id));
            }
        }
    }

    // 1.2 检查 sources 依赖的 subject 是否存在且未软删除
    {
        use crate::database::entities::{source as src, subject as sub};
        let all_sources = src::Entity::find()
            .filter(src::Column::DeletedAt.is_null())
            .all(db)
            .await
            .map_err(|e| format!("Failed to query sources: {}", e))?;
        total_checked += all_sources.len();

        for s in all_sources {
            // source 可能没有 subject_id（可选字段）
            if let Some(ref subjectid) = s.subject_id {
                let subject_exists = sub::Entity::find_by_id(subjectid.clone())
                    .filter(sub::Column::DeletedAt.is_null())
                    .one(db)
                    .await
                    .map_err(|e| format!("Failed to query subject: {}", e))?
                    .is_some();

                if !subject_exists {
                    let mut active_model: src::ActiveModel = s.clone().into();
                    active_model.deleted_at = ActiveValue::Set(Some(now));
                    active_model.updated_at = ActiveValue::Set(now);
                    active_model.version = ActiveValue::Set(s.version);
                    active_model.sync_status = ActiveValue::Set("pending".to_string());
                    active_model
                        .update(db)
                        .await
                        .map_err(|e| format!("Failed to soft delete source: {}", e))?;
                    orphan_records_soft_deleted.push(format!("source:{}", s.id));
                }
            }
        }
    }

    // ========== 第 2 层：检查依赖 error_questions 的记录 ==========

    // 2.1 检查 srs_data 依赖的 error_question 是否存在且未软删除
    {
        use crate::database::entities::{error_question as eq, srs_data as srs};
        let all_srs = srs::Entity::find()
            .filter(srs::Column::DeletedAt.is_null())
            .all(db)
            .await
            .map_err(|e| format!("Failed to query srs_data: {}", e))?;
        total_checked += all_srs.len();

        for srs_model in all_srs {
            let parent_exists = eq::Entity::find_by_id(srs_model.question_id.clone())
                .filter(eq::Column::DeletedAt.is_null())
                .one(db)
                .await
                .map_err(|e| format!("Failed to query error_question: {}", e))?
                .is_some();

            if !parent_exists {
                let mut active_model: srs::ActiveModel = srs_model.clone().into();
                active_model.deleted_at = ActiveValue::Set(Some(now));
                active_model.updated_at = ActiveValue::Set(now);
                active_model.version = ActiveValue::Set(srs_model.version);
                active_model.sync_status = ActiveValue::Set("pending".to_string());
                active_model
                    .update(db)
                    .await
                    .map_err(|e| format!("Failed to soft delete srs_data: {}", e))?;
                orphan_records_soft_deleted.push(format!("srs_data:{}", srs_model.id));
            }
        }
    }

    // 2.2 检查 error_tag 依赖的 error_question 是否存在且未软删除
    {
        use crate::database::entities::{error_question as eq, error_tag as tag};
        let all_tags = tag::Entity::find()
            .filter(tag::Column::DeletedAt.is_null())
            .all(db)
            .await
            .map_err(|e| format!("Failed to query error_tags: {}", e))?;
        total_checked += all_tags.len();

        for tag_model in all_tags {
            let parent_exists = eq::Entity::find_by_id(tag_model.question_id.clone())
                .filter(eq::Column::DeletedAt.is_null())
                .one(db)
                .await
                .map_err(|e| format!("Failed to query error_question: {}", e))?
                .is_some();

            if !parent_exists {
                let mut active_model: tag::ActiveModel = tag_model.clone().into();
                active_model.deleted_at = ActiveValue::Set(Some(now));
                active_model.updated_at = ActiveValue::Set(now);
                active_model.version = ActiveValue::Set(tag_model.version);
                active_model.sync_status = ActiveValue::Set("pending".to_string());
                active_model
                    .update(db)
                    .await
                    .map_err(|e| format!("Failed to soft delete error_tag: {}", e))?;
                orphan_records_soft_deleted.push(format!("error_tag:{}", tag_model.id));
            }
        }
    }

    // 2.3 检查 attachment 依赖的 error_question 是否存在且未软删除
    {
        use crate::database::entities::{attachment as att, error_question as eq};
        let all_attachments = att::Entity::find()
            .filter(att::Column::DeletedAt.is_null())
            .all(db)
            .await
            .map_err(|e| format!("Failed to query attachments: {}", e))?;
        total_checked += all_attachments.len();

        for att_model in all_attachments {
            let parent_exists = eq::Entity::find_by_id(att_model.question_id.clone())
                .filter(eq::Column::DeletedAt.is_null())
                .one(db)
                .await
                .map_err(|e| format!("Failed to query error_question: {}", e))?
                .is_some();

            if !parent_exists {
                let mut active_model: att::ActiveModel = att_model.clone().into();
                active_model.deleted_at = ActiveValue::Set(Some(now));
                active_model.updated_at = ActiveValue::Set(now);
                active_model.version = ActiveValue::Set(att_model.version);
                active_model.sync_status = ActiveValue::Set("pending".to_string());
                active_model
                    .update(db)
                    .await
                    .map_err(|e| format!("Failed to soft delete attachment: {}", e))?;
                orphan_records_soft_deleted.push(format!("attachment:{}", att_model.id));
            }
        }
    }

    Ok(CascadeOrphanCheckResult {
        orphan_records_soft_deleted,
        total_checked,
    })
}
