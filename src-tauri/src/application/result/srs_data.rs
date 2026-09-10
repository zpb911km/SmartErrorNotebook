use crate::domain::model::SrsData;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SrsReviewResult {
    pub(crate) srs: SrsData,
    pub(crate) next_interval_days: f32,
}

pub(crate) struct LibraryStatistics {
    pub(crate) question_total: u64,
    pub(crate) card_total: usize,
    pub(crate) due_count: usize,
    pub(crate) new_card_count: usize,
    pub(crate) average_stability: f32,
    pub(crate) average_difficulty: f32,
    pub(crate) total_reviews: i64,
}
