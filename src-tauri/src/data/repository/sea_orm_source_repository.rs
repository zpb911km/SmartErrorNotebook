use super::super::database::entity::{question, source};
use super::{timestamp, uuid};
use crate::model::Source;
use crate::repository::legacy;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, FromQueryResult, QueryFilter,
    QuerySelect, Set,
};
use std::collections::HashMap;

pub struct SeaOrmSourceRepository<'c, C: ConnectionTrait> {
    connection: &'c C,
}
impl<'c, C: ConnectionTrait> SeaOrmSourceRepository<'c, C> {
    pub fn new(connection: &'c C) -> Self {
        Self {
            connection: connection,
        }
    }
}

#[derive(FromQueryResult)]
struct TextRow {
    value: String,
}
impl<'c, C: ConnectionTrait> SeaOrmSourceRepository<'c, C> {
    async fn with_context(
        &self,
        sources: Vec<Source>,
    ) -> Vec<legacy::repository_model::source::SourceWithContext> {
        if sources.is_empty() {
            return Vec::new();
        }

        let source_ids: Vec<_> = sources.iter().map(|source| source.id).collect();
        let query = question::Entity::find()
            .filter(question::Column::DeletedAt.is_null())
            .filter(question::Column::SourceId.is_in(source_ids));
        let mut relationships: HashMap<uuid::Uuid, Vec<uuid::Uuid>> = HashMap::new();
        for question in query
            .all(self.connection)
            .await
            .expect("failed to query source relationships")
        {
            if let Some(source_id) = question.source_id {
                relationships
                    .entry(source_id)
                    .or_default()
                    .push(question.id);
            }
        }

        sources
            .into_iter()
            .map(|source| {
                let links = relationships.remove(&source.id).unwrap_or_default();
                let question_id = (links.len() == 1).then(|| links[0].to_string());
                legacy::repository_model::source::SourceWithContext {
                    source,
                    question_id,
                }
            })
            .collect()
    }

    async fn values(
        &self,
        column: source::Column,
        subject_id: Option<String>,
        book: Option<String>,
        chapter: Option<String>,
    ) -> Vec<String> {
        let mut query = source::Entity::find()
            .filter(source::Column::DeletedAt.is_null())
            .filter(column.is_not_null());
        if let Some(value) = book {
            query = query.filter(source::Column::Book.eq(value));
        }
        if let Some(value) = chapter {
            query = query.filter(source::Column::Chapter.eq(value));
        }
        if let Some(id) = subject_id {
            query = query.filter(source::Column::SubjectId.eq(uuid(&id, "subject id")));
        }
        query
            .select_only()
            .column_as(column, "value")
            .distinct()
            .into_model::<TextRow>()
            .all(self.connection)
            .await
            .expect("failed to query source values")
            .into_iter()
            .map(|row| row.value)
            .collect()
    }
}

#[async_trait::async_trait]
impl<'c, C: ConnectionTrait> legacy::SourceRepository for SeaOrmSourceRepository<'c, C> {
    async fn list_active(
        &self,
        subject_id: Option<String>,
    ) -> Vec<legacy::repository_model::source::SourceWithContext> {
        let subject_id = subject_id.map(|id| uuid(&id, "subject id"));
        let mut query = source::Entity::find().filter(source::Column::DeletedAt.is_null());
        if let Some(subject_id) = subject_id {
            query = query.filter(source::Column::SubjectId.eq(subject_id));
        }
        let sources = query
            .all(self.connection)
            .await
            .expect("failed to list sources")
            .into_iter()
            .map(Into::into)
            .collect();
        self.with_context(sources).await
    }

    async fn find_by_id(&self, id: String) -> legacy::repository_model::source::SourceWithContext {
        let source = source::Entity::find_by_id(uuid(&id, "source id"))
            .one(self.connection)
            .await
            .expect("failed to query source")
            .expect("Source not found")
            .into();
        self.with_context(vec![source]).await.remove(0)
    }

    async fn list_books(&self, subject_id: Option<String>) -> Vec<String> {
        self.values(source::Column::Book, subject_id, None, None)
            .await
    }

    async fn list_chapters(&self, subject_id: Option<String>, book: String) -> Vec<String> {
        self.values(source::Column::Chapter, subject_id, Some(book), None)
            .await
    }

