// Entity 模型模块

pub mod attachment;
pub mod error_question;
pub mod error_tag;
pub mod prelude;
pub mod source;
pub mod srs_data;
pub mod subject;
pub mod user_config;

use sea_orm::Set;

use crate::domain::{self, AttachmentType, EntityMetadata, FileType, QuestionType, SyncStatus};

fn metadata(
    created_at: i64,
    updated_at: i64,
    deleted_at: Option<i64>,
    version: i32,
    sync_status: String,
    sync_hash: Option<String>,
) -> Result<EntityMetadata, domain::DomainError> {
    Ok(EntityMetadata {
        created_at,
        updated_at,
        deleted_at,
        version,
        sync_status: SyncStatus::try_from(sync_status)?,
        sync_hash,
    })
}

impl TryFrom<subject::Model> for domain::Subject {
    type Error = domain::DomainError;

    fn try_from(model: subject::Model) -> Result<Self, Self::Error> {
        Ok(Self {
            id: model.id,
            name: model.name,
            color: model.color,
            metadata: metadata(
                model.created_at,
                model.updated_at,
                model.deleted_at,
                model.version,
                model.sync_status,
                model.sync_hash,
            )?,
        })
    }
}

impl From<domain::Subject> for subject::ActiveModel {
    fn from(model: domain::Subject) -> Self {
        Self {
            id: Set(model.id),
            name: Set(model.name),
            color: Set(model.color),
            created_at: Set(model.metadata.created_at),
            updated_at: Set(model.metadata.updated_at),
            deleted_at: Set(model.metadata.deleted_at),
            version: Set(model.metadata.version),
            sync_status: Set(model.metadata.sync_status.into()),
            sync_hash: Set(model.metadata.sync_hash),
        }
    }
}

impl TryFrom<error_question::Model> for domain::ErrorQuestion {
    type Error = domain::DomainError;

    fn try_from(model: error_question::Model) -> Result<Self, Self::Error> {
        Ok(Self {
            id: model.id,
            user_id: model.userid,
            subject_id: model.subjectid,
            source_id: model.sourceid,
            prompt: model.prompt,
            question_type: QuestionType::from(model.type_),
            answer: model.answer,
            analysis: model.analysis,
            error_note: model.error_note,
            metadata: metadata(
                model.created_at,
                model.updated_at,
                model.deleted_at,
                model.version,
                model.sync_status,
                model.sync_hash,
            )?,
        })
    }
}

impl From<domain::ErrorQuestion> for error_question::ActiveModel {
    fn from(model: domain::ErrorQuestion) -> Self {
        Self {
            id: Set(model.id),
            userid: Set(model.user_id),
            subjectid: Set(model.subject_id),
            sourceid: Set(model.source_id),
            prompt: Set(model.prompt),
            type_: Set(model.question_type.into()),
            answer: Set(model.answer),
            analysis: Set(model.analysis),
            error_note: Set(model.error_note),
            created_at: Set(model.metadata.created_at),
            updated_at: Set(model.metadata.updated_at),
            deleted_at: Set(model.metadata.deleted_at),
            version: Set(model.metadata.version),
            sync_status: Set(model.metadata.sync_status.into()),
            sync_hash: Set(model.metadata.sync_hash),
        }
    }
}

impl TryFrom<error_tag::Model> for domain::ErrorTag {
    type Error = domain::DomainError;

    fn try_from(model: error_tag::Model) -> Result<Self, Self::Error> {
        Ok(Self {
            id: model.id,
            question_id: model.question_id,
            name: model.name,
            color: model.color,
            metadata: metadata(
                model.created_at,
                model.updated_at,
                model.deleted_at,
                model.version,
                model.sync_status,
                model.sync_hash,
            )?,
        })
    }
}

impl From<domain::ErrorTag> for error_tag::ActiveModel {
    fn from(model: domain::ErrorTag) -> Self {
        Self {
            id: Set(model.id),
            question_id: Set(model.question_id),
            name: Set(model.name),
            color: Set(model.color),
            created_at: Set(model.metadata.created_at),
            updated_at: Set(model.metadata.updated_at),
            deleted_at: Set(model.metadata.deleted_at),
            version: Set(model.metadata.version),
            sync_status: Set(model.metadata.sync_status.into()),
            sync_hash: Set(model.metadata.sync_hash),
        }
    }
}

