use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GetSrsDataQuery {
    ByQuestionId(Uuid),
}
