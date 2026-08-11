use crate::database::entities::{
    error_question,
    prelude::{ErrorQuestion, Subject},
    srs_data,
};
use crate::domain;
use crate::repository::{
    accept_uuid_insert_result, to_domain, to_domains, RepositoryError, RepositoryResult,
};
use async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbConn, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect, Set,
};
use std::sync::Arc;

pub struct QuestionQuery {
    pub subject_id: Option<String>,
    pub search: Option<String>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}
pub struct NewQuestion {
    pub id: String,
    pub user_id: String,
    pub subject_id: String,
    pub source_id: Option<String>,
    pub prompt: String,
    pub type_: String,
    pub answer: Option<String>,
    pub analysis: Option<String>,
    pub error_note: Option<String>,
    pub now: i64,
}
pub struct QuestionChanges {
    pub id: String,
    pub subject_id: Option<String>,
    pub source_id: Option<String>,
    pub prompt: Option<String>,
    pub type_: Option<String>,
    pub answer: Option<String>,
    pub analysis: Option<String>,
    pub error_note: Option<String>,
    pub now: i64,
}
pub struct SyncedQuestion {
    pub id: String,
    pub version: i32,
    pub deleted_at: Option<i64>,
    pub user_id: String,
    pub subject_id: String,
    pub source_id: Option<String>,
    pub prompt: String,
    pub type_: String,
    pub answer: Option<String>,
    pub analysis: Option<String>,
    pub error_note: Option<String>,
    pub sync_hash: Option<String>,
    pub now: i64,
}

