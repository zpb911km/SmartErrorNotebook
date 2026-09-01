use super::metadata::Metadata;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attachment {
    pub id: Uuid,
    pub metadata: Metadata,
    pub mime_type: String,
    pub data: Vec<u8>,
    pub sha256: String,
}
