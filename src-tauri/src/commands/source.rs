// 来源相关命令

use crate::AppState;
use sea_orm::{ColumnTrait, EntityTrait, FromQueryResult, QueryFilter, QuerySelect};
use tauri::State;

use crate::database::entities::{prelude::Source, source};

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct CreateSourceInput {
    pub subject_id: Option<String>,
    pub book: Option<String>,
    pub chapter: Option<String>,
    pub knowledge: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct UpdateSourceInput {
    pub id: String,
    pub subject_id: Option<String>,
    pub book: Option<String>,
    pub chapter: Option<String>,
    pub knowledge: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct SourceFilter {
    pub subject_id: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize, FromQueryResult)]
pub struct BookInfo {
    pub book: String,
}

#[derive(serde::Serialize, serde::Deserialize, FromQueryResult)]
pub struct ChapterInfo {
    pub chapter: String,
}

#[derive(serde::Serialize, serde::Deserialize, FromQueryResult)]
pub struct KnowledgeInfo {
    pub knowledge: String,
}

/// 获取所有来源
#[tauri::command]
pub async fn get_sources(
    state: State<'_, AppState>,
    filter: Option<SourceFilter>,
) -> Result<Vec<source::Model>, String> {
    let db = state.db.as_ref();
    let filter = filter.unwrap_or_default();

    let mut query = Source::find().filter(source::Column::DeletedAt.is_null());

    // 按科目筛选
    if let Some(subject_id) = filter.subject_id {
        query = query.filter(source::Column::SubjectId.eq(subject_id));
    }

    let sources = query.all(db).await.map_err(|e| e.to_string())?;

    Ok(sources)
}

#[tauri::command]
pub async fn get_source(state: State<'_, AppState>, id: String) -> Result<source::Model, String> {
    let db = state.db.as_ref();

    let source = Source::find_by_id(id)
        .one(db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or("Source not found")?;
    Ok(source)
}

/// 获取指定科目下的所有书名
#[tauri::command]
pub async fn get_books(
    state: State<'_, AppState>,
    subject_id: Option<String>,
) -> Result<Vec<String>, String> {
    let db = state.db.as_ref();

    let mut query = Source::find()
        .filter(source::Column::DeletedAt.is_null())
        .filter(source::Column::Book.is_not_null())
        .select_only()
        .column(source::Column::Book)
        .distinct();

    if let Some(subject_id) = subject_id {
        query = query.filter(source::Column::SubjectId.eq(subject_id));
    }

    let books = query
        .into_model::<BookInfo>()
        .all(db)
        .await
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|b| b.book)
        .collect();

    Ok(books)
}

/// 获取指定科目和书名下的所有章节
#[tauri::command]
pub async fn get_chapters(
    state: State<'_, AppState>,
    subject_id: Option<String>,
    book: String,
) -> Result<Vec<String>, String> {
    let db = state.db.as_ref();

    let mut query = Source::find()
        .filter(source::Column::DeletedAt.is_null())
        .filter(source::Column::Book.eq(&book))
        .filter(source::Column::Chapter.is_not_null())
        .select_only()
        .column(source::Column::Chapter)
        .distinct();

    if let Some(subject_id) = subject_id {
        query = query.filter(source::Column::SubjectId.eq(subject_id));
    }

    let chapters = query
        .into_model::<ChapterInfo>()
        .all(db)
        .await
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|c| c.chapter)
        .collect();

    Ok(chapters)
}

/// 获取指定科目、书名和章节下的所有知识点
#[tauri::command]
pub async fn get_knowledges(
    state: State<'_, AppState>,
    subject_id: Option<String>,
    book: String,
    chapter: String,
) -> Result<Vec<String>, String> {
    let db = state.db.as_ref();

    let mut query = Source::find()
        .filter(source::Column::DeletedAt.is_null())
        .filter(source::Column::Book.eq(&book))
        .filter(source::Column::Chapter.eq(&chapter))
        .filter(source::Column::Knowledge.is_not_null())
        .select_only()
        .column(source::Column::Knowledge)
        .distinct();

    if let Some(subject_id) = subject_id {
        query = query.filter(source::Column::SubjectId.eq(subject_id));
    }

    let knowledges = query
        .into_model::<KnowledgeInfo>()
        .all(db)
        .await
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|k| k.knowledge)
        .collect();

    Ok(knowledges)
}

