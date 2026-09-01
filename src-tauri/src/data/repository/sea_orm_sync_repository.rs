use super::super::database::entity::{
    attachment, question, question_attachment_cross_ref, question_tag_cross_ref, source, srs_data,
    subject, tag,
};
use super::{timestamp, uuid};
use crate::model::legacy as legacy_model;
use crate::repository::legacy as legacy_repository;
use crate::util::{codec, legacy::codec as legacy_codec};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, Set,
};
use serde_json::{json, Value};

pub struct SeaOrmSyncRepository<'c, C: ConnectionTrait> {
    connection: &'c C,
}
impl<'c, C: ConnectionTrait> SeaOrmSyncRepository<'c, C> {
    pub fn new(connection: &'c C) -> Self {
        Self { connection }
    }
}

fn legacy_status(value: &str) -> String {
    match value {
        "PENDING" => "pending",
        "SYNCED" => "synced",
        other => other,
    }
    .to_owned()
}
fn normalized_status(value: legacy_model::SyncStatus) -> String {
    match value {
        legacy_model::SyncStatus::Pending => "PENDING".into(),
        legacy_model::SyncStatus::Synced => "SYNCED".into(),
        legacy_model::SyncStatus::Conflict => "CONFLICT".into(),
    }
}

fn header(
    id: String,
    table: &str,
    created: i64,
    updated: i64,
    deleted: Option<i64>,
    version: i64,
    status: &str,
) -> legacy_repository::repository_model::sync::SyncRecordHeader {
    legacy_repository::repository_model::sync::SyncRecordHeader {
        id,
        table_name: table.into(),
        version: version.clamp(i32::MIN as i64, i32::MAX as i64) as i32,
        status: legacy_status(status),
        deleted_at: deleted,
        updated_at: updated,
        created_at: created,
    }
}
fn output(
    header: legacy_repository::repository_model::sync::SyncRecordHeader,
    data: Value,
) -> legacy_repository::repository_model::sync::SyncRecordOutput {
    legacy_repository::repository_model::sync::SyncRecordOutput {
        id: header.id,
        table_name: header.table_name,
        version: header.version,
        status: header.status,
        deleted_at: header.deleted_at,
        updated_at: header.updated_at,
        data,
    }
}

async fn attachment_questions<C: ConnectionTrait>(db: &C, id: uuid::Uuid) -> Vec<String> {
    question_attachment_cross_ref::Entity::find()
        .filter(question_attachment_cross_ref::Column::AttachmentId.eq(id))
        .order_by_asc(question_attachment_cross_ref::Column::QuestionId)
        .all(db)
        .await
        .expect("failed to query attachment relation")
        .into_iter()
        .map(|value| value.question_id.to_string())
        .collect()
}
async fn tag_questions<C: ConnectionTrait>(db: &C, id: uuid::Uuid) -> Vec<String> {
    question_tag_cross_ref::Entity::find()
        .filter(question_tag_cross_ref::Column::TagId.eq(id))
        .order_by_asc(question_tag_cross_ref::Column::QuestionId)
        .all(db)
        .await
        .expect("failed to query tag relation")
        .into_iter()
        .map(|value| value.question_id.to_string())
        .collect()
}
async fn source_owner<C: ConnectionTrait>(db: &C, id: uuid::Uuid) -> Option<String> {
    question::Entity::find()
        .filter(question::Column::SourceId.eq(id))
        .filter(question::Column::DeletedAt.is_null())
        .order_by_asc(question::Column::Id)
        .one(db)
        .await
        .expect("failed to query source owner")
        .map(|q| q.id.to_string())
}

async fn source_subject<C: ConnectionTrait>(db: &C, id: Option<uuid::Uuid>) -> String {
    let Some(id) = id else {
        return String::new();
    };
    source::Entity::find_by_id(id)
        .one(db)
        .await
        .expect("failed to query question subject")
        .and_then(|source| source.subject_id)
        .map(|id| id.to_string())
        .unwrap_or_default()
}

