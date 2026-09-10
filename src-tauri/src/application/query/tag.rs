use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GetTagQuery {
    ById(Uuid),
    ByIdIncludingDeleted(Uuid),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ListTagsQuery {
    All,
    ByAttribute { name: String, color: String },
    ByQuestionId(Uuid),
}
