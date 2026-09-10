use uuid::Uuid;

pub(crate) use crate::domain::repository::{QuestionFilter, QuestionSort, ReviewState};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GetQuestionQuery {
    ById(Uuid),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ListQuestionsQuery {
    All,
    Filtered {
        filter: Box<QuestionFilter>,
        sort: Vec<QuestionSort>,
        offset: Option<usize>,
        limit: Option<usize>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CountQuestionsQuery {
    All,
    Filtered(Box<QuestionFilter>),
}
