use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, PaginatorTrait, QueryFilter, Set,
};
use uuid::Uuid;

use crate::domain::model::Subject;
use crate::domain::repository::{
    error::{
        CorruptedData, EntityReference, Referenced, RepositoryDeleteError, RepositoryFindError,
        RepositoryInfrastructureError, RepositorySaveError,
    },
    legacy, SubjectRepository,
};

use super::super::database::entity::{source, subject};
use super::{timestamp, uuid};

pub struct SeaOrmSubjectRepository<'c, C: ConnectionTrait> {
    connection: &'c C,
}
impl<'c, C: ConnectionTrait> SeaOrmSubjectRepository<'c, C> {
    pub fn new(connection: &'c C) -> Self {
        Self { connection }
    }
}

#[async_trait::async_trait]
impl<'c, C: ConnectionTrait> SubjectRepository for SeaOrmSubjectRepository<'c, C> {
    async fn save(&self, subject: &Subject) -> Result<(), RepositorySaveError> {
        if subject.metadata.deleted_at.is_some()
            && source::Entity::find()
                .filter(source::Column::SubjectId.eq(subject.id))
                .filter(source::Column::DeletedAt.is_null())
                .count(self.connection)
                .await
                .map_err(|error| {
                    RepositoryInfrastructureError::new("count active subject references", error)
                })?
                != 0
        {
            return Err(Referenced {
                target: EntityReference {
                    entity: "subject",
                    id: subject.id,
                },
                referenced_by: "source",
            }
            .into());
        }
        let is_existing = subject::Entity::find_by_id(subject.id)
            .one(self.connection)
            .await
            .map_err(|error| RepositoryInfrastructureError::new("query subject", error))?
            .is_some();
        let mut active_model = subject::ActiveModel {
            id: Set(subject.id),
            created_at: Set(subject.metadata.created_at),
            updated_at: Set(subject.metadata.updated_at),
            deleted_at: Set(subject.metadata.deleted_at),
            sync_status: Set(subject.metadata.sync_status.clone().into()),
            sync_version: Set(subject.metadata.sync_version),
            name: Set(subject.name.clone()),
            color: Set(subject.color.clone()),
        };
        if is_existing {
            active_model.id = sea_orm::ActiveValue::Unchanged(subject.id);
            active_model
                .update(self.connection)
                .await
                .map_err(|error| RepositoryInfrastructureError::new("update subject", error))?;
        } else {
            active_model
                .insert(self.connection)
                .await
                .map_err(|error| RepositoryInfrastructureError::new("insert subject", error))?;
        }
        Ok(())
    }

    async fn delete_by_id(&self, id: &Uuid) -> Result<(), RepositoryDeleteError> {
        if source::Entity::find()
            .filter(source::Column::SubjectId.eq(*id))
            .count(self.connection)
            .await
            .map_err(|error| {
                RepositoryInfrastructureError::new("count subject references", error)
            })?
            != 0
        {
            return Err(Referenced {
                target: EntityReference {
                    entity: "subject",
                    id: *id,
                },
                referenced_by: "source",
            }
            .into());
        }
        subject::Entity::delete_by_id(*id)
            .exec(self.connection)
            .await
            .map_err(|error| RepositoryInfrastructureError::new("delete subject", error))?;
        Ok(())
    }

    async fn find_all(&self, include_deleted: bool) -> Result<Vec<Subject>, RepositoryFindError> {
        let mut query = subject::Entity::find();
        if !include_deleted {
            query = query.filter(subject::Column::DeletedAt.is_null());
        }
        query
            .all(self.connection)
            .await
            .map_err(|error| RepositoryInfrastructureError::new("list subjects", error))?
            .into_iter()
            .map(|model| {
                let id = model.id;
                Subject::try_from(model)
                    .map_err(|error| CorruptedData::new("subject", id, error).into())
            })
            .collect()
    }

    async fn find_by_id(&self, id: &Uuid) -> Result<Option<Subject>, RepositoryFindError> {
        subject::Entity::find_by_id(*id)
            .one(self.connection)
            .await
            .map_err(|error| RepositoryInfrastructureError::new("query subject", error))?
            .map(|model| {
                let id = model.id;
                Subject::try_from(model)
                    .map_err(|error| CorruptedData::new("subject", id, error).into())
            })
            .transpose()
    }
}

#[async_trait::async_trait]
impl<'c, C: ConnectionTrait> legacy::SubjectRepository for SeaOrmSubjectRepository<'c, C> {
    async fn list_active(&self) -> Vec<Subject> {
        subject::Entity::find()
            .filter(subject::Column::DeletedAt.is_null())
            .all(self.connection)
            .await
            .expect("failed to list subjects")
            .into_iter()
            .map(|model| {
                Subject::try_from(model).expect("persisted legacy subject must be mappable")
            })
            .collect()
    }