impl<'c, C: ConnectionTrait> SeaOrmSyncRepository<'c, C> {
    async fn headers(
        &self,
        pending_only: bool,
    ) -> Vec<legacy_repository::repository_model::sync::SyncRecordHeader> {
        let mut out = Vec::new();
        macro_rules! push_headers {
            ($entity:ty,$column:path,$table:expr,$id:expr) => {{
                let mut query = <$entity>::find();
                if pending_only {
                    query = query.filter($column.eq("PENDING"));
                }
                for model in query
                    .all(self.connection)
                    .await
                    .unwrap_or_else(|error| panic!("failed to query {}: {error}", $table))
                {
                    let m = &model;
                    out.push(header(
                        ($id)(m),
                        $table,
                        m.created_at.timestamp(),
                        m.updated_at.timestamp(),
                        m.deleted_at.map(|v| v.timestamp()),
                        m.sync_version,
                        &m.sync_status,
                    ));
                }
            }};
        }
        push_headers!(
            question::Entity,
            question::Column::SyncStatus,
            "error_questions",
            |m: &question::Model| m.id.to_string()
        );
        push_headers!(
            subject::Entity,
            subject::Column::SyncStatus,
            "subjects",
            |m: &subject::Model| m.id.to_string()
        );
        push_headers!(
            srs_data::Entity,
            srs_data::Column::SyncStatus,
            "srs_data",
            |m: &srs_data::Model| m.question_id.to_string()
        );
        push_headers!(
            attachment::Entity,
            attachment::Column::SyncStatus,
            "attachments",
            |m: &attachment::Model| m.id.to_string()
        );
        push_headers!(
            tag::Entity,
            tag::Column::SyncStatus,
            "error_tags",
            |m: &tag::Model| m.id.to_string()
        );
        push_headers!(
            source::Entity,
            source::Column::SyncStatus,
            "sources",
            |m: &source::Model| m.id.to_string()
        );
        out
    }

    async fn output_for(
        &self,
        id: uuid::Uuid,
        table: Option<&str>,
    ) -> Option<legacy_repository::repository_model::sync::SyncRecordOutput> {
        if table.is_none() || table == Some("error_questions") {
            if let Some(m) = question::Entity::find_by_id(id)
                .one(self.connection)
                .await
                .expect("failed to query question")
            {
                let h = header(
                    m.id.to_string(),
                    "error_questions",
                    m.created_at.timestamp(),
                    m.updated_at.timestamp(),
                    m.deleted_at.map(|v| v.timestamp()),
                    m.sync_version,
                    &m.sync_status,
                );
                let type_ = match m.question_type.as_deref() {
                    Some("SINGLE_SELECT") => "单选题",
                    Some("MULTIPLE_SELECT") => "多选题",
                    Some("TRUE_FALSE") => "判断题",
                    Some("FILL_IN_THE_BLANK") => "填空题",
                    Some("SHORT_ANSWER") => "简答题",
                    Some("CALCULATION") => "计算题",
                    Some("ESSAY") => "论述题",
                    Some(v) => v,
                    None => "",
                };
                let subject_id = source_subject(self.connection, m.source_id).await;
                return Some(output(
                    h,
                    json!({"id":m.id.to_string(),"userid":"","subjectid":subject_id,"sourceid":m.source_id.map(|v|v.to_string()),"prompt":m.stem,"type_":type_,"answer":if m.correct_answer.is_empty(){None}else{Some(m.correct_answer)},"analysis":m.explanation,"error_note":m.note,"sync_hash":Value::Null}),
                ));
            }
        }
        if table.is_none() || table == Some("subjects") {
            if let Some(m) = subject::Entity::find_by_id(id)
                .one(self.connection)
                .await
                .expect("failed to query subject")
            {
                let h = header(
                    m.id.to_string(),
                    "subjects",
                    m.created_at.timestamp(),
                    m.updated_at.timestamp(),
                    m.deleted_at.map(|v| v.timestamp()),
                    m.sync_version,
                    &m.sync_status,
                );
                return Some(output(
                    h,
                    json!({"id":m.id.to_string(),"name":m.name,"color":if m.color.is_empty(){None}else{Some(m.color)},"sync_hash":Value::Null}),
                ));
            }
        }
        if table.is_none() || table == Some("srs_data") {
            if let Some(m) = srs_data::Entity::find_by_id(id)
                .one(self.connection)
                .await
                .expect("failed to query srs data")
            {
                let h = header(
                    m.question_id.to_string(),
                    "srs_data",
                    m.created_at.timestamp(),
                    m.updated_at.timestamp(),
                    m.deleted_at.map(|v| v.timestamp()),
                    m.sync_version,
                    &m.sync_status,
                );
                return Some(output(
                    h,
                    json!({"id":m.question_id.to_string(),"question_id":m.question_id.to_string(),"stability":m.stability,"difficulty":m.difficulty,"next_review_at":m.next_review_at.map(|v|v.timestamp()),"lastreviewed_at":m.last_reviewed_at.map(|v|v.timestamp()),"review_count":m.review_count,"feedback_history":m.feedback_history.to_string(),"sync_hash":Value::Null}),
                ));
            }
        }
        if table.is_none() || table == Some("attachments") {
            if let Some(m) = attachment::Entity::find_by_id(id)
                .one(self.connection)
                .await
                .expect("failed to query attachment")
            {
                let question_ids = attachment_questions(self.connection, id).await;
                let h = header(
                    m.id.to_string(),
                    "attachments",
                    m.created_at.timestamp(),
                    m.updated_at.timestamp(),
                    m.deleted_at.map(|v| v.timestamp()),
                    m.sync_version,
                    &m.sync_status,
                );
                return Some(output(
                    h,
                    json!({
                        "id":m.id.to_string(),
                        "question_ids":question_ids,
                        "type_":"original",
                        "file_type":legacy_codec::legacy_file_type(&m.mime_type),
                        "base64_data":codec::encode_base64(&m.data).into_bytes(),
                        "hash":m.sha256,
                        "sync_hash":Value::Null
                    }),
                ));
            }
        }
        if table.is_none() || table == Some("error_tags") {
            if let Some(m) = tag::Entity::find_by_id(id)
                .one(self.connection)
                .await
                .expect("failed to query tag")
            {
                let question_ids = tag_questions(self.connection, id).await;
                let h = header(
                    m.id.to_string(),
                    "error_tags",
                    m.created_at.timestamp(),
                    m.updated_at.timestamp(),
                    m.deleted_at.map(|v| v.timestamp()),
                    m.sync_version,
                    &m.sync_status,
                );
                return Some(output(
                    h,
                    json!({"id":m.id.to_string(),"question_ids":question_ids,"name":m.name,"color":m.color,"sync_hash":Value::Null}),
                ));
            }
        }
        if table.is_none() || table == Some("sources") {
            if let Some(m) = source::Entity::find_by_id(id)
                .one(self.connection)
                .await
                .expect("failed to query source")
            {
                let owner = source_owner(self.connection, id).await;
                let h = header(
                    m.id.to_string(),
                    "sources",
                    m.created_at.timestamp(),
                    m.updated_at.timestamp(),
                    m.deleted_at.map(|v| v.timestamp()),
                    m.sync_version,
                    &m.sync_status,
                );
                return Some(output(
                    h,
                    json!({"id":m.id.to_string(),"question_id":owner,"subject_id":m.subject_id.map(|id|id.to_string()),"book":m.book,"chapter":m.chapter,"knowledge":m.knowledge,"sync_hash":Value::Null}),
                ));
            }
        }
        None
    }
}

