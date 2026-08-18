use super::metadata::Metadata;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Source {
    pub id: Uuid,
    pub metadata: Metadata,
    pub subject_id: Option<Uuid>,
    pub book: Option<String>,
    pub chapter: Option<String>,
    pub knowledge: Option<String>,
}
