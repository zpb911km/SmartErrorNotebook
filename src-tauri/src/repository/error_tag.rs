use crate::database::entities::{error_tag, prelude::ErrorTag};
use crate::repository::{accept_uuid_insert_result, RepositoryError, RepositoryResult};
use async_trait::async_trait;
use sea_orm::{ActiveModelTrait, ColumnTrait, DbConn, EntityTrait, QueryFilter, Set};
use std::sync::Arc;

pub struct NewErrorTag {
    pub id: String,
    pub question_id: String,
    pub name: String,
    pub color: String,
    pub now: i64,
}
pub struct SyncedErrorTag {
    pub id: String,
    pub version: i32,
    pub deleted_at: Option<i64>,
    pub question_id: String,
    pub name: String,
    pub color: String,
    pub now: i64,
}

#[async_trait]
pub trait ErrorTagRepository: Send + Sync {
    async fn create_many(&self, tags: Vec<NewErrorTag>) -> RepositoryResult<Vec<error_tag::Model>>;
    async fn list_active(&self) -> RepositoryResult<Vec<error_tag::Model>>;
    async fn list_active_by_question(
        &self,
        question_id: String,
    ) -> RepositoryResult<Vec<error_tag::Model>>;
    async fn soft_delete(&self, id: String, now: i64) -> RepositoryResult<()>;
    async fn upsert_synced(&self, input: SyncedErrorTag) -> RepositoryResult<()>;
    async fn update_by_name(
        &self,
        old_name: String,
        new_name: String,
        new_color: String,
        now: i64,
    ) -> RepositoryResult<()>;
    async fn update_by_id(
        &self,
        id: String,
        name: String,
        color: Option<String>,
        now: i64,
    ) -> RepositoryResult<()>;
}

