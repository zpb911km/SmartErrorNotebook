use crate::database::entities::{prelude::SrsData, srs_data};
use crate::repository::{accept_uuid_insert_result, RepositoryError, RepositoryResult};
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
    ) -> RepositoryResult<Option<srs_data::Model>>;
    async fn list_active(&self) -> RepositoryResult<Vec<srs_data::Model>>;
    async fn create(&self, input: NewSrsData) -> RepositoryResult<srs_data::Model>;
    async fn update_state(&self, input: SrsStateChanges) -> RepositoryResult<srs_data::Model>;
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
    ) -> RepositoryResult<Option<srs_data::Model>> {
        let mut q = SrsData::find().filter(srs_data::Column::QuestionId.eq(question_id));
        if active_only {
            q = q.filter(srs_data::Column::DeletedAt.is_null());
        }
        q.one(self.db.as_ref()).await.map_err(RepositoryError::from)
    }
    async fn list_active(&self) -> RepositoryResult<Vec<srs_data::Model>> {
        SrsData::find()
            .filter(srs_data::Column::DeletedAt.is_null())
            .all(self.db.as_ref())
            .await
            .map_err(RepositoryError::from)
    }
    async fn create(&self, input: NewSrsData) -> RepositoryResult<srs_data::Model> {
        let insert_result = srs_data::ActiveModel {
            id: Set(input.id.clone()),
            question_id: Set(input.question_id),
            stability: Set(input.stability),
            difficulty: Set(input.difficulty),
            next_review_at: Set(input.next_review_at),
            lastreviewed_at: Set(input.last_reviewed_at),
            review_count: Set(input.review_count),
            feedback_history: Set(input.feedback_history),
            created_at: Set(input.now),
            updated_at: Set(input.now),
            version: Set(0),
            sync_status: Set("pending".into()),
            sync_hash: Set(None),
            deleted_at: Set(None),
        }
        .insert(self.db.as_ref())
        .await;
        accept_uuid_insert_result(insert_result)?;
        SrsData::find_by_id(input.id)
            .one(self.db.as_ref())
            .await
            .map_err(RepositoryError::from)?
            .ok_or_else(|| RepositoryError::not_found("SRS 数据插入后查询失败"))
    }
    async fn update_state(&self, input: SrsStateChanges) -> RepositoryResult<srs_data::Model> {
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
        active.sync_status = Set("pending".into());
        active.deleted_at = Set(input.deleted_at);
        active
            .update(self.db.as_ref())
            .await
            .map_err(RepositoryError::from)
    }
    async fn upsert_synced(&self, input: SyncedSrsData) -> RepositoryResult<()> {
        let existing = SrsData::find_by_id(&input.id)
            .one(self.db.as_ref())
            .await
            .map_err(|e| RepositoryError::context("Query failed", e))?;
        let is_update = existing.is_some();
        let active = if let Some(model) = existing {
            let mut a: srs_data::ActiveModel = model.into();
            a.question_id = Set(input.question_id);
            a.stability = Set(input.stability);
            a.difficulty = Set(input.difficulty);
            a.next_review_at = Set(input.next_review_at);
            a.lastreviewed_at = Set(input.last_reviewed_at);
            a.review_count = Set(input.review_count);
            a.feedback_history = Set(input.feedback_history);
            a.updated_at = Set(input.now);
            a.version = Set(input.version);
            a.sync_status = Set("synced".into());
            a.deleted_at = Set(input.deleted_at);
            a
        } else {
            srs_data::ActiveModel {
                id: Set(input.id),
                question_id: Set(input.question_id),
                stability: Set(input.stability),
                difficulty: Set(input.difficulty),
                next_review_at: Set(input.next_review_at),
                lastreviewed_at: Set(input.last_reviewed_at),
                review_count: Set(input.review_count),
                feedback_history: Set(input.feedback_history),
                created_at: Set(input.now),
                updated_at: Set(input.now),
                version: Set(input.version),
                sync_status: Set("synced".into()),
                sync_hash: Set(None),
                deleted_at: Set(None),
            }
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
