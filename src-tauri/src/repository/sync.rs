use async_trait::async_trait;
use sea_orm::{ActiveModelTrait, ActiveValue, ColumnTrait, DbConn, EntityTrait, QueryFilter};
use serde::Serialize;
use std::sync::Arc;

use crate::domain::SyncStatus;
use crate::repository::{RepositoryError, RepositoryResult};

#[derive(Debug, Clone, Serialize)]
pub struct SyncRecordHeader {
    pub id: String,
    pub table_name: String,
    pub version: i32,
    pub status: String,
    pub deleted_at: Option<i64>,
    pub updated_at: i64,
    pub created_at: i64,
}
#[derive(Debug, Clone, Serialize)]
pub struct SyncRecordOutput {
    pub id: String,
    pub table_name: String,
    pub version: i32,
    pub status: String,
    pub deleted_at: Option<i64>,
    pub updated_at: i64,
    #[serde(rename = "data")]
    pub data: serde_json::Value,
}
#[derive(Debug, Serialize)]
pub struct CascadeOrphanCheckResult {
    pub orphan_records_soft_deleted: Vec<String>,
    pub total_checked: usize,
}

#[async_trait]
pub trait SyncRepository: Send + Sync {
    async fn all_headers(&self) -> RepositoryResult<Vec<SyncRecordHeader>>;
    async fn pending_records(&self) -> RepositoryResult<Vec<SyncRecordOutput>>;
    async fn record_for_upload(&self, id: String) -> RepositoryResult<SyncRecordOutput>;
    async fn set_status_version(
        &self,
        id: String,
        status: SyncStatus,
        version: i32,
    ) -> RepositoryResult<String>;
    async fn purge_synced_deletions(&self) -> RepositoryResult<serde_json::Value>;
    async fn check_orphans(&self, now: i64) -> RepositoryResult<CascadeOrphanCheckResult>;
}
pub struct SeaOrmSyncRepository {
    db: Arc<DbConn>,
}
impl SeaOrmSyncRepository {
    pub fn new(db: Arc<DbConn>) -> Self {
        Self { db }
    }
}

fn json_object<M: Serialize>(model: M) -> serde_json::Map<String, serde_json::Value> {
    serde_json::to_value(model)
        .unwrap_or_default()
        .as_object()
        .cloned()
        .unwrap_or_default()
}
fn to_output<M: Serialize>(model: M, table: &str) -> SyncRecordOutput {
    let obj = json_object(model);
    let mut data = obj.clone();
    for key in [
        "version",
        "sync_status",
        "deleted_at",
        "updated_at",
        "created_at",
    ] {
        data.remove(key);
    }
    SyncRecordOutput {
        id: obj.get("id").and_then(|v| v.as_str()).unwrap_or("").into(),
        table_name: table.into(),
        version: obj.get("version").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
        status: obj
            .get("sync_status")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .into(),
        deleted_at: obj.get("deleted_at").and_then(|v| v.as_i64()),
        updated_at: obj.get("updated_at").and_then(|v| v.as_i64()).unwrap_or(0),
        data: serde_json::Value::Object(data),
    }
}
fn to_header<M: Serialize>(model: M, table: &str) -> SyncRecordHeader {
    let obj = json_object(model);
    SyncRecordHeader {
        id: obj.get("id").and_then(|v| v.as_str()).unwrap_or("").into(),
        table_name: table.into(),
        version: obj.get("version").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
        status: obj
            .get("sync_status")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .into(),
        deleted_at: obj.get("deleted_at").and_then(|v| v.as_i64()),
        updated_at: obj.get("updated_at").and_then(|v| v.as_i64()).unwrap_or(0),
        created_at: obj.get("created_at").and_then(|v| v.as_i64()).unwrap_or(0),
    }
}

