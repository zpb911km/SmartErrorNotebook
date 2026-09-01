use std::collections::{HashMap, HashSet};

use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, PaginatorTrait, QueryFilter,
    QuerySelect, RelationTrait, Set,
};
use uuid::Uuid;

use crate::model::Tag;
use crate::repository::{
    error::{
        CorruptedData, EntityReference, Referenced, RepositoryDeleteError, RepositoryFindError,
        RepositoryInfrastructureError, RepositorySaveError,
    },
    legacy, TagRepository,
};

use super::super::database::entity::{question, question_tag_cross_ref, tag};
use super::{timestamp, uuid};

pub struct SeaOrmTagRepository<'c, C: ConnectionTrait> {
    connection: &'c C,
}
impl<'c, C: ConnectionTrait> SeaOrmTagRepository<'c, C> {
    pub fn new(connection: &'c C) -> Self {
        Self { connection }
    }
}

#[async_trait::async_trait]
impl<'c, C: ConnectionTrait> TagRepository for SeaOrmTagRepository<'c, C> {
    async fn save(&self, tag: &Tag) -> Result<(), RepositorySaveError> {
        if tag.metadata.deleted_at.is_some()
            && question::Entity::find()
                .join(
                    sea_orm::JoinType::InnerJoin,
                    question::Relation::QuestionTagCrossRef.def(),
                )
                .filter(question_tag_cross_ref::Column::TagId.eq(tag.id))
                .filter(question::Column::DeletedAt.is_null())
                .count(self.connection)
                .await
                .map_err(|error| {
                    RepositoryInfrastructureError::new("count active tag references", error)
                })?
                != 0
        {
            return Err(Referenced {
                target: EntityReference {
                    entity: "tag",
                    id: tag.id,
                },
                referenced_by: "question",
            }
            .into());
        }
        let is_existing = tag::Entity::find_by_id(tag.id)
            .one(self.connection)
            .await
            .map_err(|error| RepositoryInfrastructureError::new("query tag", error))?
            .is_some();
        let mut active_model = tag::ActiveModel {
            id: Set(tag.id),
            created_at: Set(tag.metadata.created_at),
            updated_at: Set(tag.metadata.updated_at),
            deleted_at: Set(tag.metadata.deleted_at),
            sync_status: Set(tag.metadata.sync_status.clone().into()),
            sync_version: Set(tag.metadata.sync_version),
            name: Set(tag.name.clone()),
            color: Set(tag.color.clone()),
        };
        if is_existing {
            active_model.id = sea_orm::ActiveValue::Unchanged(tag.id);
            active_model
                .update(self.connection)
                .await
                .map_err(|error| RepositoryInfrastructureError::new("update tag", error))?;
        } else {
            active_model
                .insert(self.connection)
                .await
                .map_err(|error| RepositoryInfrastructureError::new("insert tag", error))?;
        }
        Ok(())
    }

    async fn delete_by_id(&self, id: &Uuid) -> Result<(), RepositoryDeleteError> {
        if question_tag_cross_ref::Entity::find()
            .filter(question_tag_cross_ref::Column::TagId.eq(*id))
            .count(self.connection)
            .await
            .map_err(|error| RepositoryInfrastructureError::new("count tag references", error))?
            != 0
        {
            return Err(Referenced {
                target: EntityReference {
                    entity: "tag",
                    id: *id,
                },
                referenced_by: "question",
            }
            .into());
        }
        tag::Entity::delete_by_id(*id)
            .exec(self.connection)
            .await
            .map_err(|error| RepositoryInfrastructureError::new("delete tag", error))?;
        Ok(())
    }

    async fn find_all(&self, include_deleted: bool) -> Result<Vec<Tag>, RepositoryFindError> {
        let mut query = tag::Entity::find();
        if !include_deleted {
            query = query.filter(tag::Column::DeletedAt.is_null());
        }
        query
            .all(self.connection)
            .await
            .map_err(|error| RepositoryInfrastructureError::new("list tags", error))?
            .into_iter()
            .map(|model| {
                let id = model.id;
                Tag::try_from(model).map_err(|error| CorruptedData::new("tag", id, error).into())
            })
            .collect()
    }

    async fn find_by_id(&self, id: &Uuid) -> Result<Option<Tag>, RepositoryFindError> {
        tag::Entity::find_by_id(*id)
            .one(self.connection)
            .await
            .map_err(|error| RepositoryInfrastructureError::new("query tag", error))?
            .map(|model| {
                let id = model.id;
                Tag::try_from(model).map_err(|error| CorruptedData::new("tag", id, error).into())
            })
            .transpose()
    }

