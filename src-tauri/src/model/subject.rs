use super::metadata::Metadata;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Subject {
    pub id: Uuid,
    pub metadata: Metadata,
    pub name: String,
    pub color: String,
}
