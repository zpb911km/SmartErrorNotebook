use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct UpdateSrsDataCommand {
    pub question_id: Uuid,
    pub review_feedback: f32,
    pub reviewed_at: DateTime<Utc>,
}
