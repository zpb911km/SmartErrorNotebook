use crate::data::database::entity::subject;
use crate::model::Subject;

use super::{error::MappingError, metadata};

impl TryFrom<subject::Model> for Subject {
    type Error = MappingError;

    fn try_from(value: subject::Model) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id,
            metadata: metadata::try_from_values(
                value.created_at,
                value.updated_at,
                value.deleted_at,
                value.sync_status,
                value.sync_version,
            )?,
            name: value.name,
            color: value.color,
        })
    }
}
