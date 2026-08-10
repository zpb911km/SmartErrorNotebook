use std::sync::Arc;

use async_trait::async_trait;
use sea_orm::{ActiveModelTrait, ColumnTrait, DbConn, EntityTrait, QueryFilter, Set};

use crate::database::entities::{attachment, prelude::Attachment};
use crate::repository::{accept_uuid_insert_result, RepositoryError, RepositoryResult};

pub struct NewAttachment {
    pub id: String,
    pub question_id: String,
    pub type_: String,
    pub file_type: String,
    pub base64_data: Vec<u8>,
    pub hash: String,
    pub now: i64,
}

pub struct SyncedAttachment {
    pub id: String,
    pub version: i32,
    pub deleted_at: Option<i64>,
    pub question_id: String,
    pub type_: String,
    pub file_type: String,
    pub base64_data: Vec<u8>,
    pub hash: String,
    pub now: i64,
}

#[async_trait]
pub trait AttachmentRepository: Send + Sync {
    async fn create(&self, input: NewAttachment) -> RepositoryResult<attachment::Model>;
    async fn list_active_by_question(
        &self,
        question_id: String,
    ) -> RepositoryResult<Vec<attachment::Model>>;
    async fn soft_delete(&self, id: String, now: i64) -> RepositoryResult<()>;
    async fn upsert_synced(&self, input: SyncedAttachment) -> RepositoryResult<()>;
}

pub struct SeaOrmAttachmentRepository {
    db: Arc<DbConn>,
}
impl SeaOrmAttachmentRepository {
    pub fn new(db: Arc<DbConn>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl AttachmentRepository for SeaOrmAttachmentRepository {
    async fn create(&self, input: NewAttachment) -> RepositoryResult<attachment::Model> {
        let insert_result = attachment::ActiveModel {
            id: Set(input.id.clone()),
            question_id: Set(input.question_id),
            type_: Set(input.type_),
            file_type: Set(input.file_type),
            base64_data: Set(input.base64_data),
            hash: Set(input.hash),
            created_at: Set(input.now),
            updated_at: Set(input.now),
            deleted_at: Set(None),
            version: Set(0),
            sync_status: Set("pending".into()),
            sync_hash: Set(None),
        }
        .insert(self.db.as_ref())
        .await;
        accept_uuid_insert_result(insert_result)?;
        Attachment::find_by_id(input.id)
            .one(self.db.as_ref())
            .await
            .map_err(RepositoryError::from)?
            .ok_or_else(|| RepositoryError::not_found("Attachment not found"))
    }

    async fn list_active_by_question(
        &self,
        question_id: String,
    ) -> RepositoryResult<Vec<attachment::Model>> {
        Attachment::find()
            .filter(attachment::Column::QuestionId.eq(question_id))
            .filter(attachment::Column::DeletedAt.is_null())
            .all(self.db.as_ref())
            .await
            .map_err(RepositoryError::from)
    }

    async fn soft_delete(&self, id: String, now: i64) -> RepositoryResult<()> {
        let model = Attachment::find_by_id(id)
            .one(self.db.as_ref())
            .await
            .map_err(RepositoryError::from)?
            .ok_or_else(|| RepositoryError::not_found("Attachment not found"))?;
        let mut active: attachment::ActiveModel = model.into();
        active.deleted_at = Set(Some(now));
        active.updated_at = Set(now);
        active.sync_status = Set("pending".into());
        active
            .update(self.db.as_ref())
            .await
            .map_err(RepositoryError::from)?;
        Ok(())
    }

    async fn upsert_synced(&self, input: SyncedAttachment) -> RepositoryResult<()> {
        let existing = Attachment::find_by_id(&input.id)
            .one(self.db.as_ref())
            .await
            .map_err(|e| RepositoryError::context("Query failed", e))?;
        let is_update = existing.is_some();
        let active = if let Some(model) = existing {
            let mut active: attachment::ActiveModel = model.into();
            active.question_id = Set(input.question_id);
            active.type_ = Set(input.type_);
            active.file_type = Set(input.file_type);
            active.base64_data = Set(input.base64_data);
            active.hash = Set(input.hash);
            active.updated_at = Set(input.now);
            active.version = Set(input.version);
            active.sync_status = Set("synced".into());
            active.deleted_at = Set(input.deleted_at);
            active
        } else {
            attachment::ActiveModel {
                id: Set(input.id),
                question_id: Set(input.question_id),
                type_: Set(input.type_),
                file_type: Set(input.file_type),
                base64_data: Set(input.base64_data),
                hash: Set(input.hash),
                created_at: Set(input.now),
                updated_at: Set(input.now),
                deleted_at: Set(input.deleted_at),
                version: Set(input.version),
                sync_status: Set("synced".into()),
                sync_hash: Set(None),
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
