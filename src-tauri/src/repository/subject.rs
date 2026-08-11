use std::sync::Arc;

use async_trait::async_trait;
use sea_orm::{ActiveModelTrait, ColumnTrait, DbConn, EntityTrait, QueryFilter, Set};

use crate::database::entities::{prelude::Subject, subject};
use crate::domain;
use crate::repository::{
    accept_uuid_insert_result, to_domain, to_domains, RepositoryError, RepositoryResult,
};

pub struct NewSubject {
    pub id: String,
    pub name: String,
    pub color: Option<String>,
    pub now: i64,
}

pub struct SubjectChanges {
    pub id: String,
    pub name: Option<String>,
    pub color: Option<String>,
    pub now: i64,
}

pub struct SyncedSubject {
    pub id: String,
    pub version: i32,
    pub deleted_at: Option<i64>,
    pub name: String,
    pub color: Option<String>,
    pub now: i64,
}

#[async_trait]
pub trait SubjectRepository: Send + Sync {
    async fn list_active(&self) -> RepositoryResult<Vec<domain::Subject>>;
    async fn create(&self, input: NewSubject) -> RepositoryResult<domain::Subject>;
    async fn update(&self, input: SubjectChanges) -> RepositoryResult<domain::Subject>;
    async fn soft_delete(&self, id: String, now: i64) -> RepositoryResult<()>;
    async fn upsert_synced(&self, input: SyncedSubject) -> RepositoryResult<()>;
}

pub struct SeaOrmSubjectRepository {
    db: Arc<DbConn>,
}

impl SeaOrmSubjectRepository {
    pub fn new(db: Arc<DbConn>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl SubjectRepository for SeaOrmSubjectRepository {
    async fn list_active(&self) -> RepositoryResult<Vec<domain::Subject>> {
        let models = Subject::find()
            .filter(subject::Column::DeletedAt.is_null())
            .all(self.db.as_ref())
            .await
            .map_err(RepositoryError::from)?;
        to_domains(models)
    }

    async fn create(&self, input: NewSubject) -> RepositoryResult<domain::Subject> {
        let id = input.id;
        let active: subject::ActiveModel = domain::Subject {
            id: id.clone(),
            name: input.name,
            color: input.color,
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

        let model = Subject::find_by_id(id)
            .one(self.db.as_ref())
            .await
            .map_err(RepositoryError::from)?
            .ok_or_else(|| RepositoryError::not_found("Subject not found"))?;
        to_domain(model)
    }

    async fn update(&self, input: SubjectChanges) -> RepositoryResult<domain::Subject> {
        let model = Subject::find_by_id(input.id)
            .one(self.db.as_ref())
            .await
            .map_err(RepositoryError::from)?
            .ok_or_else(|| RepositoryError::not_found("Subject not found"))?;
        let mut active: subject::ActiveModel = model.into();
        if let Some(name) = input.name {
            active.name = Set(name);
        }
        if let Some(color) = input.color {
            active.color = Set(Some(color));
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
        let model = Subject::find_by_id(id)
            .one(self.db.as_ref())
            .await
            .map_err(RepositoryError::from)?
            .ok_or_else(|| RepositoryError::not_found("Subject not found"))?;
        let mut active: subject::ActiveModel = model.into();
        active.deleted_at = Set(Some(now));
        active.updated_at = Set(now);
        active.sync_status = Set("pending".to_owned());
        active
            .update(self.db.as_ref())
            .await
            .map_err(RepositoryError::from)?;
        Ok(())
    }

    async fn upsert_synced(&self, input: SyncedSubject) -> RepositoryResult<()> {
        let existing = Subject::find_by_id(&input.id)
            .one(self.db.as_ref())
            .await
            .map_err(|e| RepositoryError::context("Query failed", e))?;
        let is_update = existing.is_some();
        let active: subject::ActiveModel = if let Some(model) = existing {
            let mut active: subject::ActiveModel = model.into();
            active.name = Set(input.name);
            active.color = Set(input.color);
            active.updated_at = Set(input.now);
            active.version = Set(input.version);
            active.sync_status = Set("synced".to_owned());
            active.deleted_at = Set(input.deleted_at);
            active
        } else {
            domain::Subject {
                id: input.id,
                name: input.name,
                color: input.color,
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

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use sea_orm::{ConnectOptions, ConnectionTrait, Database};

    use super::{NewSubject, SeaOrmSubjectRepository, SubjectChanges, SubjectRepository};
    use crate::repository::RepositoryError;

    #[tokio::test]
    async fn sea_orm_repository_supports_create_list_and_soft_delete() {
        let mut options = ConnectOptions::new("sqlite::memory:");
        options.max_connections(1).min_connections(1);
        let db = Database::connect(options).await.unwrap();
        crate::database::init_database(&db).await.unwrap();
        let repository = SeaOrmSubjectRepository::new(Arc::new(db.clone()));

        let created = repository
            .create(NewSubject {
                id: "repository-subject".to_string(),
                name: "Repository".to_string(),
                color: Some("blue".to_string()),
                now: 1,
            })
            .await
            .unwrap();
        assert_eq!(created.id, "repository-subject");
        assert_eq!(repository.list_active().await.unwrap().len(), 1);

        db.execute_unprepared(
            "CREATE TRIGGER reject_unrelated_subject_update \
             BEFORE UPDATE OF created_at, sync_hash ON subjects \
             BEGIN SELECT RAISE(FAIL, 'unrelated fields must not be updated'); END;",
        )
        .await
        .unwrap();

        let updated = repository
            .update(SubjectChanges {
                id: "repository-subject".to_string(),
                name: Some("Updated repository".to_string()),
                color: None,
                now: 2,
            })
            .await
            .unwrap();
        assert_eq!(updated.name, "Updated repository");

        repository
            .soft_delete("repository-subject".to_string(), 3)
            .await
            .unwrap();
        assert!(repository.list_active().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn create_reports_real_insert_failures_as_database_errors() {
        let mut options = ConnectOptions::new("sqlite::memory:");
        options.max_connections(1).min_connections(1);
        let db = Database::connect(options).await.unwrap();
        crate::database::init_database(&db).await.unwrap();
        db.execute_unprepared(
            "CREATE TRIGGER reject_subject_insert \
             BEFORE INSERT ON subjects \
             BEGIN SELECT RAISE(FAIL, 'forced insert failure'); END;",
        )
        .await
        .unwrap();
        let repository = SeaOrmSubjectRepository::new(Arc::new(db));

        let error = repository
            .create(NewSubject {
                id: "rejected-subject".to_string(),
                name: "Rejected".to_string(),
                color: None,
                now: 1,
            })
            .await
            .unwrap_err();

        assert!(matches!(error, RepositoryError::Database(_)));
        assert!(error.to_string().contains("forced insert failure"));
    }
}
