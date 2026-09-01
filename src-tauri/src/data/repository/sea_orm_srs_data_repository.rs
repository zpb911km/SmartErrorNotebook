use sea_orm::{ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, Set};
use uuid::Uuid;

use crate::data::mapping::legacy::normalize_srs_state;
use crate::domain::model::SrsData;
use crate::domain::repository::{
    error::{
        CorruptedData, EntityReference, MissingReference, RepositoryFindError,
        RepositoryInfrastructureError, RepositorySaveError,
    },
    legacy, SrsDataRepository,
};

use super::super::database::entity::{question, srs_data};
use super::{timestamp, uuid};

pub struct SeaOrmSrsDataRepository<'c, C: ConnectionTrait> {
    connection: &'c C,
}
impl<'c, C: ConnectionTrait> SeaOrmSrsDataRepository<'c, C> {
    pub fn new(connection: &'c C) -> Self {
        Self { connection }
    }
}

fn feedback(value: &str) -> Result<serde_json::Value, serde_json::Error> {
    serde_json::from_str(value)
}

#[async_trait::async_trait]
impl<'c, C: ConnectionTrait> SrsDataRepository for SeaOrmSrsDataRepository<'c, C> {
    async fn save(&self, srs_data: &SrsData) -> Result<(), RepositorySaveError> {
        let feedback_history = serde_json::to_value(srs_data.feedback_history())
            .expect("SRS feedback history is always serializable");
        let mut question_query = question::Entity::find_by_id(srs_data.question_id);
        if srs_data.metadata.deleted_at.is_none() {
            question_query = question_query.filter(question::Column::DeletedAt.is_null());
        }
        if question_query
            .one(self.connection)
            .await
            .map_err(|error| RepositoryInfrastructureError::new("query SRS question", error))?
            .is_none()
        {
            return Err(MissingReference {
                owner: EntityReference {
                    entity: "srs_data",
                    id: srs_data.question_id,
                },
                missing: vec![EntityReference {
                    entity: "question",
                    id: srs_data.question_id,
                }],
            }
            .into());
        }
        let is_existing = srs_data::Entity::find_by_id(srs_data.question_id)
            .one(self.connection)
            .await
            .map_err(|error| RepositoryInfrastructureError::new("query SRS data", error))?
            .is_some();
        let mut active_model = srs_data::ActiveModel {
            question_id: Set(srs_data.question_id),
            created_at: Set(srs_data.metadata.created_at),
            updated_at: Set(srs_data.metadata.updated_at),
            deleted_at: Set(srs_data.metadata.deleted_at),
            sync_status: Set(srs_data.metadata.sync_status.clone().into()),
            sync_version: Set(srs_data.metadata.sync_version),
            stability: Set(srs_data.stability()),
            difficulty: Set(srs_data.difficulty()),
            next_review_at: Set(srs_data.next_review_at()),
            last_reviewed_at: Set(srs_data.last_review_at()),
            review_count: Set(srs_data.review_count()),
            feedback_history: Set(feedback_history),
        };
        if is_existing {
            active_model.question_id = sea_orm::ActiveValue::Unchanged(srs_data.question_id);
            active_model
                .update(self.connection)
                .await
                .map_err(|error| RepositoryInfrastructureError::new("update SRS data", error))?;
        } else {
            active_model
                .insert(self.connection)
                .await
                .map_err(|error| RepositoryInfrastructureError::new("insert SRS data", error))?;
        }
        Ok(())
    }

    async fn find_all(&self, include_deleted: bool) -> Result<Vec<SrsData>, RepositoryFindError> {
        let mut query = srs_data::Entity::find();
        if !include_deleted {
            query = query.filter(srs_data::Column::DeletedAt.is_null());
        }
        query
            .all(self.connection)
            .await
            .map_err(|error| RepositoryInfrastructureError::new("list SRS data", error))?
            .into_iter()
            .map(|model| {
                let id = model.question_id;
                SrsData::try_from(model)
                    .map_err(|error| CorruptedData::new("srs_data", id, error).into())
            })
            .collect()
    }

    async fn find_by_question_id(&self, id: &Uuid) -> Result<Option<SrsData>, RepositoryFindError> {
        srs_data::Entity::find_by_id(*id)
            .one(self.connection)
            .await
            .map_err(|error| RepositoryInfrastructureError::new("query SRS data", error))?
            .map(|model| {
                let id = model.question_id;
                SrsData::try_from(model)
                    .map_err(|error| CorruptedData::new("srs_data", id, error).into())
            })
            .transpose()
    }
}

#[async_trait::async_trait]
impl<'c, C: ConnectionTrait> legacy::SrsDataRepository for SeaOrmSrsDataRepository<'c, C> {
    async fn find_by_question(&self, question_id: String, active_only: bool) -> Option<SrsData> {
        let mut query = srs_data::Entity::find_by_id(uuid(&question_id, "question id"));
        if active_only {
            query = query.filter(srs_data::Column::DeletedAt.is_null());
        }
        query
            .one(self.connection)
            .await
            .expect("failed to query srs data")
            .map(|model| {
                model
                    .try_into_legacy_srs_data()
                    .expect("persisted legacy SRS data must be mappable")
            })
    }

