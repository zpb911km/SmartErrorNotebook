use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct GetQuestionQuery {
    pub id: Uuid,
}
