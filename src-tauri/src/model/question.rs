use super::macros::pub_string_enum;
use super::metadata::Metadata;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Question {
    pub id: Uuid,
    pub metadata: Metadata,
    pub question_type: Option<QuestionType>,
    pub source_id: Option<Uuid>,
    pub stem: String,
    pub correct_answer: String,
    pub explanation: Option<String>,
    pub note: Option<String>,
}

pub_string_enum!(
    QuestionType {
        SingleSelect => "SINGLE_SELECT",
        MultipleSelect => "MULTIPLE_SELECT",
        TrueFalse => "TRUE_FALSE",
        FillInTheBlank => "FILL_IN_THE_BLANK",
        ShortAnswer => "SHORT_ANSWER",
        Calculation => "CALCULATION",
        Essay => "ESSAY",
    }
);