    async fn find_by_question_id(
        &self,
        question_id: &Uuid,
    ) -> Result<Vec<Tag>, RepositoryFindError> {
        let ids = question_tag_cross_ref::Entity::find()
            .filter(question_tag_cross_ref::Column::QuestionId.eq(*question_id))
            .all(self.connection)
            .await
            .map_err(|error| RepositoryInfrastructureError::new("query tag references", error))?
            .into_iter()
            .map(|link| link.tag_id)
            .collect::<Vec<_>>();
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        tag::Entity::find()
            .filter(tag::Column::Id.is_in(ids))
            .all(self.connection)
            .await
            .map_err(|error| RepositoryInfrastructureError::new("query tags", error))?
            .into_iter()
            .map(|model| {
                let id = model.id;
                Tag::try_from(model).map_err(|error| CorruptedData::new("tag", id, error).into())
            })
            .collect()
    }
}

#[async_trait::async_trait]
impl<'c, C: ConnectionTrait> legacy::ErrorTagRepository for SeaOrmTagRepository<'c, C> {
    async fn create_many(
        &self,
        inputs: Vec<legacy::repository_model::error_tag::NewErrorTag>,
    ) -> Vec<Tag> {
        let mut output = Vec::with_capacity(inputs.len());
        for input in inputs {
            let id = uuid(&input.id, "tag id");
            let now = timestamp(input.now, "tag timestamp");
            let model = tag::ActiveModel {
                id: Set(id),
                created_at: Set(now),
                updated_at: Set(now),
                deleted_at: Set(None),
                sync_status: Set("PENDING".into()),
                sync_version: Set(0),
                name: Set(input.name),
                color: Set(input.color),
            }
            .insert(self.connection)
            .await
            .expect("failed to create tag");
            question_tag_cross_ref::ActiveModel {
                question_id: Set(uuid(&input.question_id, "question id")),
                tag_id: Set(id),
            }
            .insert(self.connection)
            .await
            .expect("failed to relate tag");
            output.push(Tag::try_from(model).expect("created legacy tag must be mappable"));
        }
        output
    }

    async fn list_active(&self) -> Vec<Tag> {
        tag::Entity::find()
            .filter(tag::Column::DeletedAt.is_null())
            .all(self.connection)
            .await
            .expect("failed to list tags")
            .into_iter()
            .map(|model| Tag::try_from(model).expect("persisted legacy tag must be mappable"))
            .collect()
    }

    async fn list_active_with_questions(
        &self,
    ) -> Vec<legacy::repository_model::error_tag::TagWithQuestion> {
        let tags = self.list_active().await;
        if tags.is_empty() {
            return Vec::new();
        }

        let tag_ids: Vec<_> = tags.iter().map(|tag| tag.id).collect();
        let mut relationships = HashMap::<uuid::Uuid, Vec<uuid::Uuid>>::new();
        for relationship in question_tag_cross_ref::Entity::find()
            .filter(question_tag_cross_ref::Column::TagId.is_in(tag_ids))
            .all(self.connection)
            .await
            .expect("failed to query tag relationships")
        {
            relationships
                .entry(relationship.tag_id)
                .or_default()
                .push(relationship.question_id);
        }

        let mut output = Vec::new();
        for tag in tags {
            let question_ids = relationships.remove(&tag.id).unwrap_or_default();
            if question_ids.is_empty() {
                output.push(legacy::repository_model::error_tag::TagWithQuestion {
                    tag,
                    question_id: None,
                });
                continue;
            }
            for question_id in question_ids {
                output.push(legacy::repository_model::error_tag::TagWithQuestion {
                    tag: tag.clone(),
                    question_id: Some(question_id.to_string()),
                });
            }
        }
        output
    }

    async fn list_active_by_question(&self, question_id: String) -> Vec<Tag> {
        let links = question_tag_cross_ref::Entity::find()
            .filter(
                question_tag_cross_ref::Column::QuestionId.eq(uuid(&question_id, "question id")),
            )
            .all(self.connection)
            .await
            .expect("failed to query tag relations");
        if links.is_empty() {
            return Vec::new();
        }
        tag::Entity::find()
            .filter(tag::Column::Id.is_in(links.into_iter().map(|link| link.tag_id)))
            .filter(tag::Column::DeletedAt.is_null())
            .all(self.connection)
            .await
            .expect("failed to query tags")
            .into_iter()
            .map(|model| Tag::try_from(model).expect("persisted legacy tag must be mappable"))
            .collect()
    }

