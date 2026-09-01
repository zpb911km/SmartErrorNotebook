use chrono::{DateTime, Utc};
use thiserror::Error;

use super::legacy::UnknownSyncStatus;

#[derive(Debug, Clone, PartialEq, Error)]
pub enum DomainError {
    #[error(transparent)]
    UnknownSyncStatus(#[from] UnknownSyncStatus),
    #[error(transparent)]
    InvalidSrsState(#[from] InvalidSrsState),
    #[error(transparent)]
    InvalidFeedback(#[from] InvalidFeedback),
    #[error(transparent)]
    InvalidReviewTime(#[from] InvalidReviewTime),
    #[error(transparent)]
    InvalidFeedbackHistory(#[from] InvalidFeedbackHistory),
}

#[derive(Debug, Clone, PartialEq, Error)]
pub enum InvalidSrsState {
    #[error("SRS stability must be finite and greater than 0, got {0}")]
    Stability(f32),
    #[error("SRS difficulty must be finite and in [1, 10], got {0}")]
    Difficulty(f32),
    #[error("SRS review count must not be negative, got {0}")]
    ReviewCount(i64),
}

#[derive(Debug, Clone, PartialEq, Error)]
#[error("Feedback must be in [0, 1], got {feedback}")]
pub struct InvalidFeedback {
    pub(crate) feedback: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("Review time {reviewed_at} must not be earlier than last review {last_review_at}")]
pub struct InvalidReviewTime {
    pub(crate) reviewed_at: DateTime<Utc>,
    pub(crate) last_review_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("Invalid feedback history: must be an array of at most 5 numbers in [0, 1]")]
pub struct InvalidFeedbackHistory;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::legacy::UnknownSyncStatus;

    #[test]
    fn wraps_specific_errors_transparently() {
        let status: DomainError = UnknownSyncStatus {
            status: "invalid".into(),
        }
        .into();
        assert_eq!(status.to_string(), "Unknown sync status: invalid");

        let feedback: DomainError = InvalidFeedback { feedback: 1.5 }.into();
        assert_eq!(feedback.to_string(), "Feedback must be in [0, 1], got 1.5");

        let history: DomainError = InvalidFeedbackHistory.into();
        assert_eq!(
            history.to_string(),
            "Invalid feedback history: must be an array of at most 5 numbers in [0, 1]"
        );

        let state: DomainError = InvalidSrsState::Stability(0.0).into();
        assert_eq!(
            state.to_string(),
            "SRS stability must be finite and greater than 0, got 0"
        );

        let reviewed_at = Utc::now();
        let last_review_at = reviewed_at + chrono::Duration::seconds(1);
        let time: DomainError = InvalidReviewTime {
            reviewed_at,
            last_review_at,
        }
        .into();
        assert_eq!(
            time.to_string(),
            format!(
                "Review time {reviewed_at} must not be earlier than last review {last_review_at}"
            )
        );
    }
}
