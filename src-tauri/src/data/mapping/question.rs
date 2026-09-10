use uuid::Uuid;

use crate::data::database::entity::question;
use crate::domain::model::{Question, QuestionType};

use super::{error::MappingError, metadata};

impl question::Model {
    pub(crate) fn try_into_question(
        self,
        attachment_ids: Vec<Uuid>,
        tag_ids: Vec<Uuid>,
    ) -> Result<Question, MappingError> {
        let question_type = self
            .question_type
            .map(|value| QuestionType::from_str(&value).map_err(MappingError::StringToEnum))
            .transpose()?;
        Ok(Question::new(
            self.id,
            metadata::try_from_values(
                self.created_at,
                self.updated_at,
                self.deleted_at,
                self.sync_status,
                self.sync_version,
            )?,
            question_type,
            self.source_id,
            self.stem,
            self.correct_answer,
            self.explanation,
            self.note,
            attachment_ids,
            tag_ids,
        ))
    }
}
