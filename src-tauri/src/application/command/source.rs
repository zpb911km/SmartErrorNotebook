use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CreateSourceCommand {
    pub subject_id: Option<Uuid>,
    pub book: Option<String>,
    pub chapter: Option<String>,
    pub knowledge: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct UpdateSourceCommand {
    pub id: Uuid,
    pub subject_id: Option<Uuid>,
    pub book: Option<String>,
    pub chapter: Option<String>,
    pub knowledge: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DeleteSourceCommand {
    pub id: Uuid,
    pub deleted_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DeleteSourcesCommand {
    pub ids: Vec<Uuid>,
    pub deleted_at: DateTime<Utc>,
}
