use super::super::database::entity::srs_data;
use super::{timestamp, uuid};
use crate::model::SrsData;
use crate::repository::legacy;
use sea_orm::{ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, Set};

pub struct SeaOrmSrsDataRepository<'c, C: ConnectionTrait> {
    connection: &'c C,
}
impl<'c, C: ConnectionTrait> SeaOrmSrsDataRepository<'c, C> {
    pub fn new(connection: &'c C) -> Self {
        Self {
            connection: connection,
        }
    }
}

fn feedback(value: &str) -> serde_json::Value {
    serde_json::from_str(value).expect("invalid feedback_history")
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
            .map(Into::into)
    }

    async fn list_active(&self) -> Vec<SrsData> {
        srs_data::Entity::find()
            .filter(srs_data::Column::DeletedAt.is_null())
            .all(self.connection)
            .await
            .expect("failed to list srs data")
            .into_iter()
            .map(Into::into)
            .collect()
    }

    async fn create(&self, input: legacy::repository_model::srs_data::NewSrsData) -> SrsData {
        let now = timestamp(input.now, "srs timestamp");
        srs_data::ActiveModel {
            question_id: Set(uuid(&input.question_id, "question id")),
            created_at: Set(now),
            updated_at: Set(now),
            deleted_at: Set(None),
            sync_status: Set("PENDING".into()),
            sync_version: Set(0),
            stability: Set(input.stability),
            difficulty: Set(input.difficulty),
            next_review_at: Set(input
                .next_review_at
                .map(|value| timestamp(value, "next_review_at"))),
            last_reviewed_at: Set(input
                .last_reviewed_at
                .map(|value| timestamp(value, "last_review_at"))),
            review_count: Set(i64::from(input.review_count)),
            feedback_history: Set(feedback(&input.feedback_history)),
        }
        .insert(self.connection)
        .await
        .expect("failed to create srs data")
        .into()
    }

    async fn update_state(
        &self,
        input: legacy::repository_model::srs_data::SrsStateChanges,
    ) -> SrsData {
        let model = srs_data::Entity::find_by_id(uuid(&input.id, "srs id"))
            .one(self.connection)
            .await
            .expect("failed to query srs data")
            .expect("SRS data not found");
        let mut active: srs_data::ActiveModel = model.into();
        active.stability = Set(input.stability);
        active.difficulty = Set(input.difficulty);
        active.next_review_at = Set(input
            .next_review_at
            .map(|value| timestamp(value, "next_review_at")));
        active.last_reviewed_at = Set(input
            .last_reviewed_at
            .map(|value| timestamp(value, "last_review_at")));
        active.review_count = Set(i64::from(input.review_count));
        active.feedback_history = Set(feedback(&input.feedback_history));
        active.deleted_at = Set(input
            .deleted_at
            .map(|value| timestamp(value, "srs deleted_at")));
        active.updated_at = Set(timestamp(input.now, "srs timestamp"));
        active.sync_status = Set("PENDING".into());
        active
            .update(self.connection)
            .await
            .expect("failed to update srs data")
            .into()
    }

    async fn upsert_synced(&self, input: legacy::repository_model::srs_data::SyncedSrsData) {
        let id = uuid(&input.question_id, "question id");
        let now = timestamp(input.now, "srs timestamp");
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
            stability: Set(input.stability),
            difficulty: Set(input.difficulty),
            next_review_at: Set(input
                .next_review_at
                .map(|value| timestamp(value, "next_review_at"))),
            last_reviewed_at: Set(input
                .last_reviewed_at
                .map(|value| timestamp(value, "last_review_at"))),
            review_count: Set(i64::from(input.review_count)),
            feedback_history: Set(feedback(&input.feedback_history)),
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
