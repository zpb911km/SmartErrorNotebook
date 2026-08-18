use super::metadata::MetadataOutput;
use crate::model::{Question, QuestionType};
use serde::Serialize;

fn legacy_question_type(value: Option<&QuestionType>) -> String {
    match value {
        Some(QuestionType::SingleSelect) => "单选题",
        Some(QuestionType::MultipleSelect) => "多选题",
        Some(QuestionType::TrueFalse) => "判断题",
        Some(QuestionType::FillInTheBlank) => "填空题",
        Some(QuestionType::ShortAnswer) => "简答题",
        Some(QuestionType::Calculation) => "计算题",
        Some(QuestionType::Essay) => "论述题",
        None => "",
    }
    .into()
}

#[derive(Serialize)]
pub struct ErrorQuestionOutput {
    pub id: String,
    pub user_id: String,
    pub subject_id: String,
    pub source_id: Option<String>,
    pub prompt: String,
    #[serde(rename = "type")]
    pub question_type: String,
    pub answer: Option<String>,
    pub analysis: Option<String>,
    pub error_note: Option<String>,
    #[serde(flatten)]
    pub metadata: MetadataOutput,
    pub userid: String,
    pub subjectid: String,
    pub sourceid: Option<String>,
    #[serde(rename = "type_")]
    pub legacy_type: String,
}
impl ErrorQuestionOutput {
    pub fn new(value: Question, subject_id: Option<String>) -> Self {
        let subject_id = subject_id.unwrap_or_default();
        let source_id = value.source_id.map(|id| id.to_string());
        let question_type = legacy_question_type(value.question_type.as_ref());
        Self {
            id: value.id.to_string(),
            user_id: String::new(),
            subject_id: subject_id.clone(),
            source_id: source_id.clone(),
            prompt: value.stem,
            question_type: question_type.clone(),
            answer: (!value.correct_answer.is_empty()).then_some(value.correct_answer),
            analysis: value.explanation,
            error_note: value.note,
            metadata: (&value.metadata).into(),
            userid: String::new(),
            subjectid: subject_id,
            sourceid: source_id,
            legacy_type: question_type,
        }
    }
}

#[derive(Serialize)]
pub struct QuestionStats {
    pub total: u64,
}