#[async_trait::async_trait]
impl<'c, C: ConnectionTrait> legacy_repository::SyncRepository for SeaOrmSyncRepository<'c, C> {
    async fn all_headers(
        &self,
    ) -> Vec<legacy_repository::repository_model::sync::SyncRecordHeader> {
        self.headers(false).await
    }

    async fn pending_records(
        &self,
    ) -> Vec<legacy_repository::repository_model::sync::SyncRecordOutput> {
        let headers = self.headers(true).await;
        let mut out = Vec::new();
        for h in headers {
            if let Some(record) = self
                .output_for(uuid(&h.id, "record id"), Some(&h.table_name))
                .await
            {
                out.push(record);
            }
        }
        out
    }

    async fn record_for_upload(
        &self,
        table_name: Option<String>,
        id: String,
    ) -> Result<legacy_repository::repository_model::sync::SyncRecordOutput, String> {
        let parsed_id =
            uuid::Uuid::parse_str(&id).map_err(|_| format!("invalid record id: {id}"))?;
        self.output_for(parsed_id, table_name.as_deref())
            .await
            .ok_or_else(|| match table_name {
                Some(table_name) => format!("record not found: {table_name}:{id}"),
                None => format!("Record not found with id: {id}"),
            })
    }

    async fn set_status_version(
        &self,
        table_name: Option<String>,
        id: String,
        status: legacy_model::SyncStatus,
        version: i32,
    ) -> Result<String, String> {
        let original_id = id;
        let id = uuid::Uuid::parse_str(&original_id)
            .map_err(|_| format!("invalid record id: {original_id}"))?;
        let table_name = match table_name {
            Some(table_name) => table_name,
            None => self
                .output_for(id, None)
                .await
                .map(|record| record.table_name)
                .ok_or_else(|| format!("Record not found with id: {original_id}"))?,
        };
        let status = normalized_status(status);
        macro_rules! update_one {
            ($entity:ty,$active:ty) => {{
                if let Some(model) = <$entity>::find_by_id(id)
                    .one(self.connection)
                    .await
                    .map_err(|error| {
                        format!("failed to query {table_name}:{original_id}: {error}")
                    })?
                {
                    let mut a: $active = model.into();
                    a.sync_status = Set(status.clone());
                    a.sync_version = Set(i64::from(version));
                    a.update(self.connection).await.map_err(|error| {
                        format!("failed to update {table_name}:{original_id}: {error}")
                    })?;
                    Ok(format!("Record updated: {table_name}:{original_id}"))
                } else {
                    Err(format!("record not found: {table_name}:{original_id}"))
                }
            }};
        }
        match table_name.as_str() {
            "error_questions" => update_one!(question::Entity, question::ActiveModel),
            "subjects" => update_one!(subject::Entity, subject::ActiveModel),
            "srs_data" => update_one!(srs_data::Entity, srs_data::ActiveModel),
            "attachments" => update_one!(attachment::Entity, attachment::ActiveModel),
            "error_tags" => update_one!(tag::Entity, tag::ActiveModel),
            "sources" => update_one!(source::Entity, source::ActiveModel),
            _ => Err(format!("unsupported sync table: {table_name}")),
        }
    }

