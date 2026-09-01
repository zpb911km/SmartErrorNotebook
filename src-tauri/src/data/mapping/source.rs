use crate::data::database::entity::source;
use crate::model::Source;

use super::{error::MappingError, metadata};

impl TryFrom<source::Model> for Source {
    type Error = MappingError;

    fn try_from(value: source::Model) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id,
            metadata: metadata::try_from_values(
                value.created_at,
                value.updated_at,
                value.deleted_at,
                value.sync_status,
                value.sync_version,
            )?,
            subject_id: value.subject_id,
            book: value.book,
            chapter: value.chapter,
            knowledge: value.knowledge,
        })
    }
}
