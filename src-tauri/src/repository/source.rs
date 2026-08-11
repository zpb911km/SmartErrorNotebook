use crate::database::entities::{prelude::Source, source};
use crate::domain;
use crate::repository::{
    accept_uuid_insert_result, to_domain, to_domains, RepositoryError, RepositoryResult,
};
use async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbConn, EntityTrait, FromQueryResult, QueryFilter, QuerySelect,
    Set,
};
use std::sync::Arc;

#[derive(FromQueryResult)]
struct BookRow {
    book: String,
}
#[derive(FromQueryResult)]
struct ChapterRow {
    chapter: String,
}
#[derive(FromQueryResult)]
struct KnowledgeRow {
    knowledge: String,
}

#[derive(Clone)]
pub struct SourceValues {
    pub subject_id: Option<String>,
    pub book: Option<String>,
    pub chapter: Option<String>,
    pub knowledge: Option<String>,
}
pub struct NewSource {
    pub id: String,
    pub values: SourceValues,
    pub now: i64,
}
pub struct SourceChanges {
    pub id: String,
    pub values: SourceValues,
    pub now: i64,
}
pub struct SyncedSource {
    pub id: String,
    pub version: i32,
    pub deleted_at: Option<i64>,
    pub question_id: Option<String>,
    pub values: SourceValues,
    pub now: i64,
}

#[async_trait]
pub trait SourceRepository: Send + Sync {
    async fn list_active(
        &self,
        subject_id: Option<String>,
    ) -> RepositoryResult<Vec<domain::Source>>;
    async fn find_by_id(&self, id: String) -> RepositoryResult<domain::Source>;
    async fn list_books(&self, subject_id: Option<String>) -> RepositoryResult<Vec<String>>;
    async fn list_chapters(
        &self,
        subject_id: Option<String>,
        book: String,
    ) -> RepositoryResult<Vec<String>>;
    async fn list_knowledges(
        &self,
        subject_id: Option<String>,
        book: String,
        chapter: String,
    ) -> RepositoryResult<Vec<String>>;
    async fn find_active_exact(
        &self,
        values: &SourceValues,
    ) -> RepositoryResult<Option<domain::Source>>;
    async fn create(&self, input: NewSource) -> RepositoryResult<domain::Source>;
    async fn update(&self, input: SourceChanges) -> RepositoryResult<domain::Source>;
    async fn soft_delete(&self, id: String, now: i64) -> RepositoryResult<()>;
    async fn upsert_synced(&self, input: SyncedSource) -> RepositoryResult<()>;
}

