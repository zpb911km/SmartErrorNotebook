use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct GetSubjectQuery {
    pub id: Uuid,
}
