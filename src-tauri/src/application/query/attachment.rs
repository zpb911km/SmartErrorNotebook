use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GetAttachmentQuery {
    ById(Uuid),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ListAttachmentsQuery {
    ByQuestionId(Uuid),
}