    async fn purge_synced_deletions(&self) -> Value {
        macro_rules! purge {
            ($entity:ty,$status:path,$deleted:path) => {
                <$entity>::delete_many()
                    .filter($status.eq("SYNCED"))
                    .filter($deleted.is_not_null())
                    .exec(self.connection)
                    .await
                    .expect("failed to purge synced records")
                    .rows_affected
            };
        }
        let srs = purge!(
            srs_data::Entity,
            srs_data::Column::SyncStatus,
            srs_data::Column::DeletedAt
        );
        let attachments = purge!(
            attachment::Entity,
            attachment::Column::SyncStatus,
            attachment::Column::DeletedAt
        );
        let tags = purge!(tag::Entity, tag::Column::SyncStatus, tag::Column::DeletedAt);
        let questions = purge!(
            question::Entity,
            question::Column::SyncStatus,
            question::Column::DeletedAt
        );
        let sources = purge!(
            source::Entity,
            source::Column::SyncStatus,
            source::Column::DeletedAt
        );
        let subjects = purge!(
            subject::Entity,
            subject::Column::SyncStatus,
            subject::Column::DeletedAt
        );
        json!({
            "error_questions":{"deleted":questions},
            "subjects":{"deleted":subjects},
            "srs_data":{"deleted":srs},
            "attachments":{"deleted":attachments},
            "error_tags":{"deleted":tags},
            "sources":{"deleted":sources}
        })
    }

    async fn check_orphans(
        &self,
        now: i64,
    ) -> legacy_repository::repository_model::sync::CascadeOrphanCheckResult {
        let now = timestamp(now, "orphan check timestamp");
        let attachments = attachment::Entity::find()
            .filter(attachment::Column::DeletedAt.is_null())
            .all(self.connection)
            .await
            .expect("failed to query attachments");
        let tags = tag::Entity::find()
            .filter(tag::Column::DeletedAt.is_null())
            .all(self.connection)
            .await
            .expect("failed to query tags");
        let total_checked = attachments.len() + tags.len();
        let mut deleted = Vec::new();
        for model in attachments {
            if question_attachment_cross_ref::Entity::find()
                .filter(question_attachment_cross_ref::Column::AttachmentId.eq(model.id))
                .count(self.connection)
                .await
                .expect("failed to query attachment relations")
                == 0
            {
                let id = model.id;
                let mut a: attachment::ActiveModel = model.into();
                a.deleted_at = Set(Some(now));
                a.updated_at = Set(now);
                a.sync_status = Set("PENDING".into());
                a.update(self.connection)
                    .await
                    .expect("failed to soft-delete orphan attachment");
                deleted.push(format!("attachments:{id}"));
            }
        }
        for model in tags {
            if question_tag_cross_ref::Entity::find()
                .filter(question_tag_cross_ref::Column::TagId.eq(model.id))
                .count(self.connection)
                .await
                .expect("failed to query tag relations")
                == 0
            {
                let id = model.id;
                let mut a: tag::ActiveModel = model.into();
                a.deleted_at = Set(Some(now));
                a.updated_at = Set(now);
                a.sync_status = Set("PENDING".into());
                a.update(self.connection)
                    .await
                    .expect("failed to soft-delete orphan tag");
                deleted.push(format!("error_tags:{id}"));
            }
        }
        legacy_repository::repository_model::sync::CascadeOrphanCheckResult {
            orphan_records_soft_deleted: deleted,
            total_checked,
        }
    }
}