    async fn unlink(&self, question_id: String, id: String, now: i64) -> Result<(), String> {
        let id = uuid(&id, "tag id");
        let question_id = uuid(&question_id, "question id");
        let deleted = question_tag_cross_ref::Entity::delete_by_id((question_id, id))
            .exec(self.connection)
            .await
            .map_err(|error| format!("failed to unlink tag: {error}"))?;
        if deleted.rows_affected == 0 {
            return Err("tag is not linked to the question".into());
        }
        let model = tag::Entity::find_by_id(id)
            .one(self.connection)
            .await
            .map_err(|error| format!("failed to query tag: {error}"))?
            .ok_or_else(|| "Tag not found".to_owned())?;
        let remaining = question_tag_cross_ref::Entity::find()
            .filter(question_tag_cross_ref::Column::TagId.eq(id))
            .count(self.connection)
            .await
            .map_err(|error| format!("failed to count tag links: {error}"))?;
        let mut active: tag::ActiveModel = model.into();
        let now = timestamp(now, "tag timestamp");
        if remaining == 0 {
            active.deleted_at = Set(Some(now));
        }
        active.updated_at = Set(now);
        active.sync_status = Set("PENDING".into());
        active
            .update(self.connection)
            .await
            .map_err(|error| format!("failed to update tag after unlink: {error}"))?;
        Ok(())
    }

    async fn upsert_synced(
        &self,
        input: legacy::repository_model::error_tag::SyncedErrorTag,
    ) -> Result<(), String> {
        let id = uuid(&input.id, "tag id");
        let question_ids: HashSet<_> = input
            .question_ids
            .iter()
            .map(|value| uuid(value, "question id"))
            .collect();
        if input.deleted_at.is_none() && question_ids.is_empty() {
            return Err("an active tag must be linked to at least one question".into());
        }
        let now = timestamp(input.now, "tag timestamp");
        let exists = tag::Entity::find_by_id(id)
            .one(self.connection)
            .await
            .expect("failed to query tag")
            .is_some();
        let mut active = tag::ActiveModel {
            id: Set(id),
            created_at: Set(now),
            updated_at: Set(now),
            deleted_at: Set(input
                .deleted_at
                .map(|value| timestamp(value, "tag deleted_at"))),
            sync_status: Set("SYNCED".into()),
            sync_version: Set(i64::from(input.version)),
            name: Set(input.name),
            color: Set(input.color),
        };
        if exists {
            active.id = sea_orm::ActiveValue::Unchanged(id);
            active
                .update(self.connection)
                .await
                .expect("failed to update synced tag");
        } else {
            active
                .insert(self.connection)
                .await
                .expect("failed to insert synced tag");
        }
        let existing_links = question_tag_cross_ref::Entity::find()
            .filter(question_tag_cross_ref::Column::TagId.eq(id))
            .all(self.connection)
            .await
            .map_err(|error| format!("failed to query tag relations: {error}"))?;
        let existing_question_ids: HashSet<_> =
            existing_links.iter().map(|link| link.question_id).collect();
        for question_id in existing_question_ids.difference(&question_ids) {
            question_tag_cross_ref::Entity::delete_by_id((*question_id, id))
                .exec(self.connection)
                .await
                .map_err(|error| format!("failed to remove tag relation: {error}"))?;
        }
        for question_id in question_ids.difference(&existing_question_ids) {
            question_tag_cross_ref::ActiveModel {
                question_id: Set(*question_id),
                tag_id: Set(id),
            }
            .insert(self.connection)
            .await
            .map_err(|error| format!("failed to insert tag relation: {error}"))?;
        }
        Ok(())
    }

    async fn update_by_name(
        &self,
        old_name: String,
        new_name: String,
        new_color: String,
        now: i64,
    ) -> Result<(), String> {
        let models = tag::Entity::find()
            .filter(tag::Column::Name.eq(old_name))
            .filter(tag::Column::DeletedAt.is_null())
            .all(self.connection)
            .await
            .map_err(|error| format!("failed to query tags: {error}"))?;
        for model in models {
            let mut active: tag::ActiveModel = model.into();
            active.name = Set(new_name.clone());
            active.color = Set(new_color.clone());
            active.updated_at = Set(timestamp(now, "tag timestamp"));
            active.sync_status = Set("PENDING".into());
            active
                .update(self.connection)
                .await
                .map_err(|error| format!("failed to update tag: {error}"))?;
        }
        Ok(())
    }

    async fn update_by_id(
        &self,
        id: String,
        name: String,
        color: Option<String>,
        now: i64,
    ) -> Result<(), String> {
        let model = tag::Entity::find_by_id(uuid(&id, "tag id"))
            .one(self.connection)
            .await
            .map_err(|error| format!("failed to query tag: {error}"))?
            .ok_or_else(|| "Tag not found".to_owned())?;
        let mut active: tag::ActiveModel = model.into();
        active.name = Set(name);
        if let Some(color) = color {
            active.color = Set(color);
        }
        active.updated_at = Set(timestamp(now, "tag timestamp"));
        active.sync_status = Set("PENDING".into());
        active
            .update(self.connection)
            .await
            .map_err(|error| format!("failed to update tag: {error}"))?;
        Ok(())
    }
}
