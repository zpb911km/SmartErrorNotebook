use crate::data::database::entity::srs_data;
use crate::domain::model::SrsData;

use super::{error::MappingError, metadata};

pub(crate) struct NormalizedSrsState {
    pub stability: f32,
    pub difficulty: f32,
    pub review_count: i64,
    pub feedback_history: Vec<f32>,
}

pub(crate) fn normalize_srs_state(
    stability: f32,
    difficulty: f32,
    review_count: i64,
    feedback_history: serde_json::Value,
) -> NormalizedSrsState {
    let mut feedback_history = serde_json::from_value::<Vec<f32>>(feedback_history)
        .unwrap_or_default()
        .into_iter()
        .filter(|feedback| feedback.is_finite() && (0.0..=1.0).contains(feedback))
        .collect::<Vec<_>>();
    if feedback_history.len() > SrsData::MAX_FEEDBACK_HISTORY_LEN {
        feedback_history.drain(
            0..feedback_history
                .len()
                .saturating_sub(SrsData::MAX_FEEDBACK_HISTORY_LEN),
        );
    }
    NormalizedSrsState {
        stability: if stability.is_finite() && stability > 0.0 {
            stability
        } else {
            SrsData::INITIAL_STABILITY
        },
        difficulty: if difficulty.is_finite() && (1.0..=10.0).contains(&difficulty) {
            difficulty
        } else {
            SrsData::INITIAL_DIFFICULTY
        },
        review_count: review_count.max(0),
        feedback_history,
    }
}

impl srs_data::Model {
    pub(crate) fn try_into_legacy_srs_data(self) -> Result<SrsData, MappingError> {
        let state = normalize_srs_state(
            self.stability,
            self.difficulty,
            self.review_count,
            self.feedback_history,
        );
        Ok(SrsData::new_with_state(
            self.question_id,
            metadata::try_from_values(
                self.created_at,
                self.updated_at,
                self.deleted_at,
                self.sync_status,
                self.sync_version,
            )?,
            state.stability,
            state.difficulty,
            self.next_review_at,
            self.last_reviewed_at,
            state.review_count,
            state.feedback_history,
        )?)
    }
}
