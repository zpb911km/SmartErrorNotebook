use uuid::Uuid;

use crate::domain::model::QuestionType;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CreateQuestionCommand {
    pub question_type: Option<QuestionType>,
    pub source_id: Option<Uuid>,
    pub stem: String,
    pub correct_answer: String,
    pub explanation: Option<String>,
    pub note: Option<String>,
    pub attachment_ids: Vec<Uuid>,
    pub tag_ids: Vec<Uuid>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct UpdateQuestionCommand {
    pub id: Uuid,
    pub question_type: Option<QuestionType>,
    pub source_id: Option<Uuid>,
    pub stem: String,
    pub correct_answer: String,
    pub explanation: Option<String>,
    pub note: Option<String>,
    pub attachment_ids: Vec<Uuid>,
    pub tag_ids: Vec<Uuid>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DeleteQuestionCommand {
    pub id: Uuid,
}
