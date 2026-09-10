use serde::Serialize;

use crate::domain::model::{Question, QuestionType};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuestionData {
    pub(crate) id: String,
    pub(crate) question_type: Option<&'static str>,
    pub(crate) stem: String,
    pub(crate) correct_answer: String,
    pub(crate) explanation: Option<String>,
    pub(crate) note: Option<String>,
    pub(crate) created_at: String,
    pub(crate) updated_at: String,
    pub(crate) source_id: Option<String>,
    pub(crate) tag_ids: Vec<String>,
    pub(crate) attachment_ids: Vec<String>,
}

impl From<Question> for QuestionData {
    fn from(value: Question) -> Self {
        Self {
            id: value.id.to_string(),
            question_type: value.question_type.as_ref().map(QuestionType::as_str),
            source_id: value.source_id.map(|id| id.to_string()),
            tag_ids: value.tag_ids().iter().map(ToString::to_string).collect(),
            attachment_ids: value
                .attachment_ids()
                .iter()
                .map(ToString::to_string)
                .collect(),
            stem: value.stem,
            correct_answer: value.correct_answer,
            explanation: value.explanation,
            note: value.note,
            created_at: value.metadata.created_at.to_rfc3339(),
            updated_at: value.metadata.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateQuestionResponse {
    pub(crate) question: QuestionData,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateQuestionResponse {
    pub(crate) question: QuestionData,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteQuestionResponse {
    pub(crate) id: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetQuestionResponse {
    pub(crate) question: QuestionData,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListQuestionsResponse {
    pub(crate) items: Vec<QuestionData>,
    pub(crate) total: usize,
}
