use super::super::database::entity::subject;
use super::{timestamp, uuid};
use crate::model::Subject;
use crate::repository::legacy;
use sea_orm::{ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, Set};

pub struct SeaOrmSubjectRepository<'c, C: ConnectionTrait> {
    connection: &'c C,
}
impl<'c, C: ConnectionTrait> SeaOrmSubjectRepository<'c, C> {
    pub fn new(connection: &'c C) -> Self {
        Self {
            connection: connection,
        }
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
            .map(Into::into)
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
        .into()
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
        Ok(active
            .update(self.connection)
            .await
            .map_err(|error| format!("failed to update subject: {error}"))?
            .into())
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