    async fn list_knowledges(
        &self,
        subject_id: Option<String>,
        book: String,
        chapter: String,
    ) -> Vec<String> {
        self.values(
            source::Column::Knowledge,
            subject_id,
            Some(book),
            Some(chapter),
        )
        .await
    }

    async fn find_active_exact(
        &self,
        values: &legacy::repository_model::source::SourceValues,
    ) -> Option<Source> {
        self.list_active(values.subject_id.clone())
            .await
            .into_iter()
            .map(|record| record.source)
            .find(|source| {
                source.book == values.book
                    && source.chapter == values.chapter
                    && source.knowledge == values.knowledge
            })
    }

    async fn create(&self, input: legacy::repository_model::source::NewSource) -> Source {
        let now = timestamp(input.now, "source timestamp");
        source::ActiveModel {
            id: Set(uuid(&input.id, "source id")),
            created_at: Set(now),
            updated_at: Set(now),
            deleted_at: Set(None),
            sync_status: Set("PENDING".into()),
            sync_version: Set(0),
            subject_id: Set(input
                .values
                .subject_id
                .map(|id| uuid(&id, "source subject id"))),
            book: Set(input.values.book),
            chapter: Set(input.values.chapter),
            knowledge: Set(input.values.knowledge),
        }
        .insert(self.connection)
        .await
        .expect("failed to create source")
        .into()
    }

    async fn update(&self, input: legacy::repository_model::source::SourceChanges) -> Source {
        let model = source::Entity::find_by_id(uuid(&input.id, "source id"))
            .one(self.connection)
            .await
            .expect("failed to query source")
            .expect("Source not found");
        let mut active: source::ActiveModel = model.into();
        if input.values.subject_id.is_some() {
            active.subject_id = Set(input
                .values
                .subject_id
                .map(|id| uuid(&id, "source subject id")));
        }
        if input.values.book.is_some() {
            active.book = Set(input.values.book);
        }
        if input.values.chapter.is_some() {
            active.chapter = Set(input.values.chapter);
        }
        if input.values.knowledge.is_some() {
            active.knowledge = Set(input.values.knowledge);
        }
        active.updated_at = Set(timestamp(input.now, "source timestamp"));
        active.sync_status = Set("PENDING".into());
        active
            .update(self.connection)
            .await
            .expect("failed to update source")
            .into()
    }

    async fn soft_delete(&self, id: String, now: i64) {
        let model = source::Entity::find_by_id(uuid(&id, "source id"))
            .one(self.connection)
            .await
            .expect("failed to query source")
            .expect("Source not found");
        let mut active: source::ActiveModel = model.into();
        let now = timestamp(now, "source timestamp");
        active.deleted_at = Set(Some(now));
        active.updated_at = Set(now);
        active.sync_status = Set("PENDING".into());
        active
            .update(self.connection)
            .await
            .expect("failed to soft-delete source");
    }

    async fn upsert_synced(&self, input: legacy::repository_model::source::SyncedSource) {
        let id = uuid(&input.id, "source id");
        let now = timestamp(input.now, "source timestamp");
        let exists = source::Entity::find_by_id(id)
            .one(self.connection)
            .await
            .expect("failed to query source")
            .is_some();
        let mut active = source::ActiveModel {
            id: Set(id),
            created_at: Set(now),
            updated_at: Set(now),
            deleted_at: Set(input
                .deleted_at
                .map(|value| timestamp(value, "source deleted_at"))),
            sync_status: Set("SYNCED".into()),
            sync_version: Set(i64::from(input.version)),
            subject_id: Set(input
                .values
                .subject_id
                .map(|id| uuid(&id, "source subject id"))),
            book: Set(input.values.book),
            chapter: Set(input.values.chapter),
            knowledge: Set(input.values.knowledge),
        };
        if exists {
            active.id = sea_orm::ActiveValue::Unchanged(id);
            active
                .update(self.connection)
                .await
                .expect("failed to update synced source");
        } else {
            active
                .insert(self.connection)
                .await
                .expect("failed to insert synced source");
        }
        if let Some(question_id) = input.question_id {
            if let Some(model) = question::Entity::find_by_id(uuid(&question_id, "question id"))
                .one(self.connection)
                .await
                .expect("failed to query source owner")
            {
                let mut question: question::ActiveModel = model.into();
                question.source_id = Set(Some(id));
                question
                    .update(self.connection)
                    .await
                    .expect("failed to relate source");
            }
        }
    }
}
