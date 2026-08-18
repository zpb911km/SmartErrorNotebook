use super::metadata::Metadata;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub struct SrsData {
    pub question_id: Uuid,
    pub metadata: Metadata,
    pub stability: f32,
    pub difficulty: f32,
    pub next_review_at: Option<DateTime<Utc>>,
    pub last_review_at: Option<DateTime<Utc>>,
    pub review_count: i64,
    pub feedback_history: String,
}