/// 创建来源
#[tauri::command]
pub async fn create_source(
    state: State<'_, AppState>,
    input: CreateSourceInput,
) -> Result<source::Model, String> {
    let db = state.db.as_ref();
    let now = chrono::Utc::now().timestamp();
    use sea_orm::ActiveModelTrait;
    use sea_orm::Set;
    use uuid::Uuid;
    let id = Uuid::new_v4().to_string();

    let new_source = source::ActiveModel {
        id: Set(id.clone()),
        question_id: Set(None), // 暂时为空，后续关联错题时再更新
        subject_id: Set(input.subject_id),
        book: Set(input.book),
        chapter: Set(input.chapter),
        knowledge: Set(input.knowledge),
        created_at: Set(now),
        updated_at: Set(now),
        deleted_at: Set(None),
        version: Set(0),
        sync_status: Set("pending".to_string()),
        sync_hash: Set(None),
    };

    // 插入数据
    let _ = new_source.insert(db).await;

    // 再次查询确认插入成功
    let inserted_source = Source::find_by_id(id)
        .one(db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or("插入后未能找到新创建的记录".to_string())?;

    Ok(inserted_source)
}

/// 更新来源
#[tauri::command]
pub async fn update_source(
    state: State<'_, AppState>,
    input: UpdateSourceInput,
) -> Result<source::Model, String> {
    let db = state.db.as_ref();
    let now = chrono::Utc::now().timestamp();
    use sea_orm::ActiveModelTrait;
    use sea_orm::Set;

    let source = Source::find_by_id(input.id.clone())
        .one(db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or("Source not found")?;

    let mut source: source::ActiveModel = source.into();

    if let Some(subject_id) = input.subject_id {
        source.subject_id = Set(Some(subject_id));
    }
    if let Some(book) = input.book {
        source.book = Set(Some(book));
    }
    if let Some(chapter) = input.chapter {
        source.chapter = Set(Some(chapter));
    }
    if let Some(knowledge) = input.knowledge {
        source.knowledge = Set(Some(knowledge));
    }

    source.updated_at = Set(now);
    source.version = Set(source.version.unwrap() + 1);
    source.sync_status = Set("pending".to_string());

    let source = source.update(db).await.map_err(|e| e.to_string())?;

    Ok(source)
}

/// 删除来源(软删除)
#[tauri::command]
pub async fn delete_source(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let db = state.db.as_ref();
    let now = chrono::Utc::now().timestamp();
    use sea_orm::ActiveModelTrait;
    use sea_orm::Set;

    let source = Source::find_by_id(id.clone())
        .one(db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or("Source not found")?;

    let mut source: source::ActiveModel = source.into();

    source.deleted_at = Set(Some(now));
    source.updated_at = Set(now);
    source.version = Set(source.version.unwrap() + 1);
    source.sync_status = Set("pending".to_string());

    source.update(db).await.map_err(|e| e.to_string())?;

    Ok(())
}

/// 获取或新增来源,并返回id
#[tauri::command]
pub async fn get_or_create_source_id(
    state: State<'_, AppState>,
    input: CreateSourceInput,
) -> Result<String, String> {
    let db = state.db.as_ref();
    let input_copy = input.clone();

    // 构建查询条件，对 None 值特殊处理
    let mut query = Source::find().filter(source::Column::DeletedAt.is_null()); // 只查询未删除的记录

    if let Some(ref subject_id) = input.subject_id {
        query = query.filter(source::Column::SubjectId.eq(subject_id));
    } else {
        // 如果输入值为 None，过滤数据库中也为 NULL 的记录
        query = query.filter(source::Column::SubjectId.is_null());
    }

    if let Some(ref book) = input.book {
        query = query.filter(source::Column::Book.eq(book));
    } else {
        query = query.filter(source::Column::Book.is_null());
    }

    if let Some(ref chapter) = input.chapter {
        query = query.filter(source::Column::Chapter.eq(chapter));
    } else {
        query = query.filter(source::Column::Chapter.is_null());
    }

    if let Some(ref knowledge) = input.knowledge {
        query = query.filter(source::Column::Knowledge.eq(knowledge));
    } else {
        query = query.filter(source::Column::Knowledge.is_null());
    }

    let source = query.one(db).await.map_err(|e| e.to_string())?;

    match source {
        Some(source) => Ok(source.id),
        None => {
            let source = create_source(state, input_copy).await?;
            Ok(source.id)
        }
    }
}
