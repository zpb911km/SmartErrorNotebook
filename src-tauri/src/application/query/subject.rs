use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GetSubjectQuery {
    ById(Uuid),
    ByIdIncludingDeleted(Uuid),
}
