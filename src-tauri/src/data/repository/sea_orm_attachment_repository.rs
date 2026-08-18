use super::super::database::entity::{attachment, question_attachment_cross_ref};
use super::{timestamp, uuid};
use crate::data::util::{codec, legacy::codec as legacy_codec};
use crate::model::Attachment;
use crate::repository::legacy;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, PaginatorTrait, QueryFilter, Set,
};
use std::collections::HashSet;

pub struct SeaOrmAttachmentRepository<'c, C: ConnectionTrait> {
    connection: &'c C,
}
impl<'c, C: ConnectionTrait> SeaOrmAttachmentRepository<'c, C> {
    pub fn new(connection: &'c C) -> Self {
        Self {
            connection: connection,
        }
    }
}

#[async_trait::async_trait]
impl<'c, C: ConnectionTrait> legacy::AttachmentRepository for SeaOrmAttachmentRepository<'c, C> {
    async fn create(
        &self,
        input: legacy::repository_model::attachment::NewAttachment,
    ) -> Result<Attachment, String> {
        let id = uuid(&input.id, "attachment id");
        let question_id = uuid(&input.question_id, "question id");
        let now = timestamp(input.now, "attachment timestamp");
        let data = codec::decode_base64(&input.base64_data)?;
        let mime_type = legacy_codec::mime_type(&data, &input.file_type);
        let sha256 = codec::sha256(&data);
        let model = attachment::ActiveModel {
            id: Set(id),
            created_at: Set(now),
            updated_at: Set(now),
            deleted_at: Set(None),
            sync_status: Set("PENDING".into()),
            sync_version: Set(0),
            mime_type: Set(mime_type),
            data: Set(data),
            sha256: Set(sha256),
        }
        .insert(self.connection)
        .await
        .expect("failed to create attachment");
        question_attachment_cross_ref::ActiveModel {
            question_id: Set(question_id),
            attachment_id: Set(id),
        }
        .insert(self.connection)
        .await
        .expect("failed to relate attachment to question");
        Ok(model.into())
    }

    async fn list_active_by_question(&self, question_id: String) -> Vec<Attachment> {
        let links = question_attachment_cross_ref::Entity::find()
            .filter(
                question_attachment_cross_ref::Column::QuestionId
                    .eq(uuid(&question_id, "question id")),
            )
            .all(self.connection)
            .await
            .expect("failed to query question attachments");
        if links.is_empty() {
            return Vec::new();
        }
        attachment::Entity::find()
            .filter(attachment::Column::Id.is_in(links.into_iter().map(|link| link.attachment_id)))
            .filter(attachment::Column::DeletedAt.is_null())
            .all(self.connection)
            .await
            .expect("failed to query attachments")
            .into_iter()
            .map(Into::into)
            .collect()
    }

    async fn unlink(&self, question_id: String, id: String, now: i64) -> Result<(), String> {
        let id = uuid(&id, "attachment id");
        let question_id = uuid(&question_id, "question id");
        let deleted = question_attachment_cross_ref::Entity::delete_by_id((question_id, id))
            .exec(self.connection)
            .await
            .map_err(|error| format!("failed to unlink attachment: {error}"))?;
        if deleted.rows_affected == 0 {
            return Err("attachment is not linked to the question".into());
        }
        let model = attachment::Entity::find_by_id(id)
            .one(self.connection)
            .await
            .map_err(|error| format!("failed to query attachment: {error}"))?
            .ok_or_else(|| "Attachment not found".to_owned())?;
        let remaining = question_attachment_cross_ref::Entity::find()
            .filter(question_attachment_cross_ref::Column::AttachmentId.eq(id))
            .count(self.connection)
            .await
            .map_err(|error| format!("failed to count attachment links: {error}"))?;
        let mut active: attachment::ActiveModel = model.into();
        let now = timestamp(now, "attachment timestamp");
        if remaining == 0 {
            active.deleted_at = Set(Some(now));
        }
        active.updated_at = Set(now);
        active.sync_status = Set("PENDING".into());
        active
            .update(self.connection)
            .await
            .map_err(|error| format!("failed to update attachment after unlink: {error}"))?;
        Ok(())
    }

    async fn upsert_synced(
        &self,
        input: legacy::repository_model::attachment::SyncedAttachment,
    ) -> Result<(), String> {
        let id = uuid(&input.id, "attachment id");
        let question_ids: HashSet<_> = input
            .question_ids
            .iter()
            .map(|value| uuid(value, "question id"))
            .collect();
        if input.deleted_at.is_none() && question_ids.is_empty() {
            return Err("an active attachment must be linked to at least one question".into());
        }
        let now = timestamp(input.now, "attachment timestamp");
        let deleted_at = input
            .deleted_at
            .map(|value| timestamp(value, "attachment deleted_at"));
        let data = legacy_codec::decode_legacy_bytes(&input.base64_data)?;
        let mime_type = legacy_codec::mime_type(&data, &input.file_type);
        let sha256 = codec::sha256(&data);
        let exists = attachment::Entity::find_by_id(id)
            .one(self.connection)
            .await
            .expect("failed to query attachment")
            .is_some();
        let mut active = attachment::ActiveModel {
            id: Set(id),
            created_at: Set(now),
            updated_at: Set(now),
            deleted_at: Set(deleted_at),
            sync_status: Set("SYNCED".into()),
            sync_version: Set(i64::from(input.version)),
            mime_type: Set(mime_type),
            data: Set(data),
            sha256: Set(sha256),
        };
        if exists {
            active.id = sea_orm::ActiveValue::Unchanged(id);
            active
                .update(self.connection)
                .await
                .expect("failed to update attachment");
        } else {
            active
                .insert(self.connection)
                .await
                .expect("failed to insert attachment");
        }
        let existing_links = question_attachment_cross_ref::Entity::find()
            .filter(question_attachment_cross_ref::Column::AttachmentId.eq(id))
            .all(self.connection)
            .await
            .map_err(|error| format!("failed to query attachment relations: {error}"))?;
        let existing_question_ids: HashSet<_> =
            existing_links.iter().map(|link| link.question_id).collect();
        for question_id in existing_question_ids.difference(&question_ids) {
            question_attachment_cross_ref::Entity::delete_by_id((*question_id, id))
                .exec(self.connection)
                .await
                .map_err(|error| format!("failed to remove attachment relation: {error}"))?;
        }
        for question_id in question_ids.difference(&existing_question_ids) {
            question_attachment_cross_ref::ActiveModel {
                question_id: Set(*question_id),
                attachment_id: Set(id),
            }
            .insert(self.connection)
            .await
            .map_err(|error| format!("failed to insert attachment relation: {error}"))?;
        }
        Ok(())
    }
}
