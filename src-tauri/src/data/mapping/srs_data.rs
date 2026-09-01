use crate::data::database::entity::srs_data;
use crate::model::SrsData;

use super::{error::MappingError, metadata};

impl TryFrom<srs_data::Model> for SrsData {
    type Error = MappingError;

    fn try_from(value: srs_data::Model) -> Result<Self, Self::Error> {
        let feedback_history = serde_json::from_value(value.feedback_history)
            .map_err(MappingError::DeserializeFeedbackHistory)?;
        Ok(SrsData::new_with_state(
            value.question_id,
            metadata::try_from_values(
                value.created_at,
                value.updated_at,
                value.deleted_at,
                value.sync_status,
                value.sync_version,
            )?,
            value.stability,
            value.difficulty,
            value.next_review_at,
            value.last_reviewed_at,
            value.review_count,
            feedback_history,
        )?)
    }
}
