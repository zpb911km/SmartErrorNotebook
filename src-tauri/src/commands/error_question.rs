// 错题相关命令

use crate::AppState;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect, Set,
};
use tauri::State;
use uuid::Uuid;

use crate::database::entities::{
    error_question,
    prelude::{ErrorQuestion, Subject},
};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CreateQuestionInput {
    pub user_id: String,
    pub subject_id: String,
    pub source_id: Option<String>,
    pub prompt: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub answer: Option<String>,
    pub analysis: Option<String>,
    pub error_note: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct UpdateQuestionInput {
    pub id: String,
    pub subject_id: Option<String>,
    pub prompt: Option<String>,
    #[serde(rename = "type")]
    pub type_: Option<String>,
    pub answer: Option<String>,
    pub analysis: Option<String>,
    pub error_note: Option<String>,
}

// 用于同步的 UPSERT 输入
#[derive(serde::Serialize, serde::Deserialize)]
pub struct UpsertQuestionInput {
    pub id: String,
    pub version: i32,
    pub status: String,
    pub deleted_at: Option<i64>,
    pub userid: String,
    #[serde(alias = "subject_id")]
    pub subjectid: String,
    #[serde(alias = "source_id")]
    pub sourceid: Option<String>,
    pub prompt: String,
    #[serde(rename = "type_")]
    pub type_: String,
    pub answer: Option<String>,
    pub analysis: Option<String>,
    pub error_note: Option<String>,
    pub sync_hash: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct QuestionFilter {
    pub subject_id: Option<String>,
    pub search: Option<String>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}

/// 获取所有错题
#[tauri::command]
pub async fn get_questions(
    state: State<'_, AppState>,
    filter: Option<QuestionFilter>,
) -> Result<Vec<error_question::Model>, String> {
    let db = state.db.as_ref();
    let filter = filter.unwrap_or_default();

    let mut query = ErrorQuestion::find().filter(error_question::Column::DeletedAt.is_null());

    // 按科目筛选
    if let Some(subject_id) = filter.subject_id {
        query = query.filter(error_question::Column::Subjectid.eq(subject_id));
    }

    // 搜索
    if let Some(search) = filter.search {
        let pattern = format!("%{}%", search);
        query = query.filter(
            error_question::Column::Prompt
                .like(&pattern)
                .or(error_question::Column::Analysis.like(&pattern))
                .or(error_question::Column::ErrorNote.like(&pattern)),
        );
    }

    // 排序
    query = query.order_by_desc(error_question::Column::UpdatedAt);

    // 分页
    if let Some(limit) = filter.limit {
        query = query.limit(limit);
    }
    if let Some(offset) = filter.offset {
        query = query.offset(offset);
    }

    let questions = query.all(db).await.map_err(|e| e.to_string())?;

    Ok(questions)
}

/// 获取单个错题
#[tauri::command]
pub async fn get_question(
    state: State<'_, AppState>,
    id: String,
) -> Result<error_question::Model, String> {
    let db = state.db.as_ref();

    let question = ErrorQuestion::find_by_id(id)
        .one(db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or("Question not found")?;

    Ok(question)
}

/// 创建错题
#[tauri::command]
pub async fn create_question(
    state: State<'_, AppState>,
    input: CreateQuestionInput,
) -> Result<error_question::Model, String> {
    let db = state.db.as_ref();
    let now = chrono::Utc::now().timestamp();
    let id = Uuid::new_v4().to_string();

    // 验证科目是否存在
    let _ = Subject::find_by_id(input.subject_id.clone())
        .one(db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or("Subject not found")?;

    let new_question = error_question::ActiveModel {
        id: Set(id.clone()),
        userid: Set(input.user_id),
        subjectid: Set(input.subject_id),
        sourceid: Set(input.source_id),
        prompt: Set(input.prompt),
        type_: Set(input.type_),
        answer: Set(input.answer),
        analysis: Set(input.analysis),
        error_note: Set(input.error_note),
        created_at: Set(now),
        updated_at: Set(now),
        deleted_at: Set(None),
        version: Set(0),
        sync_status: Set("pending".to_string()),
        sync_hash: Set(None),
    };

    let _ = new_question.insert(db).await;

    let question = ErrorQuestion::find_by_id(id)
        .one(db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or("插入后未能找到新创建的记录".to_string())?;

    Ok(question)
}

/// 更新错题
#[tauri::command]
pub async fn update_question(
    state: State<'_, AppState>,
    input: UpdateQuestionInput,
) -> Result<error_question::Model, String> {
    let db = state.db.as_ref();
    let now = chrono::Utc::now().timestamp();

    let question = ErrorQuestion::find_by_id(input.id.clone())
        .one(db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or("Question not found")?;

    let mut question: error_question::ActiveModel = question.into();

    if let Some(subjectid) = input.subject_id {
        // 验证科目是否存在
        let _ = Subject::find_by_id(subjectid.clone())
            .one(db)
            .await
            .map_err(|e| e.to_string())?
            .ok_or("Subject not found")?;
        question.subjectid = Set(subjectid);
    }
    if let Some(prompt) = input.prompt {
        question.prompt = Set(prompt);
    }
    if let Some(type_) = input.type_ {
        question.type_ = Set(type_);
    }
    if let Some(answer) = input.answer {
        question.answer = Set(Some(answer));
    }
    if let Some(analysis) = input.analysis {
        question.analysis = Set(Some(analysis));
    }
    if let Some(error_note) = input.error_note {
        question.error_note = Set(Some(error_note));
    }

    question.updated_at = Set(now);
    question.version = Set(question.version.unwrap() + 1);
    question.sync_status = Set("pending".to_string());

    let question = question.update(db).await.map_err(|e| {
        eprintln!("更新错题失败: {}", e);
        format!("更新错题失败: {}", e)
    })?;

    Ok(question)
}

/// 删除错题（软删除）
#[tauri::command]
pub async fn delete_question(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let db = state.db.as_ref();
    let now = chrono::Utc::now().timestamp();

    let question = ErrorQuestion::find_by_id(id.clone())
        .one(db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or("Question not found")?;

    let mut question: error_question::ActiveModel = question.into();

    question.deleted_at = Set(Some(now));
    question.updated_at = Set(now);
    question.version = Set(question.version.unwrap() + 1);
    question.sync_status = Set("pending".to_string());

    question.update(db).await.map_err(|e| e.to_string())?;

    Ok(())
}

/// 获取错题统计
#[tauri::command]
pub async fn get_question_stats(state: State<'_, AppState>) -> Result<QuestionStats, String> {
    let db = state.db.as_ref();

    let total = ErrorQuestion::find()
        .filter(error_question::Column::DeletedAt.is_null())
        .count(db)
        .await
        .map_err(|e| e.to_string())?;

    let stats = QuestionStats { total };

    Ok(stats)
}

#[derive(serde::Serialize)]
pub struct QuestionStats {
    pub total: u64,
}

/// UPSERT: 根据 ID 插入或更新错题（用于同步）
#[tauri::command]
pub async fn upsert_error_question(
    state: State<'_, AppState>,
    input: UpsertQuestionInput,
) -> Result<(), String> {
    use crate::database::entities::error_question as eq;

    let db = state.db.as_ref();
    let now = chrono::Utc::now().timestamp();

    // 检查记录是否存在
    let existing = eq::Entity::find_by_id(input.id.clone())
        .one(db)
        .await
        .map_err(|e| format!("Query failed: {}", e))?;

    if let Some(model) = existing {
        // 更新现有记录
        let mut active_model: error_question::ActiveModel = model.into();
        active_model.subjectid = Set(input.subjectid);
        active_model.sourceid = Set(input.sourceid);
        active_model.prompt = Set(input.prompt);
        active_model.type_ = Set(input.type_);
        active_model.answer = Set(input.answer);
        active_model.analysis = Set(input.analysis);
        active_model.error_note = Set(input.error_note);
        active_model.updated_at = Set(now);
        active_model.version = Set(input.version);
        active_model.sync_status = Set("synced".to_string());
        active_model.deleted_at = Set(input.deleted_at);

        active_model.update(db).await.map_err(|e| e.to_string())?;
    } else {
        // 插入新记录
        let new_question = error_question::ActiveModel {
            id: Set(input.id),
            userid: Set(input.userid),
            subjectid: Set(input.subjectid),
            sourceid: Set(input.sourceid),
            prompt: Set(input.prompt),
            type_: Set(input.type_),
            answer: Set(input.answer),
            analysis: Set(input.analysis),
            error_note: Set(input.error_note),
            created_at: Set(now),
            updated_at: Set(now),
            deleted_at: Set(input.deleted_at),
            version: Set(input.version),
            sync_status: Set("synced".to_string()),
            sync_hash: Set(input.sync_hash)
        };

        let _ = new_question.insert(db).await;
    }

    Ok(())
}