#[async_trait]
impl SyncRepository for SeaOrmSyncRepository {
    async fn all_headers(&self) -> RepositoryResult<Vec<SyncRecordHeader>> {
        use crate::database::entities::{
            attachment as att, error_question as eq, error_tag as tag, source as src,
            srs_data as srs, subject as sub,
        };
        let db = self.db.as_ref();
        let mut out = Vec::new();
        out.extend(
            eq::Entity::find()
                .all(db)
                .await
                .map_err(|e| RepositoryError::context("Failed to query error_questions", e))?
                .into_iter()
                .map(|m| to_header(m, "error_questions")),
        );
        out.extend(
            sub::Entity::find()
                .all(db)
                .await
                .map_err(|e| RepositoryError::context("Failed to query subjects", e))?
                .into_iter()
                .map(|m| to_header(m, "subjects")),
        );
        out.extend(
            srs::Entity::find()
                .all(db)
                .await
                .map_err(|e| RepositoryError::context("Failed to query srs_data", e))?
                .into_iter()
                .map(|m| to_header(m, "srs_data")),
        );
        out.extend(
            att::Entity::find()
                .all(db)
                .await
                .map_err(|e| RepositoryError::context("Failed to query attachments", e))?
                .into_iter()
                .map(|m| to_header(m, "attachments")),
        );
        out.extend(
            tag::Entity::find()
                .all(db)
                .await
                .map_err(|e| RepositoryError::context("Failed to query error_tags", e))?
                .into_iter()
                .map(|m| to_header(m, "error_tags")),
        );
        out.extend(
            src::Entity::find()
                .all(db)
                .await
                .map_err(|e| RepositoryError::context("Failed to query sources", e))?
                .into_iter()
                .map(|m| to_header(m, "sources")),
        );
        Ok(out)
    }
    async fn pending_records(&self) -> RepositoryResult<Vec<SyncRecordOutput>> {
        use crate::database::entities::{
            attachment as att, error_question as eq, error_tag as tag, source as src,
            srs_data as srs, subject as sub,
        };
        let db = self.db.as_ref();
        let mut out = Vec::new();
        out.extend(
            eq::Entity::find()
                .filter(eq::Column::SyncStatus.eq("pending"))
                .all(db)
                .await
                .map_err(|e| RepositoryError::context("Failed to query error_questions", e))?
                .into_iter()
                .map(|m| to_output(m, "error_questions")),
        );
        out.extend(
            sub::Entity::find()
                .filter(sub::Column::SyncStatus.eq("pending"))
                .all(db)
                .await
                .map_err(|e| RepositoryError::context("Failed to query subjects", e))?
                .into_iter()
                .map(|m| to_output(m, "subjects")),
        );
        out.extend(
            srs::Entity::find()
                .filter(srs::Column::SyncStatus.eq("pending"))
                .all(db)
                .await
                .map_err(|e| RepositoryError::context("Failed to query srs_data", e))?
                .into_iter()
                .map(|m| to_output(m, "srs_data")),
        );
        out.extend(
            att::Entity::find()
                .filter(att::Column::SyncStatus.eq("pending"))
                .all(db)
                .await
                .map_err(|e| RepositoryError::context("Failed to query attachments", e))?
                .into_iter()
                .map(|m| to_output(m, "attachments")),
        );
        out.extend(
            tag::Entity::find()
                .filter(tag::Column::SyncStatus.eq("pending"))
                .all(db)
                .await
                .map_err(|e| RepositoryError::context("Failed to query error_tags", e))?
                .into_iter()
                .map(|m| to_output(m, "error_tags")),
        );
        out.extend(
            src::Entity::find()
                .filter(src::Column::SyncStatus.eq("pending"))
                .all(db)
                .await
                .map_err(|e| RepositoryError::context("Failed to query sources", e))?
                .into_iter()
                .map(|m| to_output(m, "sources")),
        );
        Ok(out)
    }
    async fn record_for_upload(&self, id: String) -> RepositoryResult<SyncRecordOutput> {
        use crate::database::entities::{
            attachment as att, error_question as eq, error_tag as tag, source as src,
            srs_data as srs, subject as sub,
        };
        let db = self.db.as_ref();
        if let Some(m) = eq::Entity::find_by_id(&id)
            .one(db)
            .await
            .map_err(|e| RepositoryError::context("Failed to query error_questions", e))?
        {
            return Ok(to_output(m, "error_questions"));
        }
        if let Some(m) = sub::Entity::find_by_id(&id)
            .one(db)
            .await
            .map_err(|e| RepositoryError::context("Failed to query subjects", e))?
        {
            return Ok(to_output(m, "subjects"));
        }
        if let Some(m) = srs::Entity::find_by_id(&id)
            .one(db)
            .await
            .map_err(|e| RepositoryError::context("Failed to query srs_data", e))?
        {
            return Ok(to_output(m, "srs_data"));
        }
        if let Some(m) = att::Entity::find_by_id(&id)
            .one(db)
            .await
            .map_err(|e| RepositoryError::context("Failed to query attachments", e))?
        {
            return Ok(to_output(m, "attachments"));
        }
        if let Some(m) = tag::Entity::find_by_id(&id)
            .one(db)
            .await
            .map_err(|e| RepositoryError::context("Failed to query error_tags", e))?
        {
            return Ok(to_output(m, "error_tags"));
        }
        if let Some(m) = src::Entity::find_by_id(&id)
            .one(db)
            .await
            .map_err(|e| RepositoryError::context("Failed to query sources", e))?
        {
            return Ok(to_output(m, "sources"));
        }
        Err(RepositoryError::not_found(format!(
            "Record not found with id: {id}"
        )))
    }
    async fn set_status_version(
        &self,
        id: String,
        status: SyncStatus,
        version: i32,
    ) -> RepositoryResult<String> {
        use crate::database::entities::{
            attachment as att, error_question as eq, error_tag as tag, source as src,
            srs_data as srs, subject as sub,
        };
        let db = self.db.as_ref();
        let status = String::from(status);
        if let Some(m) = eq::Entity::find_by_id(&id)
            .one(db)
            .await
            .map_err(|e| RepositoryError::context("Failed to query error_questions", e))?
        {
            let mut a: eq::ActiveModel = m.into();
            a.sync_status = ActiveValue::Set(status.clone());
            a.version = ActiveValue::Set(version);
            let r = a
                .update(db)
                .await
                .map_err(|e| RepositoryError::context("Failed to update error_questions", e))?;
            return Ok(format!("Record updated with id: {:?}", r.id));
        }
        if let Some(m) = sub::Entity::find_by_id(&id)
            .one(db)
            .await
            .map_err(|e| RepositoryError::context("Failed to query subjects", e))?
        {
            let mut a: sub::ActiveModel = m.into();
            a.sync_status = ActiveValue::Set(status.clone());
            a.version = ActiveValue::Set(version);
            let r = a
                .update(db)
                .await
                .map_err(|e| RepositoryError::context("Failed to update subjects", e))?;
            return Ok(format!("Record updated with id: {:?}", r.id));
        }
        if let Some(m) = srs::Entity::find_by_id(&id)
            .one(db)
            .await
            .map_err(|e| RepositoryError::context("Failed to query srs_data", e))?
        {
            let mut a: srs::ActiveModel = m.into();
            a.sync_status = ActiveValue::Set(status.clone());
            a.version = ActiveValue::Set(version);
            let r = a
                .update(db)
                .await
                .map_err(|e| RepositoryError::context("Failed to update srs_data", e))?;
            return Ok(format!("Record updated with id: {:?}", r.id));
        }
        if let Some(m) = att::Entity::find_by_id(&id)
            .one(db)
            .await
            .map_err(|e| RepositoryError::context("Failed to query attachments", e))?
        {
            let mut a: att::ActiveModel = m.into();
            a.sync_status = ActiveValue::Set(status.clone());
            a.version = ActiveValue::Set(version);
            let r = a
                .update(db)
                .await
                .map_err(|e| RepositoryError::context("Failed to update attachments", e))?;
            return Ok(format!("Record updated with id: {:?}", r.id));
        }
        if let Some(m) = tag::Entity::find_by_id(&id)
            .one(db)
            .await
            .map_err(|e| RepositoryError::context("Failed to query error_tags", e))?
        {
            let mut a: tag::ActiveModel = m.into();
            a.sync_status = ActiveValue::Set(status.clone());
            a.version = ActiveValue::Set(version);
            let r = a
                .update(db)
                .await
                .map_err(|e| RepositoryError::context("Failed to update error_tags", e))?;
            return Ok(format!("Record updated with id: {:?}", r.id));
        }
        if let Some(m) = src::Entity::find_by_id(&id)
            .one(db)
            .await
            .map_err(|e| RepositoryError::context("Failed to query sources", e))?
        {
            let mut a: src::ActiveModel = m.into();
            a.sync_status = ActiveValue::Set(status);
            a.version = ActiveValue::Set(version);
            let r = a
                .update(db)
                .await
                .map_err(|e| RepositoryError::context("Failed to update sources", e))?;
            return Ok(format!("Record updated with id: {:?}", r.id));
        }
        Err(RepositoryError::not_found(format!(
            "Record not found with id: {id}"
        )))
    }
    async fn purge_synced_deletions(&self) -> RepositoryResult<serde_json::Value> {
        use crate::database::entities::{
            attachment as att, error_question as eq, error_tag as tag, source as src,
            srs_data as srs, subject as sub,
        };
        let db = self.db.as_ref();
        let mut out = serde_json::Map::new();
        let n = eq::Entity::delete_many()
            .filter(eq::Column::SyncStatus.eq("synced"))
            .filter(eq::Column::DeletedAt.is_not_null())
            .exec(db)
            .await
            .map_err(|e| RepositoryError::context("Failed to purge error_questions", e))?
            .rows_affected;
        out.insert("error_questions".into(), serde_json::json!({"deleted":n}));
        let n = sub::Entity::delete_many()
            .filter(sub::Column::SyncStatus.eq("synced"))
            .filter(sub::Column::DeletedAt.is_not_null())
            .exec(db)
            .await
            .map_err(|e| RepositoryError::context("Failed to purge subjects", e))?
            .rows_affected;
        out.insert("subjects".into(), serde_json::json!({"deleted":n}));
        let n = att::Entity::delete_many()
            .filter(att::Column::SyncStatus.eq("synced"))
            .filter(att::Column::DeletedAt.is_not_null())
            .exec(db)
            .await
            .map_err(|e| RepositoryError::context("Failed to purge attachments", e))?
            .rows_affected;
        out.insert("attachments".into(), serde_json::json!({"deleted":n}));
        let n = tag::Entity::delete_many()
            .filter(tag::Column::SyncStatus.eq("synced"))
            .filter(tag::Column::DeletedAt.is_not_null())
            .exec(db)
            .await
            .map_err(|e| RepositoryError::context("Failed to purge error_tags", e))?
            .rows_affected;
        out.insert("error_tags".into(), serde_json::json!({"deleted":n}));
        let n = src::Entity::delete_many()
            .filter(src::Column::SyncStatus.eq("synced"))
            .filter(src::Column::DeletedAt.is_not_null())
            .exec(db)
            .await
            .map_err(|e| RepositoryError::context("Failed to purge sources", e))?
            .rows_affected;
        out.insert("sources".into(), serde_json::json!({"deleted":n}));
        let n = srs::Entity::delete_many()
            .filter(srs::Column::SyncStatus.eq("synced"))
            .filter(srs::Column::DeletedAt.is_not_null())
            .exec(db)
            .await
            .map_err(|e| RepositoryError::context("Failed to purge srs_data", e))?
            .rows_affected;
        out.insert("srs_data".into(), serde_json::json!({"deleted":n}));
        Ok(serde_json::Value::Object(out))
    }
    async fn check_orphans(&self, now: i64) -> RepositoryResult<CascadeOrphanCheckResult> {
        use crate::database::entities::{
            attachment as att, error_question as eq, error_tag as tag, source as src,
            srs_data as srs, subject as sub,
        };
        let db = self.db.as_ref();
        let mut removed = Vec::new();
        let mut checked = 0;
        let questions = eq::Entity::find()
            .filter(eq::Column::DeletedAt.is_null())
            .all(db)
            .await
            .map_err(|e| RepositoryError::context("Failed to query error_questions", e))?;
        checked += questions.len();
        for q in questions {
            if q.subjectid.is_empty() {
                continue;
            }
            let exists = sub::Entity::find_by_id(&q.subjectid)
                .filter(sub::Column::DeletedAt.is_null())
                .one(db)
                .await
                .map_err(|e| RepositoryError::context("Failed to query subject", e))?
                .is_some();
            if !exists {
                let mut a: eq::ActiveModel = q.into();
                a.subjectid = ActiveValue::Set(String::new());
                a.updated_at = ActiveValue::Set(now);
                a.update(db).await.map_err(|e| {
                    RepositoryError::context("Failed to soft delete error_question", e)
                })?;
            }
        }
        let sources = src::Entity::find()
            .filter(src::Column::DeletedAt.is_null())
            .all(db)
            .await
            .map_err(|e| RepositoryError::context("Failed to query sources", e))?;
        checked += sources.len();
        for m in sources {
            if let Some(parent) = &m.subject_id {
                if sub::Entity::find_by_id(parent)
                    .filter(sub::Column::DeletedAt.is_null())
                    .one(db)
                    .await
                    .map_err(|e| RepositoryError::context("Failed to query subject", e))?
                    .is_none()
                {
                    let id = m.id.clone();
                    let mut a: src::ActiveModel = m.into();
                    a.deleted_at = ActiveValue::Set(Some(now));
                    a.updated_at = ActiveValue::Set(now);
                    a.sync_status = ActiveValue::Set("pending".into());
                    a.update(db)
                        .await
                        .map_err(|e| RepositoryError::context("Failed to soft delete source", e))?;
                    removed.push(format!("source:{id}"));
                }
            }
        }
        let models = srs::Entity::find()
            .filter(srs::Column::DeletedAt.is_null())
            .all(db)
            .await
            .map_err(|e| RepositoryError::context("Failed to query srs_data", e))?;
        checked += models.len();
        for m in models {
            if eq::Entity::find_by_id(&m.question_id)
                .filter(eq::Column::DeletedAt.is_null())
                .one(db)
                .await
                .map_err(|e| RepositoryError::context("Failed to query error_question", e))?
                .is_none()
            {
                let id = m.id.clone();
                let mut a: srs::ActiveModel = m.into();
                a.deleted_at = ActiveValue::Set(Some(now));
                a.updated_at = ActiveValue::Set(now);
                a.sync_status = ActiveValue::Set("pending".into());
                a.update(db)
                    .await
                    .map_err(|e| RepositoryError::context("Failed to soft delete srs_data", e))?;
                removed.push(format!("srs_data:{id}"));
            }
        }
        let models = tag::Entity::find()
            .filter(tag::Column::DeletedAt.is_null())
            .all(db)
            .await
            .map_err(|e| RepositoryError::context("Failed to query error_tags", e))?;
        checked += models.len();
        for m in models {
            if eq::Entity::find_by_id(&m.question_id)
                .filter(eq::Column::DeletedAt.is_null())
                .one(db)
                .await
                .map_err(|e| RepositoryError::context("Failed to query error_question", e))?
                .is_none()
            {
                let id = m.id.clone();
                let mut a: tag::ActiveModel = m.into();
                a.deleted_at = ActiveValue::Set(Some(now));
                a.updated_at = ActiveValue::Set(now);
                a.sync_status = ActiveValue::Set("pending".into());
                a.update(db)
                    .await
                    .map_err(|e| RepositoryError::context("Failed to soft delete error_tag", e))?;
                removed.push(format!("error_tag:{id}"));
            }
        }
        let models = att::Entity::find()
            .filter(att::Column::DeletedAt.is_null())
            .all(db)
            .await
            .map_err(|e| RepositoryError::context("Failed to query attachments", e))?;
        checked += models.len();
        for m in models {
            if eq::Entity::find_by_id(&m.question_id)
                .filter(eq::Column::DeletedAt.is_null())
                .one(db)
                .await
                .map_err(|e| RepositoryError::context("Failed to query error_question", e))?
                .is_none()
            {
                let id = m.id.clone();
                let mut a: att::ActiveModel = m.into();
                a.deleted_at = ActiveValue::Set(Some(now));
                a.updated_at = ActiveValue::Set(now);
                a.sync_status = ActiveValue::Set("pending".into());
                a.update(db)
                    .await
                    .map_err(|e| RepositoryError::context("Failed to soft delete attachment", e))?;
                removed.push(format!("attachment:{id}"));
            }
        }
        Ok(CascadeOrphanCheckResult {
            orphan_records_soft_deleted: removed,
            total_checked: checked,
        })
    }
}
