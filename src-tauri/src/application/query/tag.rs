use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct GetTagQuery {
    pub id: Uuid,
}