pub struct SeaOrmErrorTagRepository {
    db: Arc<DbConn>,
}
impl SeaOrmErrorTagRepository {
    pub fn new(db: Arc<DbConn>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl ErrorTagRepository for SeaOrmErrorTagRepository {
    async fn create_many(&self, tags: Vec<NewErrorTag>) -> RepositoryResult<Vec<error_tag::Model>> {
        let mut result = Vec::with_capacity(tags.len());
        for input in tags {
            let insert_result = error_tag::ActiveModel {
                id: Set(input.id.clone()),
                question_id: Set(input.question_id),
                name: Set(input.name),
                color: Set(input.color),
                created_at: Set(input.now),
                updated_at: Set(input.now),
                deleted_at: Set(None),
                version: Set(0),
                sync_status: Set("pending".into()),
                sync_hash: Set(None),
            }
            .insert(self.db.as_ref())
            .await;
            accept_uuid_insert_result(insert_result)?;
            result.push(
                ErrorTag::find_by_id(input.id)
                    .one(self.db.as_ref())
                    .await
                    .map_err(RepositoryError::from)?
                    .ok_or_else(|| RepositoryError::not_found("没有成功添加"))?,
            );
        }
        Ok(result)
    }
    async fn list_active(&self) -> RepositoryResult<Vec<error_tag::Model>> {
        ErrorTag::find()
            .filter(error_tag::Column::DeletedAt.is_null())
            .all(self.db.as_ref())
            .await
            .map_err(RepositoryError::from)
    }
    async fn list_active_by_question(
        &self,
        question_id: String,
    ) -> RepositoryResult<Vec<error_tag::Model>> {
        ErrorTag::find()
            .filter(error_tag::Column::QuestionId.eq(question_id))
            .filter(error_tag::Column::DeletedAt.is_null())
            .all(self.db.as_ref())
            .await
            .map_err(RepositoryError::from)
    }
    async fn soft_delete(&self, id: String, now: i64) -> RepositoryResult<()> {
        if let Some(model) = ErrorTag::find_by_id(id)
            .one(self.db.as_ref())
            .await
            .map_err(RepositoryError::from)?
        {
            let mut active: error_tag::ActiveModel = model.into();
            active.deleted_at = Set(Some(now));
            active.updated_at = Set(now);
            active.sync_status = Set("pending".into());
            active
                .update(self.db.as_ref())
                .await
                .map_err(RepositoryError::from)?;
        }
        Ok(())
    }
    async fn upsert_synced(&self, input: SyncedErrorTag) -> RepositoryResult<()> {
        let existing = ErrorTag::find_by_id(&input.id)
            .one(self.db.as_ref())
            .await
            .map_err(|e| RepositoryError::context("Query failed", e))?;
        let is_update = existing.is_some();
        let active = if let Some(model) = existing {
            let mut active: error_tag::ActiveModel = model.into();
            active.question_id = Set(input.question_id);
            active.name = Set(input.name);
            active.color = Set(input.color);
            active.updated_at = Set(input.now);
            active.version = Set(input.version);
            active.sync_status = Set("synced".into());
            active.deleted_at = Set(input.deleted_at);
            active
        } else {
            error_tag::ActiveModel {
                id: Set(input.id),
                question_id: Set(input.question_id),
                name: Set(input.name),
                color: Set(input.color),
                created_at: Set(input.now),
                updated_at: Set(input.now),
                deleted_at: Set(input.deleted_at),
                version: Set(input.version),
                sync_status: Set("synced".into()),
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
    async fn update_by_name(
        &self,
        old_name: String,
        new_name: String,
        new_color: String,
        now: i64,
    ) -> RepositoryResult<()> {
        let models = ErrorTag::find()
            .filter(error_tag::Column::Name.eq(old_name))
            .filter(error_tag::Column::DeletedAt.is_null())
            .all(self.db.as_ref())
            .await
            .map_err(RepositoryError::from)?;
        for model in models {
            let mut active: error_tag::ActiveModel = model.into();
            active.name = Set(new_name.clone());
            active.color = Set(new_color.clone());
            active.updated_at = Set(now);
            active.sync_status = Set("pending".into());
            active
                .update(self.db.as_ref())
                .await
                .map_err(RepositoryError::from)?;
        }
        Ok(())
    }
    async fn update_by_id(
        &self,
        id: String,
        name: String,
        color: Option<String>,
        now: i64,
    ) -> RepositoryResult<()> {
        let model = ErrorTag::find_by_id(id)
            .filter(error_tag::Column::DeletedAt.is_null())
            .one(self.db.as_ref())
            .await
            .map_err(RepositoryError::from)?
            .ok_or_else(|| RepositoryError::not_found("标签不存在"))?;
        let mut active: error_tag::ActiveModel = model.into();
        active.name = Set(name);
        if let Some(color) = color {
            active.color = Set(color);
        }
        active.updated_at = Set(now);
        active.sync_status = Set("pending".into());
        active
            .update(self.db.as_ref())
            .await
            .map_err(RepositoryError::from)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use sea_orm::{ConnectOptions, ConnectionTrait, Database};

    use super::{ErrorTagRepository, NewErrorTag, SeaOrmErrorTagRepository};
    use crate::repository::RepositoryError;

    #[tokio::test]
    async fn update_by_id_propagates_database_errors() {
        let mut options = ConnectOptions::new("sqlite::memory:");
        options.max_connections(1).min_connections(1);
        let db = Database::connect(options).await.unwrap();
        crate::database::init_database(&db).await.unwrap();
        let repository = SeaOrmErrorTagRepository::new(Arc::new(db.clone()));

        repository
            .create_many(vec![NewErrorTag {
                id: "repository-tag".to_string(),
                question_id: "question".to_string(),
                name: "old".to_string(),
                color: "blue".to_string(),
                now: 1,
            }])
            .await
            .unwrap();

        db.execute_unprepared(
            "CREATE TRIGGER reject_error_tag_update \
             BEFORE UPDATE ON error_tags \
             BEGIN SELECT RAISE(FAIL, 'forced update failure'); END;",
        )
        .await
        .unwrap();

        let error = repository
            .update_by_id("repository-tag".to_string(), "new".to_string(), None, 2)
            .await
            .unwrap_err();

        assert!(matches!(error, RepositoryError::Database(_)));
        assert!(error.to_string().contains("forced update failure"));
    }
}
