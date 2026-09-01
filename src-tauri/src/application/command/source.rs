use uuid::Uuid;

use crate::model::Metadata;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SaveSourceCommand {
    pub id: Uuid,
    pub metadata: Metadata,
    pub subject_id: Option<Uuid>,
    pub book: Option<String>,
    pub chapter: Option<String>,
    pub knowledge: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DeleteSourceCommand {
    pub id: Uuid,
}
