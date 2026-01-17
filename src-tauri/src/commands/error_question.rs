// 错题相关命令

use crate::AppState;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect, Set, PaginatorTrait};
use tauri::State;
use uuid::Uuid;

use crate::database::entities::{prelude::{Subject, ErrorQuestion}, error_question};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CreateQuestionInput {
    pub userid: String,
    pub subjectid: String,
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
    pub subjectid: Option<String>,
    pub prompt: Option<String>,
    #[serde(rename = "type")]
    pub type_: Option<String>,
    pub answer: Option<String>,
    pub analysis: Option<String>,
    pub error_note: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct QuestionFilter {
    pub subjectid: Option<String>,
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

    let mut query = ErrorQuestion::find()
        .filter(error_question::Column::DeletedAt.is_null());

    // 按科目筛选
    if let Some(subject_id) = filter.subjectid {
        query = query.filter(error_question::Column::Subjectid.eq(subject_id));
    }

    // 搜索
    if let Some(search) = filter.search {
        let pattern = format!("%{}%", search);
        query = query.filter(
            error_question::Column::Prompt.like(&pattern)
                .or(error_question::Column::Analysis.like(&pattern))
                .or(error_question::Column::ErrorNote.like(&pattern))
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

    // 验证科目是否存在
    let _ = Subject::find_by_id(input.subjectid.clone())
        .one(db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or("Subject not found")?;

    let new_question = error_question::ActiveModel {
        id: Set(Uuid::new_v4().to_string()),
        userid: Set(input.userid),
        subjectid: Set(input.subjectid),
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

    let question = new_question.insert(db).await.map_err(|e| e.to_string())?;

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

    if let Some(subjectid) = input.subjectid {
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

    let question = question.update(db).await.map_err(|e| e.to_string())?;

    Ok(question)
}

/// 删除错题（软删除）
#[tauri::command]
pub async fn delete_question(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
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
pub async fn get_question_stats(
    state: State<'_, AppState>,
) -> Result<QuestionStats, String> {
    let db = state.db.as_ref();

    let total = ErrorQuestion::find()
        .filter(error_question::Column::DeletedAt.is_null())
        .count(db)
        .await
        .map_err(|e| e.to_string())?;

    let stats = QuestionStats {
        total,
    };

    Ok(stats)
}

#[derive(serde::Serialize)]
pub struct QuestionStats {
    pub total: u64,
}