pub struct SeaOrmSourceRepository {
    db: Arc<DbConn>,
}
impl SeaOrmSourceRepository {
    pub fn new(db: Arc<DbConn>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl SourceRepository for SeaOrmSourceRepository {
    async fn list_active(
        &self,
        subject_id: Option<String>,
    ) -> RepositoryResult<Vec<domain::Source>> {
        let mut query = Source::find().filter(source::Column::DeletedAt.is_null());
        if let Some(id) = subject_id {
            query = query.filter(source::Column::SubjectId.eq(id));
        }
        let models = query
            .all(self.db.as_ref())
            .await
            .map_err(RepositoryError::from)?;
        to_domains(models)
    }
    async fn find_by_id(&self, id: String) -> RepositoryResult<domain::Source> {
        let model = Source::find_by_id(id)
            .one(self.db.as_ref())
            .await
            .map_err(RepositoryError::from)?
            .ok_or_else(|| RepositoryError::not_found("Source not found"))?;
        to_domain(model)
    }
    async fn list_books(&self, subject_id: Option<String>) -> RepositoryResult<Vec<String>> {
        let mut query = Source::find()
            .filter(source::Column::DeletedAt.is_null())
            .filter(source::Column::Book.is_not_null())
            .select_only()
            .column(source::Column::Book)
            .distinct();
        if let Some(id) = subject_id {
            query = query.filter(source::Column::SubjectId.eq(id));
        }
        Ok(query
            .into_model::<BookRow>()
            .all(self.db.as_ref())
            .await
            .map_err(RepositoryError::from)?
            .into_iter()
            .map(|r| r.book)
            .collect())
    }
    async fn list_chapters(
        &self,
        subject_id: Option<String>,
        book: String,
    ) -> RepositoryResult<Vec<String>> {
        let mut query = Source::find()
            .filter(source::Column::DeletedAt.is_null())
            .filter(source::Column::Book.eq(book))
            .filter(source::Column::Chapter.is_not_null())
            .select_only()
            .column(source::Column::Chapter)
            .distinct();
        if let Some(id) = subject_id {
            query = query.filter(source::Column::SubjectId.eq(id));
        }
        Ok(query
            .into_model::<ChapterRow>()
            .all(self.db.as_ref())
            .await
            .map_err(RepositoryError::from)?
            .into_iter()
            .map(|r| r.chapter)
            .collect())
    }
    async fn list_knowledges(
        &self,
        subject_id: Option<String>,
        book: String,
        chapter: String,
    ) -> RepositoryResult<Vec<String>> {
        let mut query = Source::find()
            .filter(source::Column::DeletedAt.is_null())
            .filter(source::Column::Book.eq(book))
            .filter(source::Column::Chapter.eq(chapter))
            .filter(source::Column::Knowledge.is_not_null())
            .select_only()
            .column(source::Column::Knowledge)
            .distinct();
        if let Some(id) = subject_id {
            query = query.filter(source::Column::SubjectId.eq(id));
        }
        Ok(query
            .into_model::<KnowledgeRow>()
            .all(self.db.as_ref())
            .await
            .map_err(RepositoryError::from)?
            .into_iter()
            .map(|r| r.knowledge)
            .collect())
    }
    async fn find_active_exact(
        &self,
        values: &SourceValues,
    ) -> RepositoryResult<Option<domain::Source>> {
        let mut query = Source::find().filter(source::Column::DeletedAt.is_null());
        query = match &values.subject_id {
            Some(v) => query.filter(source::Column::SubjectId.eq(v)),
            None => query.filter(source::Column::SubjectId.is_null()),
        };
        query = match &values.book {
            Some(v) => query.filter(source::Column::Book.eq(v)),
            None => query.filter(source::Column::Book.is_null()),
        };
        query = match &values.chapter {
            Some(v) => query.filter(source::Column::Chapter.eq(v)),
            None => query.filter(source::Column::Chapter.is_null()),
        };
        query = match &values.knowledge {
            Some(v) => query.filter(source::Column::Knowledge.eq(v)),
            None => query.filter(source::Column::Knowledge.is_null()),
        };
        query
            .one(self.db.as_ref())
            .await
            .map_err(RepositoryError::from)?
            .map(to_domain)
            .transpose()
    }
    async fn create(&self, input: NewSource) -> RepositoryResult<domain::Source> {
        let id = input.id;
        let active: source::ActiveModel = domain::Source {
            id: id.clone(),
            question_id: None,
            subject_id: input.values.subject_id,
            book: input.values.book,
            chapter: input.values.chapter,
            knowledge: input.values.knowledge,
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
        let model = Source::find_by_id(id)
            .one(self.db.as_ref())
            .await
            .map_err(RepositoryError::from)?
            .ok_or_else(|| RepositoryError::not_found("插入后未能找到新创建的记录"))?;
        to_domain(model)
    }
    async fn update(&self, input: SourceChanges) -> RepositoryResult<domain::Source> {
        let model = Source::find_by_id(input.id)
            .one(self.db.as_ref())
            .await
            .map_err(RepositoryError::from)?
            .ok_or_else(|| RepositoryError::not_found("Source not found"))?;
        let mut active: source::ActiveModel = model.into();
        if let Some(v) = input.values.subject_id {
            active.subject_id = Set(Some(v));
        }
        if let Some(v) = input.values.book {
            active.book = Set(Some(v));
        }
        if let Some(v) = input.values.chapter {
            active.chapter = Set(Some(v));
        }
        if let Some(v) = input.values.knowledge {
            active.knowledge = Set(Some(v));
        }
        active.updated_at = Set(input.now);
        active.sync_status = Set("pending".to_owned());
        let model = active
            .update(self.db.as_ref())
            .await
            .map_err(RepositoryError::from)?;
        to_domain(model)
    }
    async fn soft_delete(&self, id: String, now: i64) -> RepositoryResult<()> {
        let model = Source::find_by_id(id)
            .one(self.db.as_ref())
            .await
            .map_err(RepositoryError::from)?
            .ok_or_else(|| RepositoryError::not_found("Source not found"))?;
        let mut active: source::ActiveModel = model.into();
        active.deleted_at = Set(Some(now));
        active.updated_at = Set(now);
        active.sync_status = Set("pending".to_owned());
        active
            .update(self.db.as_ref())
            .await
            .map_err(RepositoryError::from)?;
        Ok(())
    }
    async fn upsert_synced(&self, input: SyncedSource) -> RepositoryResult<()> {
        let existing = Source::find_by_id(&input.id)
            .one(self.db.as_ref())
            .await
            .map_err(|e| RepositoryError::context("Query failed", e))?;
        let is_update = existing.is_some();
        let active: source::ActiveModel = if let Some(model) = existing {
            let mut active: source::ActiveModel = model.into();
            active.question_id = Set(input.question_id);
            active.subject_id = Set(input.values.subject_id);
            active.book = Set(input.values.book);
            active.chapter = Set(input.values.chapter);
            active.knowledge = Set(input.values.knowledge);
            active.updated_at = Set(input.now);
            active.version = Set(input.version);
            active.sync_status = Set("synced".to_owned());
            active.deleted_at = Set(input.deleted_at);
            active
        } else {
            domain::Source {
                id: input.id,
                question_id: input.question_id,
                subject_id: input.values.subject_id,
                book: input.values.book,
                chapter: input.values.chapter,
                knowledge: input.values.knowledge,
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
