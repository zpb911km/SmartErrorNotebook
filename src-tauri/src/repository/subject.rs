use std::sync::Arc;

use async_trait::async_trait;
use sea_orm::{ActiveModelTrait, ColumnTrait, DbConn, EntityTrait, QueryFilter, Set};

use crate::database::entities::{prelude::Subject, subject};
use crate::repository::{accept_uuid_insert_result, RepositoryError, RepositoryResult};

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
    async fn list_active(&self) -> RepositoryResult<Vec<subject::Model>>;
    async fn create(&self, input: NewSubject) -> RepositoryResult<subject::Model>;
    async fn update(&self, input: SubjectChanges) -> RepositoryResult<subject::Model>;
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
    async fn list_active(&self) -> RepositoryResult<Vec<subject::Model>> {
        Subject::find()
            .filter(subject::Column::DeletedAt.is_null())
            .all(self.db.as_ref())
            .await
            .map_err(RepositoryError::from)
    }

    async fn create(&self, input: NewSubject) -> RepositoryResult<subject::Model> {
        let insert_result = subject::ActiveModel {
            id: Set(input.id.clone()),
            name: Set(input.name),
            color: Set(input.color),
            created_at: Set(input.now),
            updated_at: Set(input.now),
            deleted_at: Set(None),
            version: Set(0),
            sync_status: Set("pending".to_string()),
            sync_hash: Set(None),
        }
        .insert(self.db.as_ref())
        .await;
        accept_uuid_insert_result(insert_result)?;

        Subject::find_by_id(input.id)
            .one(self.db.as_ref())
            .await
            .map_err(RepositoryError::from)?
            .ok_or_else(|| RepositoryError::not_found("Subject not found"))
    }

    async fn update(&self, input: SubjectChanges) -> RepositoryResult<subject::Model> {
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
        active.sync_status = Set("pending".to_string());
        active
            .update(self.db.as_ref())
            .await
            .map_err(RepositoryError::from)
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
        active.sync_status = Set("pending".to_string());
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
        let active = if let Some(model) = existing {
            let mut active: subject::ActiveModel = model.into();
            active.name = Set(input.name);
            active.color = Set(input.color);
            active.updated_at = Set(input.now);
            active.version = Set(input.version);
            active.sync_status = Set("synced".to_string());
            active.deleted_at = Set(input.deleted_at);
            active
        } else {
            subject::ActiveModel {
                id: Set(input.id),
                name: Set(input.name),
                color: Set(input.color),
                created_at: Set(input.now),
                updated_at: Set(input.now),
                deleted_at: Set(input.deleted_at),
                version: Set(input.version),
                sync_status: Set("synced".to_string()),
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

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use sea_orm::{ConnectOptions, ConnectionTrait, Database};

    use super::{NewSubject, SeaOrmSubjectRepository, SubjectRepository};
    use crate::repository::RepositoryError;

    #[tokio::test]
    async fn sea_orm_repository_supports_create_list_and_soft_delete() {
        let mut options = ConnectOptions::new("sqlite::memory:");
        options.max_connections(1).min_connections(1);
        let db = Database::connect(options).await.unwrap();
        crate::database::init_database(&db).await.unwrap();
        let repository = SeaOrmSubjectRepository::new(Arc::new(db));

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

        repository
            .soft_delete("repository-subject".to_string(), 2)
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