#[async_trait]
pub trait ErrorQuestionRepository: Send + Sync {
    async fn list_active(
        &self,
        query: QuestionQuery,
    ) -> RepositoryResult<Vec<domain::ErrorQuestion>>;
    async fn find_by_id(&self, id: String) -> RepositoryResult<domain::ErrorQuestion>;
    async fn create(&self, input: NewQuestion) -> RepositoryResult<domain::ErrorQuestion>;
    async fn update(&self, input: QuestionChanges) -> RepositoryResult<domain::ErrorQuestion>;
    async fn soft_delete_with_srs(&self, id: String, now: i64) -> RepositoryResult<()>;
    async fn count_active(&self) -> RepositoryResult<u64>;
    async fn upsert_synced(&self, input: SyncedQuestion) -> RepositoryResult<()>;
}
pub struct SeaOrmErrorQuestionRepository {
    db: Arc<DbConn>,
}
impl SeaOrmErrorQuestionRepository {
    pub fn new(db: Arc<DbConn>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl ErrorQuestionRepository for SeaOrmErrorQuestionRepository {
    async fn list_active(
        &self,
        input: QuestionQuery,
    ) -> RepositoryResult<Vec<domain::ErrorQuestion>> {
        let mut query = ErrorQuestion::find().filter(error_question::Column::DeletedAt.is_null());
        if let Some(id) = input.subject_id {
            query = query.filter(error_question::Column::Subjectid.eq(id));
        }
        if let Some(search) = input.search {
            let pattern = format!("%{search}%");
            query = query.filter(
                error_question::Column::Prompt
                    .like(&pattern)
                    .or(error_question::Column::Analysis.like(&pattern))
                    .or(error_question::Column::ErrorNote.like(&pattern)),
            );
        }
        query = query.order_by_desc(error_question::Column::UpdatedAt);
        if let Some(limit) = input.limit {
            query = query.limit(limit);
        }
        if let Some(offset) = input.offset {
            query = query.offset(offset);
        }
        let models = query
            .all(self.db.as_ref())
            .await
            .map_err(RepositoryError::from)?;
        to_domains(models)
    }
    async fn find_by_id(&self, id: String) -> RepositoryResult<domain::ErrorQuestion> {
        let model = ErrorQuestion::find_by_id(id)
            .one(self.db.as_ref())
            .await
            .map_err(RepositoryError::from)?
            .ok_or_else(|| RepositoryError::not_found("Question not found"))?;
        to_domain(model)
    }
    async fn create(&self, input: NewQuestion) -> RepositoryResult<domain::ErrorQuestion> {
        let id = input.id;
        let active: error_question::ActiveModel = domain::ErrorQuestion {
            id: id.clone(),
            user_id: input.user_id,
            subject_id: input.subject_id,
            source_id: input.source_id,
            prompt: input.prompt,
            question_type: domain::QuestionType::from(input.type_),
            answer: input.answer,
            analysis: input.analysis,
            error_note: input.error_note,
            metadata: domain::EntityMetadata {
                created_at: input.now,
                updated_at: input.now,
                deleted_at: None,
                version: 0,
                sync_status: domain::SyncStatus::Pending,
                sync_hash: None,
            },
        }
        .into();
        let insert_result = active.insert(self.db.as_ref()).await;
        accept_uuid_insert_result(insert_result)?;
        let model = ErrorQuestion::find_by_id(id)
            .one(self.db.as_ref())
            .await
            .map_err(RepositoryError::from)?
            .ok_or_else(|| RepositoryError::not_found("插入后未能找到新创建的记录"))?;
        to_domain(model)
    }
    async fn update(&self, input: QuestionChanges) -> RepositoryResult<domain::ErrorQuestion> {
        let model = ErrorQuestion::find_by_id(input.id)
            .one(self.db.as_ref())
            .await
            .map_err(RepositoryError::from)?
            .ok_or_else(|| RepositoryError::not_found("Question not found"))?;
        let mut active: error_question::ActiveModel = model.into();
        if let Some(id) = input.subject_id {
            Subject::find_by_id(&id)
                .one(self.db.as_ref())
                .await
                .map_err(RepositoryError::from)?
                .ok_or_else(|| RepositoryError::not_found("Subject not found"))?;
            active.subjectid = Set(id);
        }
        if let Some(v) = input.source_id {
            active.sourceid = Set(Some(v));
        }
        if let Some(v) = input.prompt {
            active.prompt = Set(v);
        }
        if let Some(v) = input.type_ {
            active.type_ = Set(v);
        }
        if let Some(v) = input.answer {
            active.answer = Set(Some(v));
        }
        if let Some(v) = input.analysis {
            active.analysis = Set(Some(v));
        }
        if let Some(v) = input.error_note {
            active.error_note = Set(Some(v));
        }
        active.updated_at = Set(input.now);
        active.sync_status = Set("pending".to_owned());
        let model = active
            .update(self.db.as_ref())
            .await
            .map_err(|e| RepositoryError::context("更新错题失败", e))?;
        to_domain(model)
    }
    async fn soft_delete_with_srs(&self, id: String, now: i64) -> RepositoryResult<()> {
        if let Some(model) = srs_data::Entity::find()
            .filter(srs_data::Column::QuestionId.eq(&id))
            .one(self.db.as_ref())
            .await
            .map_err(RepositoryError::from)?
        {
            let mut active: srs_data::ActiveModel = model.into();
            active.deleted_at = Set(Some(now));
            active.updated_at = Set(now);
            active.sync_status = Set("pending".to_owned());
            active
                .update(self.db.as_ref())
                .await
                .map_err(RepositoryError::from)?;
        }
        let model = ErrorQuestion::find_by_id(id)
            .one(self.db.as_ref())
            .await
            .map_err(RepositoryError::from)?
            .ok_or_else(|| RepositoryError::not_found("Question not found"))?;
        let mut active: error_question::ActiveModel = model.into();
        active.deleted_at = Set(Some(now));
        active.updated_at = Set(now);
        active.sync_status = Set("pending".to_owned());
        active
            .update(self.db.as_ref())
            .await
            .map_err(RepositoryError::from)?;
        Ok(())
    }
    async fn count_active(&self) -> RepositoryResult<u64> {
        ErrorQuestion::find()
            .filter(error_question::Column::DeletedAt.is_null())
            .count(self.db.as_ref())
            .await
            .map_err(RepositoryError::from)
    }
    async fn upsert_synced(&self, input: SyncedQuestion) -> RepositoryResult<()> {
        let existing = ErrorQuestion::find_by_id(&input.id)
            .one(self.db.as_ref())
            .await
            .map_err(|e| RepositoryError::context("Query failed", e))?;
        let is_update = existing.is_some();
        let active: error_question::ActiveModel = if let Some(model) = existing {
            let mut active: error_question::ActiveModel = model.into();
            active.subjectid = Set(input.subject_id);
            active.sourceid = Set(input.source_id);
            active.prompt = Set(input.prompt);
            active.type_ = Set(input.type_);
            active.answer = Set(input.answer);
            active.analysis = Set(input.analysis);
            active.error_note = Set(input.error_note);
            active.updated_at = Set(input.now);
            active.version = Set(input.version);
            active.sync_status = Set("synced".to_owned());
            active.deleted_at = Set(input.deleted_at);
            active
        } else {
            domain::ErrorQuestion {
                id: input.id,
                user_id: input.user_id,
                subject_id: input.subject_id,
                source_id: input.source_id,
                prompt: input.prompt,
                question_type: domain::QuestionType::from(input.type_),
                answer: input.answer,
                analysis: input.analysis,
                error_note: input.error_note,
                metadata: domain::EntityMetadata {
                    created_at: input.now,
                    updated_at: input.now,
                    deleted_at: input.deleted_at,
                    version: input.version,
                    sync_status: domain::SyncStatus::Synced,
                    sync_hash: input.sync_hash,
                },
            }
            .into()
        };
        if is_update {
            active
                .update(self.db.as_ref())
                .await
                .map_err(RepositoryError::from)?;
        } else {
            let _ = active.insert(self.db.as_ref()).await;
        }
        Ok(())
    }
}
