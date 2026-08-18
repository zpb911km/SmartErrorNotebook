use super::database::entity::*;
use crate::model::*;
use std::str::FromStr;

impl From<attachment::Model> for Attachment {
    fn from(value: attachment::Model) -> Self {
        Attachment {
            id: value.id,
            metadata: Metadata {
                created_at: value.created_at,
                updated_at: value.updated_at,
                deleted_at: value.deleted_at,
                sync_status: SyncStatus::from_str(value.sync_status.as_str()).unwrap(),
                sync_version: value.sync_version,
            },
            mime_type: value.mime_type,
            data: value.data,
            sha256: value.sha256,
        }
    }
}

impl From<question::Model> for Question {
    fn from(value: question::Model) -> Self {
        Question {
            id: value.id,
            metadata: Metadata {
                created_at: value.created_at,
                updated_at: value.updated_at,
                deleted_at: value.deleted_at,
                sync_status: SyncStatus::from_str(value.sync_status.as_str()).unwrap(),
                sync_version: value.sync_version,
            },
            question_type: value
                .question_type
                .map(|s| QuestionType::from_str(s.as_str()).unwrap()),
            source_id: value.source_id,
            stem: value.stem,
            correct_answer: value.correct_answer,
            explanation: value.explanation,
            note: value.note,
        }
    }
}

impl From<source::Model> for Source {
    fn from(value: source::Model) -> Self {
        Source {
            id: value.id,
            metadata: Metadata {
                created_at: value.created_at,
                updated_at: value.updated_at,
                deleted_at: value.deleted_at,
                sync_status: SyncStatus::from_str(value.sync_status.as_str()).unwrap(),
                sync_version: value.sync_version,
            },
            subject_id: value.subject_id,
            book: value.book,
            chapter: value.chapter,
            knowledge: value.knowledge,
        }
    }
}

impl From<srs_data::Model> for SrsData {
    fn from(value: srs_data::Model) -> Self {
        SrsData {
            question_id: value.question_id,
            metadata: Metadata {
                created_at: value.created_at,
                updated_at: value.updated_at,
                deleted_at: value.deleted_at,
                sync_status: SyncStatus::from_str(value.sync_status.as_str()).unwrap(),
                sync_version: value.sync_version,
            },
            stability: value.stability,
            difficulty: value.difficulty,
            next_review_at: value.next_review_at,
            last_review_at: value.last_reviewed_at,
            review_count: value.review_count,
            feedback_history: value.feedback_history.to_string(),
        }
    }
}

impl From<subject::Model> for Subject {
    fn from(value: subject::Model) -> Self {
        Subject {
            id: value.id,
            metadata: Metadata {
                created_at: value.created_at,
                updated_at: value.updated_at,
                deleted_at: value.deleted_at,
                sync_status: SyncStatus::from_str(value.sync_status.as_str()).unwrap(),
                sync_version: value.sync_version,
            },
            name: value.name,
            color: value.color,
        }
    }
}

impl From<tag::Model> for Tag {
    fn from(value: tag::Model) -> Self {
        Tag {
            id: value.id,
            metadata: Metadata {
                created_at: value.created_at,
                updated_at: value.updated_at,
                deleted_at: value.deleted_at,
                sync_status: SyncStatus::from_str(value.sync_status.as_str()).unwrap(),
                sync_version: value.sync_version,
            },
            name: value.name,
            color: value.color,
        }
    }
}
