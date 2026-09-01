use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct GetAttachmentQuery {
    pub id: Uuid,
}
