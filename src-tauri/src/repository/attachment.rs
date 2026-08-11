use std::sync::Arc;

use async_trait::async_trait;
use sea_orm::{ActiveModelTrait, ColumnTrait, DbConn, EntityTrait, QueryFilter, Set};

use crate::database::entities::{attachment, prelude::Attachment};
use crate::domain;
use crate::repository::{
    accept_uuid_insert_result, to_domain, to_domains, RepositoryError, RepositoryResult,
};

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
    async fn create(&self, input: NewAttachment) -> RepositoryResult<domain::Attachment>;
    async fn list_active_by_question(
        &self,
        question_id: String,
    ) -> RepositoryResult<Vec<domain::Attachment>>;
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
    async fn create(&self, input: NewAttachment) -> RepositoryResult<domain::Attachment> {
        let id = input.id;
        let active: attachment::ActiveModel = domain::Attachment {
            id: id.clone(),
            question_id: input.question_id,
            attachment_type: domain::AttachmentType::from(input.type_),
            file_type: domain::FileType::from(input.file_type),
            data: input.base64_data,
            hash: input.hash,
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
        let model = Attachment::find_by_id(id)
            .one(self.db.as_ref())
            .await
            .map_err(RepositoryError::from)?
            .ok_or_else(|| RepositoryError::not_found("Attachment not found"))?;
        to_domain(model)
    }

    async fn list_active_by_question(
        &self,
        question_id: String,
    ) -> RepositoryResult<Vec<domain::Attachment>> {
        let models = Attachment::find()
            .filter(attachment::Column::QuestionId.eq(question_id))
            .filter(attachment::Column::DeletedAt.is_null())
            .all(self.db.as_ref())
            .await
            .map_err(RepositoryError::from)?;
        to_domains(models)
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
        active.sync_status = Set("pending".to_owned());
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
        let active: attachment::ActiveModel = if let Some(model) = existing {
            let mut active: attachment::ActiveModel = model.into();
            active.question_id = Set(input.question_id);
            active.type_ = Set(input.type_);
            active.file_type = Set(input.file_type);
            active.base64_data = Set(input.base64_data);
            active.hash = Set(input.hash);
            active.updated_at = Set(input.now);
            active.version = Set(input.version);
            active.sync_status = Set("synced".to_owned());
            active.deleted_at = Set(input.deleted_at);
            active
        } else {
            domain::Attachment {
                id: input.id,
                question_id: input.question_id,
                attachment_type: domain::AttachmentType::from(input.type_),
                file_type: domain::FileType::from(input.file_type),
                data: input.base64_data,
                hash: input.hash,
                metadata: domain::EntityMetadata {
                    created_at: input.now,
                    updated_at: input.now,
                    deleted_at: input.deleted_at,
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
