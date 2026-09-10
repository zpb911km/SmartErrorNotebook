use crate::domain::model::Question;

pub(crate) struct ListQuestionsResult {
    pub(crate) items: Vec<Question>,
    pub(crate) total: usize,
}
