use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct GetSrsDataQuery {
    pub question_id: Uuid,
}
