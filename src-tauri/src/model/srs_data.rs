use chrono::{DateTime, Duration, Utc};
use uuid::Uuid;

use super::error::{
    DomainError, InvalidFeedback, InvalidFeedbackHistory, InvalidReviewTime, InvalidSrsState,
};
use super::{Metadata, SyncStatus};

// FSRS-5 default weights. The model uses continuous feedback in [0, 1],
// mapped to the original FSRS grades with G = 3f + 1.
const W: [f32; 19] = [
    0.40255, 1.18385, 3.173, 15.69105, 7.1949, 0.5345, 1.4604, 0.0046, 1.54575, 0.1192, 1.01925,
    1.9395, 0.11, 0.29605, 2.2698, 0.2315, 2.9898, 0.51655, 0.6621,
];
const MAX_INTERVAL_DAYS: i64 = 1000;
const MAX_STABILITY_DAYS: f32 = 36500.0;
const TARGET_RETENTION: f64 = 0.9;
const LAPSE_THRESHOLD: f32 = 0.2;

#[derive(Debug, Clone, PartialEq)]
pub struct SrsData {
    pub question_id: Uuid,
    pub metadata: Metadata,
    /// Scheduling stability must be changed through `review` or `reset`.
    stability: f32,
    /// Scheduling difficulty must be changed through `review` or `reset`.
    difficulty: f32,
    /// The next review time must be changed together with the scheduling state.
    next_review_at: Option<DateTime<Utc>>,
    /// The last review time must be changed together with the scheduling state.
    last_review_at: Option<DateTime<Utc>>,
    /// The review count must be changed through `review` or `reset`.
    review_count: i64,
    /// Feedback history must be changed through `review` so it remains valid and bounded.
    feedback_history: Vec<f32>,
}
impl SrsData {
    pub(crate) const INITIAL_STABILITY: f32 = W[2];
    pub(crate) const INITIAL_DIFFICULTY: f32 = W[4];
    pub(crate) const MAX_FEEDBACK_HISTORY_LEN: usize = 5;