impl TryFrom<source::Model> for domain::Source {
    type Error = domain::DomainError;

    fn try_from(model: source::Model) -> Result<Self, Self::Error> {
        Ok(Self {
            id: model.id,
            question_id: model.question_id,
            subject_id: model.subject_id,
            book: model.book,
            chapter: model.chapter,
            knowledge: model.knowledge,
            metadata: metadata(
                model.created_at,
                model.updated_at,
                model.deleted_at,
                model.version,
                model.sync_status,
                model.sync_hash,
            )?,
        })
    }
}

impl From<domain::Source> for source::ActiveModel {
    fn from(model: domain::Source) -> Self {
        Self {
            id: Set(model.id),
            question_id: Set(model.question_id),
            subject_id: Set(model.subject_id),
            book: Set(model.book),
            chapter: Set(model.chapter),
            knowledge: Set(model.knowledge),
            created_at: Set(model.metadata.created_at),
            updated_at: Set(model.metadata.updated_at),
            deleted_at: Set(model.metadata.deleted_at),
            version: Set(model.metadata.version),
            sync_status: Set(model.metadata.sync_status.into()),
            sync_hash: Set(model.metadata.sync_hash),
        }
    }
}

impl TryFrom<attachment::Model> for domain::Attachment {
    type Error = domain::DomainError;

    fn try_from(model: attachment::Model) -> Result<Self, Self::Error> {
        Ok(Self {
            id: model.id,
            question_id: model.question_id,
            attachment_type: AttachmentType::from(model.type_),
            file_type: FileType::from(model.file_type),
            data: model.base64_data,
            hash: model.hash,
            metadata: metadata(
                model.created_at,
                model.updated_at,
                model.deleted_at,
                model.version,
                model.sync_status,
                model.sync_hash,
            )?,
        })
    }
}

impl From<domain::Attachment> for attachment::ActiveModel {
    fn from(model: domain::Attachment) -> Self {
        Self {
            id: Set(model.id),
            question_id: Set(model.question_id),
            type_: Set(model.attachment_type.into()),
            file_type: Set(model.file_type.into()),
            base64_data: Set(model.data),
            hash: Set(model.hash),
            created_at: Set(model.metadata.created_at),
            updated_at: Set(model.metadata.updated_at),
            deleted_at: Set(model.metadata.deleted_at),
            version: Set(model.metadata.version),
            sync_status: Set(model.metadata.sync_status.into()),
            sync_hash: Set(model.metadata.sync_hash),
        }
    }
}

impl TryFrom<srs_data::Model> for domain::SrsData {
    type Error = domain::DomainError;

    fn try_from(model: srs_data::Model) -> Result<Self, Self::Error> {
        Ok(Self {
            id: model.id,
            question_id: model.question_id,
            stability: model.stability,
            difficulty: model.difficulty,
            next_review_at: model.next_review_at,
            last_review_at: model.lastreviewed_at,
            review_count: model.review_count,
            feedback_history: model.feedback_history,
            metadata: metadata(
                model.created_at,
                model.updated_at,
                model.deleted_at,
                model.version,
                model.sync_status,
                model.sync_hash,
            )?,
        })
    }
}

impl From<domain::SrsData> for srs_data::ActiveModel {
    fn from(model: domain::SrsData) -> Self {
        Self {
            id: Set(model.id),
            question_id: Set(model.question_id),
            stability: Set(model.stability),
            difficulty: Set(model.difficulty),
            next_review_at: Set(model.next_review_at),
            lastreviewed_at: Set(model.last_review_at),
            review_count: Set(model.review_count),
            feedback_history: Set(model.feedback_history),
            created_at: Set(model.metadata.created_at),
            updated_at: Set(model.metadata.updated_at),
            version: Set(model.metadata.version),
            sync_status: Set(model.metadata.sync_status.into()),
            sync_hash: Set(model.metadata.sync_hash),
            deleted_at: Set(model.metadata.deleted_at),
        }
    }
}
