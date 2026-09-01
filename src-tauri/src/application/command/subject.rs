use uuid::Uuid;

use crate::model::Metadata;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SaveSubjectCommand {
    pub id: Uuid,
    pub metadata: Metadata,
    pub name: String,
    pub color: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DeleteSubjectCommand {
    pub id: Uuid,
}
