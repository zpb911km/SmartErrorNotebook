use chrono::Utc;
use serde_json::json;
use uuid::Uuid;

use crate::data::database::entity::{attachment, question, srs_data};
use crate::domain::model::error::DomainError;
use crate::domain::model::{Attachment, SrsData};

use super::error::MappingError;

fn persisted_srs_data() -> srs_data::Model {
    let now = Utc::now();
    srs_data::Model {
        question_id: Uuid::new_v4(),
        created_at: now,
        updated_at: now,
        deleted_at: None,
        sync_status: "PENDING".into(),
        sync_version: 0,
        stability: 3.0,
        difficulty: 5.0,
        next_review_at: None,
        last_reviewed_at: None,
        review_count: 0,
        feedback_history: json!([]),
    }
}

#[test]
fn rejects_invalid_persisted_enum_values() {
    let now = Utc::now();
    let attachment = attachment::Model {
        id: Uuid::new_v4(),
        created_at: now,
        updated_at: now,
        deleted_at: None,
        sync_status: "INVALID".into(),
        sync_version: 0,
        mime_type: "text/plain".into(),
        data: Vec::new(),
        sha256: String::new(),
    };
    assert!(matches!(
        Attachment::try_from(attachment),
        Err(MappingError::StringToEnum("invalid value"))
    ));

    let question = question::Model {
        id: Uuid::new_v4(),
        created_at: now,
        updated_at: now,
        deleted_at: None,
        sync_status: "PENDING".into(),
        sync_version: 0,
        question_type: Some("INVALID".into()),
        source_id: None,
        stem: String::new(),
        correct_answer: String::new(),
        explanation: None,
        note: None,
    };
    assert!(matches!(
        question.try_into_question(Vec::new(), Vec::new()),
        Err(MappingError::StringToEnum("invalid value"))
    ));
}

#[test]
fn rejects_malformed_persisted_srs_feedback_json() {
    let mut model = persisted_srs_data();
    model.feedback_history = json!({"unexpected": true});
    assert!(matches!(
        SrsData::try_from(model),
        Err(MappingError::DeserializeFeedbackHistory(_))
    ));
}

#[test]
fn rejects_persisted_srs_state_outside_domain_invariants() {
    let mut model = persisted_srs_data();
    model.stability = -1.0;
    assert!(matches!(
        SrsData::try_from(model),
        Err(MappingError::Domain(DomainError::InvalidSrsState(_)))
    ));

    let mut model = persisted_srs_data();
    model.feedback_history = json!([1.1]);
    assert!(matches!(
        SrsData::try_from(model),
        Err(MappingError::Domain(DomainError::InvalidFeedbackHistory(_)))
    ));
}

#[test]
fn normalizes_legacy_srs_state_before_domain_construction() {
    let mut model = persisted_srs_data();
    model.stability = -1.0;
    model.difficulty = 0.3;
    model.review_count = -1;
    model.feedback_history = json!([-0.1, 0.5, 1.1]);

    let data = model.try_into_legacy_srs_data().unwrap();
    assert_eq!(data.stability(), SrsData::INITIAL_STABILITY);
    assert_eq!(data.difficulty(), SrsData::INITIAL_DIFFICULTY);
    assert_eq!(data.review_count(), 0);
    assert_eq!(data.feedback_history(), &[0.5]);
}
