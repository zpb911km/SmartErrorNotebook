use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CreateSubjectCommand {
    pub name: String,
    pub color: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct UpdateSubjectCommand {
    pub id: Uuid,
    pub name: String,
    pub color: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DeleteSubjectCommand {
    pub id: Uuid,
    pub deleted_at: DateTime<Utc>,
}
