use std::collections::HashMap;

use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, FromQueryResult, PaginatorTrait,
    QueryFilter, QuerySelect, Set,
};
use uuid::Uuid;

use crate::domain::model::Source;
use crate::domain::repository::{
    error::{
        CorruptedData, EntityReference, MissingReference, Referenced, RepositoryDeleteError,
        RepositoryFindError, RepositoryInfrastructureError, RepositorySaveError,
    },
    legacy, SourceRepository,
};

use super::super::database::entity::{question, source, subject};
use super::{timestamp, uuid};

pub struct SeaOrmSourceRepository<'c, C: ConnectionTrait> {
    connection: &'c C,
}
impl<'c, C: ConnectionTrait> SeaOrmSourceRepository<'c, C> {
    pub fn new(connection: &'c C) -> Self {
        Self { connection }
    }
}

#[async_trait::async_trait]
impl<'c, C: ConnectionTrait> SourceRepository for SeaOrmSourceRepository<'c, C> {
    async fn save(&self, source: &Source) -> Result<(), RepositorySaveError> {
        if source.metadata.deleted_at.is_some()
            && question::Entity::find()
                .filter(question::Column::SourceId.eq(source.id))
                .filter(question::Column::DeletedAt.is_null())
                .count(self.connection)
                .await
                .map_err(|error| {
                    RepositoryInfrastructureError::new("count active source references", error)
                })?
                != 0
        {
            return Err(Referenced {
                target: EntityReference {
                    entity: "source",
                    id: source.id,
                },
                referenced_by: "question",
            }
            .into());
        }
        if let Some(subject_id) = source.subject_id {
            let mut subject_query = subject::Entity::find_by_id(subject_id);
            if source.metadata.deleted_at.is_none() {
                subject_query = subject_query.filter(subject::Column::DeletedAt.is_null());
            }
            if subject_query
                .one(self.connection)
                .await
                .map_err(|error| RepositoryInfrastructureError::new("query source subject", error))?
                .is_none()
            {
                return Err(MissingReference {
                    owner: EntityReference {
                        entity: "source",
                        id: source.id,
                    },
                    missing: vec![EntityReference {
                        entity: "subject",
                        id: subject_id,
                    }],
                }
                .into());
            }
        }
        let existing_entity = source::Entity::find_by_id(source.id)
            .one(self.connection)
            .await
            .map_err(|error| RepositoryInfrastructureError::new("query source", error))?;
        let is_existing = existing_entity.is_some();
        let mut active_model = source::ActiveModel {
            id: Set(source.id),
            created_at: Set(source.metadata.created_at),
            updated_at: Set(source.metadata.updated_at),
            deleted_at: Set(source.metadata.deleted_at),
            sync_status: Set(source.metadata.sync_status.clone().into()),
            sync_version: Set(source.metadata.sync_version),
            subject_id: Set(source.subject_id),
            book: Set(source.book.clone()),
            chapter: Set(source.chapter.clone()),
            knowledge: Set(source.knowledge.clone()),
        };
        if is_existing {
            active_model.id = sea_orm::ActiveValue::Unchanged(source.id);
            active_model
                .update(self.connection)
                .await
                .map_err(|error| RepositoryInfrastructureError::new("update source", error))?;
        } else {
            active_model
                .insert(self.connection)
                .await
                .map_err(|error| RepositoryInfrastructureError::new("insert source", error))?;
        }
        Ok(())
    }

    async fn delete_by_id(&self, id: &Uuid) -> Result<(), RepositoryDeleteError> {
        if question::Entity::find()
            .filter(question::Column::SourceId.eq(*id))
            .count(self.connection)
            .await
            .map_err(|error| RepositoryInfrastructureError::new("count source references", error))?
            != 0
        {
            return Err(Referenced {
                target: EntityReference {
                    entity: "source",
                    id: *id,
                },
                referenced_by: "question",
            }
            .into());
        }
        source::Entity::delete_by_id(*id)
            .exec(self.connection)
            .await
            .map_err(|error| RepositoryInfrastructureError::new("delete source", error))?;
        Ok(())
    }

    async fn find_all(&self, include_deleted: bool) -> Result<Vec<Source>, RepositoryFindError> {
        let mut query = source::Entity::find();
        if !include_deleted {
            query = query.filter(source::Column::DeletedAt.is_null());
        }
        query
            .all(self.connection)
            .await
            .map_err(|error| {
                RepositoryFindError::from(RepositoryInfrastructureError::new("list sources", error))
            })?
            .into_iter()
            .map(|model| {
                let id = model.id;
                Source::try_from(model)
                    .map_err(|error| CorruptedData::new("source", id, error).into())
            })
            .collect()
    }

    async fn find_by_id(
        &self,
        id: &Uuid,
        include_deleted: bool,
    ) -> Result<Option<Source>, RepositoryFindError> {
        let mut query = source::Entity::find_by_id(*id);
        if !include_deleted {
            query = query.filter(source::Column::DeletedAt.is_null());
        }
        query
            .one(self.connection)
            .await
            .map_err(|error| RepositoryInfrastructureError::new("query source", error))?
            .map(|model| {
                let id = model.id;
                Source::try_from(model)
                    .map_err(|error| CorruptedData::new("source", id, error).into())
            })
            .transpose()
    }

    async fn find_by_subject_id(
        &self,
        subject_id: &Uuid,
        include_deleted: bool,
    ) -> Result<Vec<Source>, RepositoryFindError> {
        let mut query = source::Entity::find().filter(source::Column::SubjectId.eq(*subject_id));
        if !include_deleted {
            query = query.filter(source::Column::DeletedAt.is_null());
        }
        query
            .all(self.connection)
            .await
            .map_err(|error| RepositoryInfrastructureError::new("query sources", error))?
            .into_iter()
            .map(|model| {
                let id = model.id;
                Source::try_from(model)
                    .map_err(|error| CorruptedData::new("source", id, error).into())
            })
            .collect()
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
            .map(|model| Source::try_from(model).expect("persisted legacy source must be mappable"))
            .collect();
        self.with_context(sources).await
    }

    async fn find_by_id(&self, id: String) -> legacy::repository_model::source::SourceWithContext {
        let source = source::Entity::find_by_id(uuid(&id, "source id"))
            .one(self.connection)
            .await
            .expect("failed to query source")
            .expect("Source not found")
            .try_into()
            .expect("persisted legacy source must be mappable");
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
        .try_into()
        .expect("created legacy source must be mappable")
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
            .try_into()
            .expect("updated legacy source must be mappable")
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
