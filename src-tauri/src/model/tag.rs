use super::metadata::Metadata;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tag {
    pub id: Uuid,
    pub metadata: Metadata,
    pub name: String,
    pub color: String,
}