    async fn list_active(&self) -> Vec<SrsData> {
        srs_data::Entity::find()
            .filter(srs_data::Column::DeletedAt.is_null())
            .all(self.connection)
            .await
            .expect("failed to list srs data")
            .into_iter()
            .map(|model| {
                model
                    .try_into_legacy_srs_data()
                    .expect("persisted legacy SRS data must be mappable")
            })
            .collect()
    }

    async fn create(&self, input: legacy::repository_model::srs_data::NewSrsData) -> SrsData {
        let now = timestamp(input.now, "srs timestamp");
        let state = normalize_srs_state(
            input.stability,
            input.difficulty,
            i64::from(input.review_count),
            feedback(&input.feedback_history).unwrap_or_default(),
        );
        srs_data::ActiveModel {
            question_id: Set(uuid(&input.question_id, "question id")),
            created_at: Set(now),
            updated_at: Set(now),
            deleted_at: Set(None),
            sync_status: Set("PENDING".into()),
            sync_version: Set(0),
            stability: Set(state.stability),
            difficulty: Set(state.difficulty),
            next_review_at: Set(input
                .next_review_at
                .map(|value| timestamp(value, "next_review_at"))),
            last_reviewed_at: Set(input
                .last_reviewed_at
                .map(|value| timestamp(value, "last_review_at"))),
            review_count: Set(state.review_count),
            feedback_history: Set(serde_json::to_value(state.feedback_history)
                .expect("normalized SRS feedback history is always serializable")),
        }
        .insert(self.connection)
        .await
        .expect("failed to create srs data")
        .try_into_legacy_srs_data()
        .expect("created legacy SRS data must be mappable")
    }

    async fn update_state(
        &self,
        input: legacy::repository_model::srs_data::SrsStateChanges,
    ) -> SrsData {
        let state = normalize_srs_state(
            input.stability,
            input.difficulty,
            i64::from(input.review_count),
            feedback(&input.feedback_history).unwrap_or_default(),
        );
        let model = srs_data::Entity::find_by_id(uuid(&input.id, "srs id"))
            .one(self.connection)
            .await
            .expect("failed to query srs data")
            .expect("SRS data not found");
        let mut active: srs_data::ActiveModel = model.into();
        active.stability = Set(state.stability);
        active.difficulty = Set(state.difficulty);
        active.next_review_at = Set(input
            .next_review_at
            .map(|value| timestamp(value, "next_review_at")));
        active.last_reviewed_at = Set(input
            .last_reviewed_at
            .map(|value| timestamp(value, "last_review_at")));
        active.review_count = Set(state.review_count);
        active.feedback_history = Set(serde_json::to_value(state.feedback_history)
            .expect("normalized SRS feedback history is always serializable"));
        active.deleted_at = Set(input
            .deleted_at
            .map(|value| timestamp(value, "srs deleted_at")));
        active.updated_at = Set(timestamp(input.now, "srs timestamp"));
        active.sync_status = Set("PENDING".into());
        active
            .update(self.connection)
            .await
            .expect("failed to update srs data")
            .try_into_legacy_srs_data()
            .expect("updated legacy SRS data must be mappable")
    }

    async fn upsert_synced(&self, input: legacy::repository_model::srs_data::SyncedSrsData) {
        let id = uuid(&input.question_id, "question id");
        let now = timestamp(input.now, "srs timestamp");
        let state = normalize_srs_state(
            input.stability,
            input.difficulty,
            i64::from(input.review_count),
            feedback(&input.feedback_history).unwrap_or_default(),
        );
        let exists = srs_data::Entity::find_by_id(id)
            .one(self.connection)
            .await
            .expect("failed to query srs data")
            .is_some();
        let mut active = srs_data::ActiveModel {
            question_id: Set(id),
            created_at: Set(now),
            updated_at: Set(now),
            deleted_at: Set(input
                .deleted_at
                .map(|value| timestamp(value, "srs deleted_at"))),
            sync_status: Set("SYNCED".into()),
            sync_version: Set(i64::from(input.version)),
            stability: Set(state.stability),
            difficulty: Set(state.difficulty),
            next_review_at: Set(input
                .next_review_at
                .map(|value| timestamp(value, "next_review_at"))),
            last_reviewed_at: Set(input
                .last_reviewed_at
                .map(|value| timestamp(value, "last_review_at"))),
            review_count: Set(state.review_count),
            feedback_history: Set(serde_json::to_value(state.feedback_history)
                .expect("normalized SRS feedback history is always serializable")),
        };
        if exists {
            active.question_id = sea_orm::ActiveValue::Unchanged(id);
            active
                .update(self.connection)
                .await
                .expect("failed to update synced srs data");
        } else {
            active
                .insert(self.connection)
                .await
                .expect("failed to insert synced srs data");
        }
    }
}