    pub(crate) fn new(question_id: Uuid, metadata: Metadata) -> Self {
        let now = metadata.created_at;
        Self {
            question_id,
            metadata,
            stability: Self::INITIAL_STABILITY,
            difficulty: Self::INITIAL_DIFFICULTY,
            next_review_at: Some(now + Duration::days(1)),
            last_review_at: Some(now),
            review_count: 1,
            feedback_history: Vec::new(),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new_with_state(
        question_id: Uuid,
        metadata: Metadata,
        stability: f32,
        difficulty: f32,
        next_review_at: Option<DateTime<Utc>>,
        last_review_at: Option<DateTime<Utc>>,
        review_count: i64,
        feedback_history: Vec<f32>,
    ) -> Result<Self, DomainError> {
        validate_state(stability, difficulty, review_count)?;
        validate_feedback_history(&feedback_history)?;
        Ok(Self {
            question_id,
            metadata,
            stability,
            difficulty,
            next_review_at,
            last_review_at,
            review_count,
            feedback_history,
        })
    }

    pub(crate) fn stability(&self) -> f32 {
        self.stability
    }

    pub(crate) fn difficulty(&self) -> f32 {
        self.difficulty
    }

    pub(crate) fn next_review_at(&self) -> Option<DateTime<Utc>> {
        self.next_review_at
    }

    pub(crate) fn last_review_at(&self) -> Option<DateTime<Utc>> {
        self.last_review_at
    }

    pub(crate) fn review_count(&self) -> i64 {
        self.review_count
    }

    pub(crate) fn feedback_history(&self) -> &[f32] {
        &self.feedback_history
    }

    /// Predicts the current retrievability using the FSRS power forgetting curve.
    pub(crate) fn retrievability_at(&self, now: i64) -> f32 {
        predict_retrievability(
            self.stability,
            days_elapsed(self.last_review_at.map(|value| value.timestamp()), now),
        )
    }

    /// Applies a continuous review feedback value and calculates the next SRS state.
    pub(crate) fn review(&mut self, now: DateTime<Utc>, feedback: f32) -> Result<f32, DomainError> {
        if !(0.0..=1.0).contains(&feedback) {
            return Err(InvalidFeedback { feedback }.into());
        }
        if let Some(last_review_at) = self.last_review_at {
            if now < last_review_at {
                return Err(InvalidReviewTime {
                    reviewed_at: now,
                    last_review_at,
                }
                .into());
            }
        }
        validate_feedback_history(&self.feedback_history)?;
        let mut feedback_history = self.feedback_history.clone();
        feedback_history.push(feedback);
        if feedback_history.len() > Self::MAX_FEEDBACK_HISTORY_LEN {
            feedback_history.drain(0..feedback_history.len() - Self::MAX_FEEDBACK_HISTORY_LEN);
        }

        let (next_interval_days, stability, difficulty, next_review_at) = if feedback == 0.0 {
            (1, Self::INITIAL_STABILITY, Self::INITIAL_DIFFICULTY, now)
        } else if feedback == 1.0 {
            (
                MAX_INTERVAL_DAYS,
                (self.stability * 5.0).min(MAX_STABILITY_DAYS),
                self.difficulty,
                now + Duration::days(MAX_INTERVAL_DAYS),
            )
        } else {
            let grade = 3.0 * feedback + 1.0;
            let elapsed_days = days_elapsed(
                self.last_review_at.map(|value| value.timestamp()),
                now.timestamp(),
            );
            let retrievability = predict_retrievability(self.stability, elapsed_days);
            let difficulty_delta = -W[6] * (grade - 3.0);
            let damped_difficulty =
                self.difficulty + difficulty_delta * (10.0 - self.difficulty) / 9.0;
            let initial_easy_difficulty = initial_difficulty(4.0);
            let difficulty = clamp(
                W[7] * initial_easy_difficulty + (1.0 - W[7]) * damped_difficulty,
                1.0,
                10.0,
            );
            let stability = if feedback < LAPSE_THRESHOLD {
                W[11]
                    * self.difficulty.powf(-W[12])
                    * ((self.stability + 1.0).powf(W[13]) - 1.0)
                    * (W[14] * (1.0 - retrievability)).exp()
            } else {
                self.stability
                    * (W[8].exp()
                        * (11.0 - difficulty)
                        * self.stability.powf(-W[9])
                        * ((W[10] * (1.0 - retrievability)).exp() - 1.0)
                        * interpolate_multiplier(grade)
                        + 1.0)
            }
            .max(0.01);
            let stability = if elapsed_days < 1.0 {
                stability * (W[17] * (grade - 3.0 + W[18])).exp()
            } else {
                stability
            }
            .min(MAX_STABILITY_DAYS);
            let interval = compute_next_interval(stability);
            (
                interval,
                stability,
                difficulty,
                now + Duration::days(interval),
            )
        };

        self.stability = stability;
        self.difficulty = difficulty;
        self.next_review_at = Some(next_review_at);
        self.last_review_at = Some(now);
        self.review_count = self.review_count.saturating_add(1);
        self.feedback_history = feedback_history;
        self.metadata.updated_at = now;
        self.metadata.sync_status = SyncStatus::Pending;
        self.metadata.deleted_at = None;
        Ok(next_interval_days as f32)
    }

    pub(crate) fn reset(&mut self, now: DateTime<Utc>) {
        self.stability = Self::INITIAL_STABILITY;
        self.difficulty = Self::INITIAL_DIFFICULTY;
        self.next_review_at = Some(now);
        self.last_review_at = Some(now);
        self.review_count = 1;
        self.feedback_history.clear();
        self.metadata.updated_at = now;
        self.metadata.sync_status = SyncStatus::Pending;
        self.metadata.deleted_at = None;
    }
}

fn validate_state(stability: f32, difficulty: f32, review_count: i64) -> Result<(), DomainError> {
    if !stability.is_finite() || stability <= 0.0 {
        return Err(InvalidSrsState::Stability(stability).into());
    }
    if !difficulty.is_finite() || !(1.0..=10.0).contains(&difficulty) {
        return Err(InvalidSrsState::Difficulty(difficulty).into());
    }
    if review_count < 0 {
        return Err(InvalidSrsState::ReviewCount(review_count).into());
    }
    Ok(())
}

fn validate_feedback_history(history: &[f32]) -> Result<(), DomainError> {
    if history.len() > SrsData::MAX_FEEDBACK_HISTORY_LEN
        || history
            .iter()
            .any(|value| !value.is_finite() || !(0.0..=1.0).contains(value))
    {
        return Err(InvalidFeedbackHistory.into());
    }
    Ok(())
}

fn days_elapsed(last_review_at: Option<i64>, now: i64) -> f64 {
    last_review_at
        .map(|last_review_at| now.saturating_sub(last_review_at).max(0) as f64 / (24.0 * 3600.0))
        .unwrap_or(0.0)
}

fn predict_retrievability(stability: f32, elapsed_days: f64) -> f32 {
    if stability <= 0.0 {
        return 0.0;
    }
    (1.0 + (19.0 / 81.0) * elapsed_days / stability as f64).powf(-0.5) as f32
}

fn compute_next_interval(stability: f32) -> i64 {
    if stability <= 0.0 {
        return 1;
    }
    let interval_days = stability as f64 * (81.0 / 19.0) * (TARGET_RETENTION.powf(-2.0) - 1.0);
    (interval_days.max(1.0).ceil() as i64).min(MAX_INTERVAL_DAYS)
}

fn initial_difficulty(grade: f32) -> f32 {
    (W[4] - (W[5] * (grade - 1.0)).exp() + 1.0).clamp(1.0, 10.0)
}

fn interpolate_multiplier(grade: f32) -> f32 {
    let grade = grade.clamp(2.0, 4.0);
    if grade <= 3.0 {
        W[15] + (1.0 - W[15]) * (grade - 2.0)
    } else {
        1.0 + (W[16] - 1.0) * (grade - 3.0)
    }
}

fn clamp(value: f32, low: f32, high: f32) -> f32 {
    value.max(low).min(high)
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;

    fn srs_data(feedback_history: Vec<f32>) -> SrsData {
        SrsData::new_with_state(
            Uuid::new_v4(),
            Metadata::new(Utc::now()),
            3.0,
            5.0,
            None,
            None,
            0,
            feedback_history,
        )
        .unwrap()
    }

    #[test]
    fn validates_scheduling_state() {
        let metadata = Metadata::new(Utc::now());
        for stability in [0.0, -1.0, f32::INFINITY, f32::NAN] {
            assert!(matches!(
                SrsData::new_with_state(
                    Uuid::new_v4(),
                    metadata.clone(),
                    stability,
                    5.0,
                    None,
                    None,
                    0,
                    Vec::new(),
                ),
                Err(DomainError::InvalidSrsState(InvalidSrsState::Stability(value)))
                    if value.to_bits() == stability.to_bits()
            ));
        }
        for difficulty in [0.0, 10.1, f32::INFINITY, f32::NAN] {
            assert!(matches!(
                SrsData::new_with_state(
                    Uuid::new_v4(),
                    metadata.clone(),
                    3.0,
                    difficulty,
                    None,
                    None,
                    0,
                    Vec::new(),
                ),
                Err(DomainError::InvalidSrsState(InvalidSrsState::Difficulty(value)))
                    if value.to_bits() == difficulty.to_bits()
            ));
        }
        assert_eq!(
            SrsData::new_with_state(
                Uuid::new_v4(),
                metadata,
                3.0,
                5.0,
                None,
                None,
                -1,
                Vec::new(),
            ),
            Err(DomainError::InvalidSrsState(InvalidSrsState::ReviewCount(
                -1
            )))
        );
    }

    #[test]
    fn validates_feedback_history() {
        assert_eq!(
            srs_data(vec![0.0, 0.5, 1.0]).feedback_history(),
            &[0.0, 0.5, 1.0]
        );

        for value in [
            vec![-0.1],
            vec![1.1],
            vec![0.0; SrsData::MAX_FEEDBACK_HISTORY_LEN + 1],
            vec![f32::NAN],
            vec![f32::INFINITY],
        ] {
            assert_eq!(
                SrsData::new_with_state(
                    Uuid::new_v4(),
                    Metadata::new(Utc::now()),
                    3.0,
                    5.0,
                    None,
                    None,
                    0,
                    value,
                ),
                Err(DomainError::InvalidFeedbackHistory(InvalidFeedbackHistory))
            );
        }
    }

    #[test]
    fn new_creates_an_initially_scheduled_card() {
        let now = Utc::now();
        let card = SrsData::new(Uuid::new_v4(), Metadata::new(now));
        assert_eq!(card.stability(), SrsData::INITIAL_STABILITY);
        assert_eq!(card.difficulty(), SrsData::INITIAL_DIFFICULTY);
        assert_eq!(card.next_review_at(), Some(now + Duration::days(1)));
        assert_eq!(card.last_review_at(), Some(now));
        assert_eq!(card.review_count(), 1);
        assert!(card.feedback_history().is_empty());
    }

    #[test]
    fn predicts_retrievability_from_card_state() {
        let mut card = srs_data(Vec::new());
        card.stability = 10.0;
        card.last_review_at = Some(DateTime::from_timestamp(1_000_000, 0).unwrap());

        assert!((card.retrievability_at(1_000_000) - 1.0).abs() < 0.001);
        assert!((card.retrievability_at(999_999) - 1.0).abs() < 0.001);
        assert!((card.retrievability_at(1_000_000 + 10 * 24 * 3600) - 0.9).abs() < 0.001);
        assert!(card.retrievability_at(1_000_000 + 100_000_000 * 24 * 3600) < 0.001);
    }

    #[test]
    fn computes_fsrs_interval() {
        assert!((compute_next_interval(10.0) as f64 - 10.0).abs() < 1.0);
    }

    #[test]
    fn clamps_initial_difficulty_to_domain_range() {
        for grade in [1.0, 4.0] {
            assert!((1.0..=10.0).contains(&initial_difficulty(grade)));
        }
    }

    #[test]
    fn review_with_good_feedback_increases_stability() {
        let mut card = srs_data(vec![0.6, 0.7, 0.65, 0.7, 0.6]);
        card.stability = 5.0;
        card.difficulty = 5.0;
        card.review_count = 5;
        card.last_review_at = Some(DateTime::from_timestamp(1_000_000, 0).unwrap());

        let now = 1_000_000 + 5 * 24 * 3600;
        let previous_stability = card.stability;
        let next_interval_days = card
            .review(DateTime::from_timestamp(now, 0).unwrap(), 0.7)
            .unwrap();
        assert!(card.stability > previous_stability);
        assert!(next_interval_days > 0.0);
        assert!(card.next_review_at.unwrap().timestamp() > now);
        assert_eq!(card.last_review_at.unwrap().timestamp(), now);
        assert_eq!(card.review_count, 6);
        assert_eq!(card.feedback_history, vec![0.7, 0.65, 0.7, 0.6, 0.7]);
    }

    #[test]
    fn review_with_lapse_decreases_stability() {
        let mut card = srs_data(vec![0.5, 0.6, 0.7, 0.6, 0.5]);
        card.stability = 10.0;
        card.difficulty = 5.0;
        card.review_count = 5;
        card.last_review_at = Some(DateTime::from_timestamp(1_000_000, 0).unwrap());

        let previous_stability = card.stability;
        card.review(
            DateTime::from_timestamp(1_000_000 + 30 * 24 * 3600, 0).unwrap(),
            0.1,
        )
        .unwrap();
        assert!(card.stability < previous_stability);
    }

    #[test]
    fn review_handles_reset_maximum_and_feedback_boundaries() {
        let mut card = srs_data(Vec::new());
        card.stability = 50.0;
        card.difficulty = 3.0;
        card.last_review_at = Some(DateTime::from_timestamp(1_000_000, 0).unwrap());
        let now = 2_000_000;

        let now = DateTime::from_timestamp(now, 0).unwrap();
        let reset_interval = card.review(now, 0.0).unwrap();
        assert_eq!(reset_interval, 1.0);
        assert_eq!(card.stability, SrsData::INITIAL_STABILITY);
        assert_eq!(card.difficulty, SrsData::INITIAL_DIFFICULTY);
        assert_eq!(card.next_review_at, Some(now));

        let maximum_interval = card.review(now, 1.0).unwrap();
        assert_eq!(maximum_interval, MAX_INTERVAL_DAYS as f32);
        assert_eq!(
            card.next_review_at,
            Some(now + Duration::days(MAX_INTERVAL_DAYS))
        );

        assert!(card.clone().review(now, 0.2).is_ok());
        assert!(card.clone().review(now, 0.5).is_ok());
        assert!(card.clone().review(now, -0.1).is_err());
        assert!(card.clone().review(now, 1.1).is_err());
        assert!(card.clone().review(now, f32::NAN).is_err());
    }

    #[test]
    fn maximum_feedback_preserves_finite_stability_invariant() {
        let now = Utc::now();
        let mut card = srs_data(Vec::new());
        card.stability = f32::MAX;

        card.review(now, 1.0).unwrap();

        assert!(card.stability().is_finite());
        assert_eq!(card.stability(), MAX_STABILITY_DAYS);
        assert!(SrsData::new_with_state(
            card.question_id,
            card.metadata.clone(),
            card.stability(),
            card.difficulty(),
            card.next_review_at(),
            card.last_review_at(),
            card.review_count(),
            card.feedback_history().to_vec(),
        )
        .is_ok());
    }

    #[test]
    fn review_updates_and_truncates_feedback_history() {
        let now = Utc::now();
        let mut card = SrsData::new(Uuid::new_v4(), Metadata::new(now));
        for index in 0..10 {
            card.review(now + Duration::days(index + 1), index as f32 / 10.0)
                .unwrap();
        }
        assert_eq!(
            card.feedback_history.len(),
            SrsData::MAX_FEEDBACK_HISTORY_LEN
        );
        assert_eq!(card.feedback_history, vec![0.5, 0.6, 0.7, 0.8, 0.9]);
    }

    #[test]
    fn failed_review_does_not_mutate_card() {
        let now = Utc::now();
        let mut card = SrsData::new(Uuid::new_v4(), Metadata::new(now));
        let original = card.clone();
        assert!(matches!(
            card.review(now, f32::NAN),
            Err(DomainError::InvalidFeedback(InvalidFeedback { feedback }))
                if feedback.is_nan()
        ));
        assert_eq!(card, original);

        card.feedback_history = vec![1.1];
        let invalid = card.clone();
        assert_eq!(
            card.review(now, 0.5),
            Err(DomainError::InvalidFeedbackHistory(InvalidFeedbackHistory))
        );
        assert_eq!(card, invalid);
    }

    #[test]
    fn rejects_review_before_the_last_review_without_mutating_card() {
        let last_review_at = Utc::now();
        let mut card = SrsData::new(Uuid::new_v4(), Metadata::new(last_review_at));
        let original = card.clone();
        let reviewed_at = last_review_at - Duration::seconds(1);

        assert_eq!(
            card.review(reviewed_at, 0.5),
            Err(DomainError::InvalidReviewTime(InvalidReviewTime {
                reviewed_at,
                last_review_at,
            }))
        );
        assert_eq!(card, original);

        assert!(card.review(last_review_at, 0.5).is_ok());
        assert_eq!(card.last_review_at(), Some(last_review_at));
    }

    #[test]
    fn reset_restores_all_scheduling_state() {
        let now = Utc::now();
        let mut card = srs_data(vec![0.5]);
        card.reset(now);
        assert_eq!(card.stability(), SrsData::INITIAL_STABILITY);
        assert_eq!(card.difficulty(), SrsData::INITIAL_DIFFICULTY);
        assert_eq!(card.next_review_at(), Some(now));
        assert_eq!(card.last_review_at(), Some(now));
        assert_eq!(card.review_count(), 1);
        assert!(card.feedback_history().is_empty());
    }

    #[test]
    fn interpolates_review_multiplier() {
        assert!((interpolate_multiplier(2.0) - W[15]).abs() < 0.001);
        assert!((interpolate_multiplier(3.0) - 1.0).abs() < 0.001);
        assert!((interpolate_multiplier(4.0) - W[16]).abs() < 0.001);
        assert!((interpolate_multiplier(2.5) - (W[15] + 1.0) / 2.0).abs() < 0.001);
    }
}
