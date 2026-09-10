use uuid::Uuid;

use super::macros::pub_string_enum;
use super::metadata::Metadata;

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
    /// Attachment relations must be changed through `set_attachment_ids` so IDs remain unique.
    attachment_ids: Vec<Uuid>,
    /// Tag relations must be changed through `set_tag_ids` so IDs remain unique.
    tag_ids: Vec<Uuid>,
}
impl Question {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        id: Uuid,
        metadata: Metadata,
        question_type: Option<QuestionType>,
        source_id: Option<Uuid>,
        stem: String,
        correct_answer: String,
        explanation: Option<String>,
        note: Option<String>,
        attachment_ids: Vec<Uuid>,
        tag_ids: Vec<Uuid>,
    ) -> Self {
        let mut question = Self {
            id,
            metadata,
            question_type,
            source_id,
            stem,
            correct_answer,
            explanation,
            note,
            attachment_ids: Vec::new(),
            tag_ids: Vec::new(),
        };
        question.set_attachment_ids(attachment_ids);
        question.set_tag_ids(tag_ids);
        question
    }

    pub(crate) fn attachment_ids(&self) -> &[Uuid] {
        &self.attachment_ids
    }

    pub(crate) fn set_attachment_ids(&mut self, mut attachment_ids: Vec<Uuid>) {
        attachment_ids.sort_unstable();
        attachment_ids.dedup();
        self.attachment_ids = attachment_ids;
    }

    pub(crate) fn tag_ids(&self) -> &[Uuid] {
        &self.tag_ids
    }

    pub(crate) fn set_tag_ids(&mut self, mut tag_ids: Vec<Uuid>) {
        tag_ids.sort_unstable();
        tag_ids.dedup();
        self.tag_ids = tag_ids;
    }
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

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;

    #[test]
    fn new_normalizes_relation_ids() {
        let attachment_id_1 = Uuid::new_v4();
        let attachment_id_2 = Uuid::new_v4();
        let tag_id_1 = Uuid::new_v4();
        let tag_id_2 = Uuid::new_v4();
        let question = Question::new(
            Uuid::new_v4(),
            Metadata::new(Utc::now()),
            Some(QuestionType::ShortAnswer),
            None,
            "stem".into(),
            "answer".into(),
            None,
            None,
            vec![attachment_id_2, attachment_id_1, attachment_id_2],
            vec![tag_id_2, tag_id_1, tag_id_2],
        );
        let mut expected_attachment_ids = vec![attachment_id_1, attachment_id_2];
        expected_attachment_ids.sort_unstable();
        let mut expected_tag_ids = vec![tag_id_1, tag_id_2];
        expected_tag_ids.sort_unstable();
        assert_eq!(question.attachment_ids(), expected_attachment_ids);
        assert_eq!(question.tag_ids(), expected_tag_ids);
        assert_eq!(question.stem, "stem");
        assert_eq!(question.correct_answer, "answer");
    }

    #[test]
    fn relation_setters_normalize_ids() {
        let attachment_id = Uuid::new_v4();
        let tag_id = Uuid::new_v4();
        let mut question = Question::new(
            Uuid::new_v4(),
            Metadata::new(Utc::now()),
            None,
            None,
            "stem".into(),
            "answer".into(),
            None,
            None,
            Vec::new(),
            Vec::new(),
        );
        question.set_attachment_ids(vec![attachment_id, attachment_id]);
        question.set_tag_ids(vec![tag_id, tag_id]);
        assert_eq!(question.attachment_ids(), &[attachment_id]);
        assert_eq!(question.tag_ids(), &[tag_id]);
    }

    #[test]
    fn question_type_parses_through_its_inherent_method() {
        assert_eq!(
            QuestionType::from_str("SHORT_ANSWER"),
            Ok(QuestionType::ShortAnswer)
        );
        assert!(QuestionType::from_str("UNSUPPORTED").is_err());
    }
}
