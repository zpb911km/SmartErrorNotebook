use crate::data::database::entity::attachment;
use crate::domain::model::Attachment;

use super::{error::MappingError, metadata};

impl TryFrom<attachment::Model> for Attachment {
    type Error = MappingError;

    fn try_from(value: attachment::Model) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id,
            metadata: metadata::try_from_values(
                value.created_at,
                value.updated_at,
                value.deleted_at,
                value.sync_status,
                value.sync_version,
            )?,
            mime_type: value.mime_type,
            data: value.data,
            sha256: value.sha256,
        })
    }
}
