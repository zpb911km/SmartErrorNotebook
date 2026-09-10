use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
// The shared `By` prefix makes the lookup criterion explicit at call sites.
#[allow(clippy::enum_variant_names)]
pub(crate) enum GetSourceQuery {
    ById(Uuid),
    ByIdIncludingDeleted(Uuid),
    ByAttributes {
        subject_id: Option<Uuid>,
        book: Option<String>,
        chapter: Option<String>,
        knowledge: Option<String>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ListSourcesQuery {
    All,
    BySubjectId(Uuid),
}
