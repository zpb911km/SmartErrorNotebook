use crate::database::entities::{prelude::SrsData, srs_data};
use crate::domain;
use crate::repository::{
    accept_uuid_insert_result, to_domain, to_domains, RepositoryError, RepositoryResult,
};
use async_trait::async_trait;
use sea_orm::{ActiveModelTrait, ColumnTrait, DbConn, EntityTrait, QueryFilter, Set};
use std::sync::Arc;

pub struct NewSrsData {
    pub id: String,
    pub question_id: String,
    pub stability: f32,
    pub difficulty: f32,
    pub next_review_at: Option<i64>,
    pub last_reviewed_at: Option<i64>,
    pub review_count: i32,
    pub feedback_history: String,
    pub now: i64,
}
pub struct SrsStateChanges {
    pub id: String,
    pub stability: f32,
    pub difficulty: f32,
    pub next_review_at: Option<i64>,
    pub last_reviewed_at: Option<i64>,
    pub review_count: i32,
    pub feedback_history: String,
    pub deleted_at: Option<i64>,
    pub now: i64,
}
pub struct SyncedSrsData {
    pub id: String,
    pub version: i32,
    pub deleted_at: Option<i64>,
    pub question_id: String,
    pub stability: f32,
    pub difficulty: f32,
    pub next_review_at: Option<i64>,
    pub last_reviewed_at: Option<i64>,
    pub review_count: i32,
    pub feedback_history: String,
    pub now: i64,
}

#[async_trait]
pub trait SrsDataRepository: Send + Sync {
    async fn find_by_question(
        &self,
        question_id: String,
        active_only: bool,
    ) -> RepositoryResult<Option<domain::SrsData>>;
    async fn list_active(&self) -> RepositoryResult<Vec<domain::SrsData>>;
    async fn create(&self, input: NewSrsData) -> RepositoryResult<domain::SrsData>;
    async fn update_state(&self, input: SrsStateChanges) -> RepositoryResult<domain::SrsData>;
    async fn upsert_synced(&self, input: SyncedSrsData) -> RepositoryResult<()>;
}
pub struct SeaOrmSrsDataRepository {
    db: Arc<DbConn>,
}
impl SeaOrmSrsDataRepository {
    pub fn new(db: Arc<DbConn>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl SrsDataRepository for SeaOrmSrsDataRepository {
    async fn find_by_question(
        &self,
        question_id: String,
        active_only: bool,
    ) -> RepositoryResult<Option<domain::SrsData>> {
        let mut q = SrsData::find().filter(srs_data::Column::QuestionId.eq(question_id));
        if active_only {
            q = q.filter(srs_data::Column::DeletedAt.is_null());
        }
        q.one(self.db.as_ref())
            .await
            .map_err(RepositoryError::from)?
            .map(to_domain)
            .transpose()
    }
    async fn list_active(&self) -> RepositoryResult<Vec<domain::SrsData>> {
        let models = SrsData::find()
            .filter(srs_data::Column::DeletedAt.is_null())
            .all(self.db.as_ref())
            .await
            .map_err(RepositoryError::from)?;
        to_domains(models)
    }
    async fn create(&self, input: NewSrsData) -> RepositoryResult<domain::SrsData> {
        let id = input.id;
        let active: srs_data::ActiveModel = domain::SrsData {
            id: id.clone(),
            question_id: input.question_id,
            stability: input.stability,
            difficulty: input.difficulty,
            next_review_at: input.next_review_at,
            last_review_at: input.last_reviewed_at,
            review_count: input.review_count,
            feedback_history: input.feedback_history,
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
        let model = SrsData::find_by_id(id)
            .one(self.db.as_ref())
            .await
            .map_err(RepositoryError::from)?
            .ok_or_else(|| RepositoryError::not_found("SRS 数据插入后查询失败"))?;
        to_domain(model)
    }
    async fn update_state(&self, input: SrsStateChanges) -> RepositoryResult<domain::SrsData> {
        let model = SrsData::find_by_id(input.id)
            .one(self.db.as_ref())
            .await
            .map_err(RepositoryError::from)?
            .ok_or_else(|| RepositoryError::not_found("SRS data not found"))?;
        let mut active: srs_data::ActiveModel = model.into();
        active.stability = Set(input.stability);
        active.difficulty = Set(input.difficulty);
        active.next_review_at = Set(input.next_review_at);
        active.lastreviewed_at = Set(input.last_reviewed_at);
        active.review_count = Set(input.review_count);
        active.feedback_history = Set(input.feedback_history);
        active.updated_at = Set(input.now);
        active.sync_status = Set("pending".to_owned());
        active.deleted_at = Set(input.deleted_at);
        let model = active
            .update(self.db.as_ref())
            .await
            .map_err(RepositoryError::from)?;
        to_domain(model)
    }
    async fn upsert_synced(&self, input: SyncedSrsData) -> RepositoryResult<()> {
        let existing = SrsData::find_by_id(&input.id)
            .one(self.db.as_ref())
            .await
            .map_err(|e| RepositoryError::context("Query failed", e))?;
        let is_update = existing.is_some();
        let active: srs_data::ActiveModel = if let Some(model) = existing {
            let mut active: srs_data::ActiveModel = model.into();
            active.question_id = Set(input.question_id);
            active.stability = Set(input.stability);
            active.difficulty = Set(input.difficulty);
            active.next_review_at = Set(input.next_review_at);
            active.lastreviewed_at = Set(input.last_reviewed_at);
            active.review_count = Set(input.review_count);
            active.feedback_history = Set(input.feedback_history);
            active.updated_at = Set(input.now);
            active.version = Set(input.version);
            active.sync_status = Set("synced".to_owned());
            active.deleted_at = Set(input.deleted_at);
            active
        } else {
            domain::SrsData {
                id: input.id,
                question_id: input.question_id,
                stability: input.stability,
                difficulty: input.difficulty,
                next_review_at: input.next_review_at,
                last_review_at: input.last_reviewed_at,
                review_count: input.review_count,
                feedback_history: input.feedback_history,
                metadata: domain::EntityMetadata {
                    created_at: input.now,
                    updated_at: input.now,
                    deleted_at: None,
                    version: input.version,
                    sync_status: domain::SyncStatus::Synced,
                    sync_hash: None,
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