    async fn create(&self, input: legacy::repository_model::subject::NewSubject) -> Subject {
        let now = timestamp(input.now, "subject timestamp");
        subject::ActiveModel {
            id: Set(uuid(&input.id, "subject id")),
            created_at: Set(now),
            updated_at: Set(now),
            deleted_at: Set(None),
            sync_status: Set("PENDING".into()),
            sync_version: Set(0),
            name: Set(input.name),
            color: Set(input.color.unwrap_or_default()),
        }
        .insert(self.connection)
        .await
        .expect("failed to create subject")
        .try_into()
        .expect("created legacy subject must be mappable")
    }

    async fn update(
        &self,
        input: legacy::repository_model::subject::SubjectChanges,
    ) -> Result<Subject, String> {
        let model = subject::Entity::find_by_id(uuid(&input.id, "subject id"))
            .one(self.connection)
            .await
            .map_err(|error| format!("failed to query subject: {error}"))?
            .ok_or_else(|| "Subject not found".to_owned())?;
        let mut active: subject::ActiveModel = model.into();
        if let Some(value) = input.name {
            active.name = Set(value);
        }
        if let Some(value) = input.color {
            active.color = Set(value);
        }
        active.updated_at = Set(timestamp(input.now, "subject timestamp"));
        active.sync_status = Set("PENDING".into());
        active
            .update(self.connection)
            .await
            .map_err(|error| format!("failed to update subject: {error}"))?
            .try_into()
            .map_err(|error| format!("failed to map subject: {error}"))
    }

    async fn soft_delete(&self, id: String, now: i64) -> Result<(), String> {
        let model = subject::Entity::find_by_id(uuid(&id, "subject id"))
            .one(self.connection)
            .await
            .map_err(|error| format!("failed to query subject: {error}"))?
            .ok_or_else(|| "Subject not found".to_owned())?;
        let mut active: subject::ActiveModel = model.into();
        let now = timestamp(now, "subject timestamp");
        active.deleted_at = Set(Some(now));
        active.updated_at = Set(now);
        active.sync_status = Set("PENDING".into());
        active
            .update(self.connection)
            .await
            .map_err(|error| format!("failed to soft-delete subject: {error}"))?;
        Ok(())
    }

    async fn upsert_synced(&self, input: legacy::repository_model::subject::SyncedSubject) {
        let id = uuid(&input.id, "subject id");
        let now = timestamp(input.now, "subject timestamp");
        let exists = subject::Entity::find_by_id(id)
            .one(self.connection)
            .await
            .expect("failed to query subject")
            .is_some();
        let mut active = subject::ActiveModel {
            id: Set(id),
            created_at: Set(now),
            updated_at: Set(now),
            deleted_at: Set(input
                .deleted_at
                .map(|value| timestamp(value, "subject deleted_at"))),
            sync_status: Set("SYNCED".into()),
            sync_version: Set(i64::from(input.version)),
            name: Set(input.name),
            color: Set(input.color.unwrap_or_default()),
        };
        if exists {
            active.id = sea_orm::ActiveValue::Unchanged(id);
            active
                .update(self.connection)
                .await
                .expect("failed to update synced subject");
        } else {
            active
                .insert(self.connection)
                .await
                .expect("failed to insert synced subject");
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use sea_orm::{ActiveModelTrait, Database, Set};

    use super::*;
    use crate::domain::repository::error::CorruptedData;

    #[tokio::test]
    async fn persisted_mapping_failures_are_returned_instead_of_panicking() {
        let database = Database::connect("sqlite::memory:").await.unwrap();
        crate::data::database::connection::init_database(&database)
            .await
            .unwrap();
        let now = Utc::now();
        let id = Uuid::new_v4();
        subject::ActiveModel {
            id: Set(id),
            created_at: Set(now),
            updated_at: Set(now),
            deleted_at: Set(None),
            sync_status: Set("INVALID".into()),
            sync_version: Set(0),
            name: Set("invalid subject".into()),
            color: Set(String::new()),
        }
        .insert(&database)
        .await
        .unwrap();
        let repository = SeaOrmSubjectRepository::new(&database);

        assert!(matches!(
            repository.find_all(false).await,
            Err(RepositoryFindError::CorruptedData(CorruptedData {
                entity: "subject",
                id: corrupted_id,
                ..
            })) if corrupted_id == id
        ));
    }
}
