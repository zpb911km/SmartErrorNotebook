use chrono::Utc;
use uuid::Uuid;

use crate::domain::model::{Metadata, Question, SrsData, SyncStatus};
use crate::domain::repository::{
    QuestionRepository, RepositoryFactory, RepositoryTransactionExecutor, SrsDataRepository,
};

use super::{
    command::{CreateQuestionCommand, DeleteQuestionCommand, UpdateQuestionCommand},
    query::GetQuestionQuery,
    UseCaseError,
};

pub(crate) async fn create_question(
    executor: &impl RepositoryTransactionExecutor,
    cmd: CreateQuestionCommand,
) -> Result<Uuid, UseCaseError> {
    executor
        .execute(|factory, _| {
            Box::pin(async move {
                let id = loop {
                    let candidate_id = Uuid::new_v4();
                    if factory
                        .question_repository()
                        .find_by_id(&candidate_id)
                        .await?
                        .is_none()
                    {
                        break candidate_id;
                    }
                };
                let now = Utc::now();
                let question = Question::new(
                    id,
                    Metadata::new(now),
                    cmd.question_type,
                    cmd.source_id,
                    cmd.stem,
                    cmd.correct_answer,
                    cmd.explanation,
                    cmd.note,
                    cmd.attachment_ids,
                    cmd.tag_ids,
                );
                factory.question_repository().save(&question).await?;
                factory
                    .srs_data_repository()
                    .save(&SrsData::new(id, Metadata::new(now)))
                    .await?;
                Ok(id)
            })
        })
        .await
}

pub(crate) async fn update_question(
    executor: &impl RepositoryTransactionExecutor,
    cmd: UpdateQuestionCommand,
) -> Result<Uuid, UseCaseError> {
    executor
        .execute(|factory, _| {
            Box::pin(async move {
                let repository = factory.question_repository();
                let question = repository
                    .find_by_id(&cmd.id)
                    .await?
                    .filter(|question| question.metadata.deleted_at.is_none())
                    .ok_or(UseCaseError::NotFound("question"))?;
                let mut metadata = question.metadata;
                metadata.updated_at = Utc::now();
                metadata.sync_status = SyncStatus::Pending;
                let question = Question::new(
                    cmd.id,
                    metadata,
                    cmd.question_type,
                    cmd.source_id,
                    cmd.stem,
                    cmd.correct_answer,
                    cmd.explanation,
                    cmd.note,
                    cmd.attachment_ids,
                    cmd.tag_ids,
                );
                repository.save(&question).await?;
                Ok(question.id)
            })
        })
        .await
}

pub(crate) async fn delete_question(
    executor: &impl RepositoryTransactionExecutor,
    cmd: DeleteQuestionCommand,
) -> Result<(), UseCaseError> {
    executor
        .execute(|factory, _| {
            Box::pin(async move {
                let repository = factory.question_repository();
                if repository.find_by_id(&cmd.id).await?.is_none() {
                    return Err(UseCaseError::NotFound("question"));
                }
                repository.delete_by_id(&cmd.id).await?;
                Ok(())
            })
        })
        .await
}

pub(crate) async fn get_question(
    executor: &impl RepositoryTransactionExecutor,
    query: GetQuestionQuery,
) -> Result<Question, UseCaseError> {
    executor
        .execute(|factory, _| {
            Box::pin(async move {
                factory
                    .question_repository()
                    .find_by_id(&query.id)
                    .await?
                    .filter(|value| value.metadata.deleted_at.is_none())
                    .ok_or(UseCaseError::NotFound("question"))
            })
        })
        .await
}

pub(crate) async fn list_questions(
    executor: &impl RepositoryTransactionExecutor,
) -> Result<Vec<Question>, UseCaseError> {
    executor
        .execute(|factory, _| {
            Box::pin(async move { Ok(factory.question_repository().find_all(false).await?) })
        })
        .await
}

pub(crate) async fn count_questions(
    executor: &impl RepositoryTransactionExecutor,
) -> Result<u64, UseCaseError> {
    executor
        .execute(|factory, _| {
            Box::pin(async move { Ok(factory.question_repository().count_all(false).await?) })
        })
        .await
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use uuid::Uuid;

    use crate::application::{
        attachment,
        command::{
            CreateAttachmentCommand, DeleteAttachmentCommand, DeleteSourceCommand,
            DeleteSubjectCommand, DeleteTagCommand, SaveSourceCommand, SaveSubjectCommand,
            SaveTagCommand,
        },
        query::{
            GetAttachmentQuery, GetSourceQuery, GetSrsDataQuery, GetSubjectQuery, GetTagQuery,
            ListSourcesQuery,
        },
        source, srs_data, subject, tag, UseCaseError,
    };
    use crate::domain::model::{
        Attachment, Metadata, QuestionType, Source, SrsData, Subject, SyncStatus, Tag,
    };
    use crate::domain::repository::error::{
        EntityReference, MissingReference, Referenced, RepositoryDeleteError, RepositoryError,
        RepositorySaveError,
    };
    use crate::domain::repository::{
        AttachmentRepository, QuestionRepository, RepositoryFactory, RepositoryTransactionExecutor,
        SourceRepository, SrsDataRepository, SubjectRepository, TagRepository,
    };

    use super::super::{
        command::{CreateQuestionCommand, DeleteQuestionCommand, UpdateQuestionCommand},
        query::GetQuestionQuery,
    };
    use super::*;

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn persists_normalized_relations_and_enforces_references() {
        let executor = super::super::test_executor().await;
        let now = Utc::now();
        let subject_model = Subject {
            id: Uuid::new_v4(),
            metadata: Metadata::new(now),
            name: "Mathematics".into(),
            color: "blue".into(),
        };
        subject::save_subject(
            &executor,
            SaveSubjectCommand {
                id: subject_model.id,
                metadata: subject_model.metadata.clone(),
                name: subject_model.name.clone(),
                color: subject_model.color.clone(),
            },
        )
        .await
        .unwrap();
        let source_model = Source {
            id: Uuid::new_v4(),
            metadata: Metadata::new(now),
            subject_id: Some(subject_model.id),
            book: Some("Algebra".into()),
            chapter: Some("One".into()),
            knowledge: Some("Addition".into()),
        };
        source::save_source(
            &executor,
            SaveSourceCommand {
                id: source_model.id,
                metadata: source_model.metadata.clone(),
                subject_id: source_model.subject_id,
                book: source_model.book.clone(),
                chapter: source_model.chapter.clone(),
                knowledge: source_model.knowledge.clone(),
            },
        )
        .await
        .unwrap();
        assert_eq!(
            subject::get_subject(
                &executor,
                GetSubjectQuery {
                    id: subject_model.id,
                },
            )
            .await
            .unwrap(),
            subject_model
        );
        assert_eq!(
            source::get_source(
                &executor,
                GetSourceQuery {
                    id: source_model.id,
                },
            )
            .await
            .unwrap(),
            source_model
        );
        assert_eq!(
            source::list_sources(
                &executor,
                ListSourcesQuery {
                    subject_id: subject_model.id,
                },
            )
            .await
            .unwrap(),
            vec![source_model.clone()]
        );

        let attachment_id_1 = attachment::create_attachment(
            &executor,
            CreateAttachmentCommand {
                mime_type: "text/plain".into(),
                data: b"proof one".to_vec(),
            },
        )
        .await
        .unwrap();
        let attachment_id_2 = attachment::create_attachment(
            &executor,
            CreateAttachmentCommand {
                mime_type: "text/plain".into(),
                data: b"proof two".to_vec(),
            },
        )
        .await
        .unwrap();
        let attachment_model = attachment::get_attachment(
            &executor,
            GetAttachmentQuery {
                id: attachment_id_1,
            },
        )
        .await
        .unwrap();
        assert_eq!(attachment_model.id, attachment_id_1);
        assert_eq!(attachment_model.data, b"proof one");

        let tag_model_1 = Tag {
            id: Uuid::new_v4(),
            metadata: Metadata::new(now),
            name: "Arithmetic".into(),
            color: "red".into(),
        };
        let tag_model_2 = Tag {
            id: Uuid::new_v4(),
            metadata: Metadata::new(now),
            name: "Algebra".into(),
            color: "blue".into(),
        };
        for tag_model in [&tag_model_1, &tag_model_2] {
            tag::save_tag(
                &executor,
                SaveTagCommand {
                    id: tag_model.id,
                    metadata: tag_model.metadata.clone(),
                    name: tag_model.name.clone(),
                    color: tag_model.color.clone(),
                },
            )
            .await
            .unwrap();
        }
        assert_eq!(
            tag::get_tag(&executor, GetTagQuery { id: tag_model_1.id })
                .await
                .unwrap(),
            tag_model_1
        );

        let missing_attachment_id = Uuid::new_v4();
        let result = create_question(
            &executor,
            CreateQuestionCommand {
                question_type: None,
                source_id: Some(source_model.id),
                stem: "invalid".into(),
                correct_answer: "invalid".into(),
                explanation: None,
                note: None,
                attachment_ids: vec![missing_attachment_id, missing_attachment_id],
                tag_ids: Vec::new(),
            },
        )
        .await;
        match result {
            Err(UseCaseError::Repository(RepositoryError::Save(
                RepositorySaveError::MissingReference(MissingReference { owner, missing }),
            ))) => {
                assert_eq!(owner.entity, "question");
                assert_ne!(owner.id, Uuid::nil());
                assert_eq!(
                    missing,
                    vec![EntityReference {
                        entity: "attachment",
                        id: missing_attachment_id,
                    }]
                );
            }
            other => panic!("unexpected result: {other:?}"),
        }
        assert_eq!(count_questions(&executor).await.unwrap(), 0);

        let question_id = create_question(
            &executor,
            CreateQuestionCommand {
                question_type: Some(QuestionType::ShortAnswer),
                source_id: Some(source_model.id),
                stem: "2 + 2".into(),
                correct_answer: "4".into(),
                explanation: None,
                note: None,
                attachment_ids: vec![attachment_id_2, attachment_id_1, attachment_id_2],
                tag_ids: vec![tag_model_2.id, tag_model_1.id, tag_model_2.id],
            },
        )
        .await
        .unwrap();
        let question_model = get_question(&executor, GetQuestionQuery { id: question_id })
            .await
            .unwrap();
        let mut expected_attachment_ids = vec![attachment_id_1, attachment_id_2];
        expected_attachment_ids.sort_unstable();
        let mut expected_tag_ids = vec![tag_model_1.id, tag_model_2.id];
        expected_tag_ids.sort_unstable();
        assert_eq!(question_model.attachment_ids(), expected_attachment_ids);
        assert_eq!(question_model.tag_ids(), expected_tag_ids);
        assert_eq!(count_questions(&executor).await.unwrap(), 1);

        let initial_srs = srs_data::get_srs_data(&executor, GetSrsDataQuery { question_id })
            .await
            .unwrap();
        assert_eq!(initial_srs.stability(), SrsData::INITIAL_STABILITY);
        assert_eq!(initial_srs.difficulty(), SrsData::INITIAL_DIFFICULTY);
        assert_eq!(initial_srs.review_count(), 1);
        assert!(initial_srs.feedback_history().is_empty());

        assert_eq!(
            attachment::delete_attachment(
                &executor,
                DeleteAttachmentCommand {
                    id: attachment_id_1,
                },
            )
            .await,
            Err(UseCaseError::Repository(RepositoryError::Delete(
                RepositoryDeleteError::Referenced(Referenced {
                    target: EntityReference {
                        entity: "attachment",
                        id: attachment_id_1,
                    },
                    referenced_by: "question",
                }),
            )))
        );
        assert_eq!(
            tag::delete_tag(&executor, DeleteTagCommand { id: tag_model_1.id },).await,
            Err(UseCaseError::Repository(RepositoryError::Delete(
                RepositoryDeleteError::Referenced(Referenced {
                    target: EntityReference {
                        entity: "tag",
                        id: tag_model_1.id,
                    },
                    referenced_by: "question",
                }),
            )))
        );
        assert_eq!(
            source::delete_source(
                &executor,
                DeleteSourceCommand {
                    id: source_model.id,
                },
            )
            .await,
            Err(UseCaseError::Repository(RepositoryError::Delete(
                RepositoryDeleteError::Referenced(Referenced {
                    target: EntityReference {
                        entity: "source",
                        id: source_model.id,
                    },
                    referenced_by: "question",
                }),
            )))
        );
        assert_eq!(
            subject::delete_subject(
                &executor,
                DeleteSubjectCommand {
                    id: subject_model.id,
                },
            )
            .await,
            Err(UseCaseError::Repository(RepositoryError::Delete(
                RepositoryDeleteError::Referenced(Referenced {
                    target: EntityReference {
                        entity: "subject",
                        id: subject_model.id,
                    },
                    referenced_by: "source",
                }),
            )))
        );

        let updated_id = update_question(
            &executor,
            UpdateQuestionCommand {
                id: question_id,
                question_type: Some(QuestionType::Calculation),
                source_id: Some(source_model.id),
                stem: "2 + 3".into(),
                correct_answer: "5".into(),
                explanation: Some("addition".into()),
                note: None,
                attachment_ids: vec![attachment_id_2, attachment_id_1, attachment_id_2],
                tag_ids: vec![tag_model_2.id, tag_model_1.id, tag_model_2.id],
            },
        )
        .await
        .unwrap();
        assert_eq!(updated_id, question_id);
        let updated = get_question(&executor, GetQuestionQuery { id: question_id })
            .await
            .unwrap();
        assert_eq!(updated.attachment_ids(), expected_attachment_ids);
        assert_eq!(updated.tag_ids(), expected_tag_ids);
        assert_eq!(
            updated.metadata.created_at,
            question_model.metadata.created_at
        );
        assert_eq!(
            updated.metadata.sync_version,
            question_model.metadata.sync_version
        );
        assert!(updated.metadata.updated_at >= question_model.metadata.updated_at);
        assert_eq!(updated.metadata.sync_status, SyncStatus::Pending);
        assert_eq!(
            srs_data::get_srs_data(&executor, GetSrsDataQuery { question_id })
                .await
                .unwrap(),
            initial_srs
        );

        update_question(
            &executor,
            UpdateQuestionCommand {
                id: question_id,
                question_type: updated.question_type.clone(),
                source_id: updated.source_id,
                stem: updated.stem.clone(),
                correct_answer: updated.correct_answer.clone(),
                explanation: updated.explanation.clone(),
                note: updated.note.clone(),
                attachment_ids: Vec::new(),
                tag_ids: Vec::new(),
            },
        )
        .await
        .unwrap();
        for id in [attachment_id_1, attachment_id_2] {
            attachment::delete_attachment(&executor, DeleteAttachmentCommand { id })
                .await
                .unwrap();
        }
        for id in [tag_model_1.id, tag_model_2.id] {
            tag::delete_tag(&executor, DeleteTagCommand { id })
                .await
                .unwrap();
        }

        let mut deleted = get_question(&executor, GetQuestionQuery { id: question_id })
            .await
            .unwrap();
        deleted.metadata.deleted_at = Some(Utc::now());
        executor
            .execute(|factory, _| {
                Box::pin(async move {
                    factory.question_repository().save(&deleted).await?;
                    Ok::<_, RepositoryError>(())
                })
            })
            .await
            .unwrap();
        assert_eq!(count_questions(&executor).await.unwrap(), 0);
        delete_question(&executor, DeleteQuestionCommand { id: question_id })
            .await
            .unwrap();
        source::delete_source(
            &executor,
            DeleteSourceCommand {
                id: source_model.id,
            },
        )
        .await
        .unwrap();
        subject::delete_subject(
            &executor,
            DeleteSubjectCommand {
                id: subject_model.id,
            },
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn repositories_optionally_include_deleted_models() {
        let executor = super::super::test_executor().await;
        let now = Utc::now();
        let active_question = Question::new(
            Uuid::new_v4(),
            Metadata::new(now),
            None,
            None,
            "active".into(),
            "answer".into(),
            None,
            None,
            Vec::new(),
            Vec::new(),
        );
        let mut deleted_metadata = Metadata::new(now);
        deleted_metadata.deleted_at = Some(now);
        let deleted_question = Question::new(
            Uuid::new_v4(),
            deleted_metadata.clone(),
            None,
            None,
            "deleted".into(),
            "answer".into(),
            None,
            None,
            Vec::new(),
            Vec::new(),
        );
        let active_subject = Subject {
            id: Uuid::new_v4(),
            metadata: Metadata::new(now),
            name: "active".into(),
            color: "blue".into(),
        };
        let deleted_subject = Subject {
            id: Uuid::new_v4(),
            metadata: deleted_metadata.clone(),
            name: "deleted".into(),
            color: "gray".into(),
        };
        let active_tag = Tag {
            id: Uuid::new_v4(),
            metadata: Metadata::new(now),
            name: "active".into(),
            color: "blue".into(),
        };
        let deleted_tag = Tag {
            id: Uuid::new_v4(),
            metadata: deleted_metadata.clone(),
            name: "deleted".into(),
            color: "gray".into(),
        };
        let active_srs_data = SrsData::new_with_state(
            active_question.id,
            Metadata::new(now),
            1.0,
            1.0,
            None,
            None,
            0,
            Vec::new(),
        )
        .unwrap();
        let deleted_srs_data = SrsData::new_with_state(
            deleted_question.id,
            deleted_metadata,
            1.0,
            1.0,
            None,
            None,
            0,
            Vec::new(),
        )
        .unwrap();

        executor
            .execute(|factory, _| {
                Box::pin(async move {
                    factory.subject_repository().save(&active_subject).await?;
                    factory.subject_repository().save(&deleted_subject).await?;
                    factory.tag_repository().save(&active_tag).await?;
                    factory.tag_repository().save(&deleted_tag).await?;
                    factory.question_repository().save(&active_question).await?;
                    factory
                        .question_repository()
                        .save(&deleted_question)
                        .await?;
                    factory.srs_data_repository().save(&active_srs_data).await?;
                    factory
                        .srs_data_repository()
                        .save(&deleted_srs_data)
                        .await?;
                    Ok::<_, RepositoryError>(())
                })
            })
            .await
            .unwrap();

        assert_eq!(list_questions(&executor).await.unwrap().len(), 1);
        assert_eq!(count_questions(&executor).await.unwrap(), 1);
        executor
            .execute(|factory, _| {
                Box::pin(async move {
                    let question_repository = factory.question_repository();
                    assert_eq!(question_repository.find_all(false).await?.len(), 1);
                    assert_eq!(question_repository.find_all(true).await?.len(), 2);
                    assert_eq!(question_repository.count_all(false).await?, 1);
                    assert_eq!(question_repository.count_all(true).await?, 2);
                    assert_eq!(factory.subject_repository().find_all(false).await?.len(), 1);
                    assert_eq!(factory.subject_repository().find_all(true).await?.len(), 2);
                    assert_eq!(factory.tag_repository().find_all(false).await?.len(), 1);
                    assert_eq!(factory.tag_repository().find_all(true).await?.len(), 2);
                    assert_eq!(
                        factory.srs_data_repository().find_all(false).await?.len(),
                        1
                    );
                    assert_eq!(factory.srs_data_repository().find_all(true).await?.len(), 2);
                    Ok::<_, RepositoryError>(())
                })
            })
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn active_models_reject_deleted_references_but_tombstones_accept_them() {
        let executor = super::super::test_executor().await;
        let now = Utc::now();
        let mut deleted_metadata = Metadata::new(now);
        deleted_metadata.deleted_at = Some(now);

        let subject_id = Uuid::new_v4();
        let source_id = Uuid::new_v4();
        let attachment_id = Uuid::new_v4();
        let tag_id = Uuid::new_v4();
        let question_id = Uuid::new_v4();
        let deleted_subject = Subject {
            id: subject_id,
            metadata: deleted_metadata.clone(),
            name: "deleted subject".into(),
            color: "gray".into(),
        };
        let deleted_source = Source {
            id: source_id,
            metadata: deleted_metadata.clone(),
            subject_id: Some(subject_id),
            book: None,
            chapter: None,
            knowledge: None,
        };
        let deleted_attachment = Attachment {
            id: attachment_id,
            metadata: deleted_metadata.clone(),
            mime_type: "text/plain".into(),
            data: b"deleted".to_vec(),
            sha256: "0".repeat(64),
        };
        let deleted_tag = Tag {
            id: tag_id,
            metadata: deleted_metadata.clone(),
            name: "deleted tag".into(),
            color: "gray".into(),
        };
        let deleted_question = Question::new(
            question_id,
            deleted_metadata.clone(),
            None,
            Some(source_id),
            "deleted question".into(),
            "answer".into(),
            None,
            None,
            vec![attachment_id],
            vec![tag_id],
        );
        let deleted_srs_data = SrsData::new_with_state(
            question_id,
            deleted_metadata,
            1.0,
            1.0,
            None,
            None,
            0,
            Vec::new(),
        )
        .unwrap();

        executor
            .execute(|factory, _| {
                Box::pin(async move {
                    factory.subject_repository().save(&deleted_subject).await?;
                    factory.tag_repository().save(&deleted_tag).await?;
                    factory
                        .attachment_repository()
                        .save(&deleted_attachment)
                        .await?;
                    factory.source_repository().save(&deleted_source).await?;
                    factory
                        .question_repository()
                        .save(&deleted_question)
                        .await?;
                    factory
                        .srs_data_repository()
                        .save(&deleted_srs_data)
                        .await?;
                    Ok::<_, RepositoryError>(())
                })
            })
            .await
            .unwrap();

        let active_source_id = Uuid::new_v4();
        assert_eq!(
            source::save_source(
                &executor,
                SaveSourceCommand {
                    id: active_source_id,
                    metadata: Metadata::new(Utc::now()),
                    subject_id: Some(subject_id),
                    book: None,
                    chapter: None,
                    knowledge: None,
                },
            )
            .await,
            Err(UseCaseError::Repository(RepositoryError::Save(
                RepositorySaveError::MissingReference(MissingReference {
                    owner: EntityReference {
                        entity: "source",
                        id: active_source_id,
                    },
                    missing: vec![EntityReference {
                        entity: "subject",
                        id: subject_id,
                    }],
                }),
            )))
        );

        let result = create_question(
            &executor,
            CreateQuestionCommand {
                question_type: None,
                source_id: Some(source_id),
                stem: "active question".into(),
                correct_answer: "answer".into(),
                explanation: None,
                note: None,
                attachment_ids: vec![attachment_id],
                tag_ids: vec![tag_id],
            },
        )
        .await;
        match result {
            Err(UseCaseError::Repository(RepositoryError::Save(
                RepositorySaveError::MissingReference(MissingReference { owner, missing }),
            ))) => {
                assert_eq!(owner.entity, "question");
                assert_eq!(
                    missing,
                    vec![
                        EntityReference {
                            entity: "source",
                            id: source_id,
                        },
                        EntityReference {
                            entity: "attachment",
                            id: attachment_id,
                        },
                        EntityReference {
                            entity: "tag",
                            id: tag_id,
                        },
                    ]
                );
            }
            other => panic!("unexpected result: {other:?}"),
        }

        assert_eq!(
            srs_data::update_srs_data(
                &executor,
                crate::application::command::UpdateSrsDataCommand {
                    question_id,
                    review_feedback: 0.5,
                    reviewed_at: now,
                },
            )
            .await,
            Err(UseCaseError::Repository(RepositoryError::Save(
                RepositorySaveError::MissingReference(MissingReference {
                    owner: EntityReference {
                        entity: "srs_data",
                        id: question_id,
                    },
                    missing: vec![EntityReference {
                        entity: "question",
                        id: question_id,
                    }],
                }),
            )))
        );
    }

    #[tokio::test]
    async fn referenced_models_cannot_be_tombstoned_while_owners_are_active() {
        let executor = super::super::test_executor().await;
        let now = Utc::now();
        let subject_model = Subject {
            id: Uuid::new_v4(),
            metadata: Metadata::new(now),
            name: "subject".into(),
            color: "blue".into(),
        };
        let source_model = Source {
            id: Uuid::new_v4(),
            metadata: Metadata::new(now),
            subject_id: Some(subject_model.id),
            book: None,
            chapter: None,
            knowledge: None,
        };
        let attachment_model = Attachment {
            id: Uuid::new_v4(),
            metadata: Metadata::new(now),
            mime_type: "text/plain".into(),
            data: b"attachment".to_vec(),
            sha256: "0".repeat(64),
        };
        let tag_model = Tag {
            id: Uuid::new_v4(),
            metadata: Metadata::new(now),
            name: "tag".into(),
            color: "blue".into(),
        };
        let question_model = Question::new(
            Uuid::new_v4(),
            Metadata::new(now),
            None,
            Some(source_model.id),
            "question".into(),
            "answer".into(),
            None,
            None,
            vec![attachment_model.id],
            vec![tag_model.id],
        );

        let subject_to_save = subject_model.clone();
        let source_to_save = source_model.clone();
        let attachment_to_save = attachment_model.clone();
        let tag_to_save = tag_model.clone();
        let question_to_save = question_model.clone();
        executor
            .execute(|factory, _| {
                Box::pin(async move {
                    factory.subject_repository().save(&subject_to_save).await?;
                    factory.source_repository().save(&source_to_save).await?;
                    factory
                        .attachment_repository()
                        .save(&attachment_to_save)
                        .await?;
                    factory.tag_repository().save(&tag_to_save).await?;
                    factory
                        .question_repository()
                        .save(&question_to_save)
                        .await?;
                    Ok::<_, RepositoryError>(())
                })
            })
            .await
            .unwrap();

        let mut deleted_subject_metadata = subject_model.metadata.clone();
        deleted_subject_metadata.deleted_at = Some(Utc::now());
        assert_eq!(
            subject::save_subject(
                &executor,
                SaveSubjectCommand {
                    id: subject_model.id,
                    metadata: deleted_subject_metadata,
                    name: subject_model.name.clone(),
                    color: subject_model.color.clone(),
                },
            )
            .await,
            Err(UseCaseError::Repository(RepositoryError::Save(
                RepositorySaveError::Referenced(Referenced {
                    target: EntityReference {
                        entity: "subject",
                        id: subject_model.id,
                    },
                    referenced_by: "source",
                }),
            )))
        );

        let mut deleted_source_metadata = source_model.metadata.clone();
        deleted_source_metadata.deleted_at = Some(Utc::now());
        assert_eq!(
            source::save_source(
                &executor,
                SaveSourceCommand {
                    id: source_model.id,
                    metadata: deleted_source_metadata,
                    subject_id: source_model.subject_id,
                    book: source_model.book.clone(),
                    chapter: source_model.chapter.clone(),
                    knowledge: source_model.knowledge.clone(),
                },
            )
            .await,
            Err(UseCaseError::Repository(RepositoryError::Save(
                RepositorySaveError::Referenced(Referenced {
                    target: EntityReference {
                        entity: "source",
                        id: source_model.id,
                    },
                    referenced_by: "question",
                }),
            )))
        );

        let mut deleted_tag_metadata = tag_model.metadata.clone();
        deleted_tag_metadata.deleted_at = Some(Utc::now());
        assert_eq!(
            tag::save_tag(
                &executor,
                SaveTagCommand {
                    id: tag_model.id,
                    metadata: deleted_tag_metadata,
                    name: tag_model.name.clone(),
                    color: tag_model.color.clone(),
                },
            )
            .await,
            Err(UseCaseError::Repository(RepositoryError::Save(
                RepositorySaveError::Referenced(Referenced {
                    target: EntityReference {
                        entity: "tag",
                        id: tag_model.id,
                    },
                    referenced_by: "question",
                }),
            )))
        );

        let mut deleted_attachment = attachment_model.clone();
        deleted_attachment.metadata.deleted_at = Some(Utc::now());
        assert_eq!(
            executor
                .execute(|factory, _| {
                    Box::pin(async move {
                        factory
                            .attachment_repository()
                            .save(&deleted_attachment)
                            .await
                            .map_err(RepositoryError::from)?;
                        Ok::<_, UseCaseError>(())
                    })
                })
                .await,
            Err(UseCaseError::Repository(RepositoryError::Save(
                RepositorySaveError::Referenced(Referenced {
                    target: EntityReference {
                        entity: "attachment",
                        id: attachment_model.id,
                    },
                    referenced_by: "question",
                }),
            )))
        );

        assert!(subject::get_subject(
            &executor,
            GetSubjectQuery {
                id: subject_model.id,
            },
        )
        .await
        .is_ok());
        assert!(source::get_source(
            &executor,
            GetSourceQuery {
                id: source_model.id,
            },
        )
        .await
        .is_ok());
        assert!(tag::get_tag(&executor, GetTagQuery { id: tag_model.id })
            .await
            .is_ok());
        assert!(attachment::get_attachment(
            &executor,
            GetAttachmentQuery {
                id: attachment_model.id,
            },
        )
        .await
        .is_ok());
    }

    #[tokio::test]
    async fn loads_question_relations_across_query_batches() {
        const EXPECTED_RELATION_QUERY_BATCH_SIZE: usize = 500;

        let executor = super::super::test_executor().await;
        let now = Utc::now();
        let attachment = Attachment {
            id: Uuid::new_v4(),
            metadata: Metadata::new(now),
            mime_type: "text/plain".into(),
            data: b"attachment".to_vec(),
            sha256: "0".repeat(64),
        };
        let tag = Tag {
            id: Uuid::new_v4(),
            metadata: Metadata::new(now),
            name: "tag".into(),
            color: "blue".into(),
        };
        let questions: Vec<_> = (0..=EXPECTED_RELATION_QUERY_BATCH_SIZE)
            .map(|index| {
                let has_relations = index == 0 || index == EXPECTED_RELATION_QUERY_BATCH_SIZE;
                Question::new(
                    Uuid::new_v4(),
                    Metadata::new(now),
                    None,
                    None,
                    format!("question {index}"),
                    "answer".into(),
                    None,
                    None,
                    has_relations.then_some(attachment.id).into_iter().collect(),
                    has_relations.then_some(tag.id).into_iter().collect(),
                )
            })
            .collect();
        let first_id = questions.first().unwrap().id;
        let last_id = questions.last().unwrap().id;
        let attachment_id = attachment.id;
        let tag_id = tag.id;

        let loaded = executor
            .execute(|factory, _| {
                Box::pin(async move {
                    factory.attachment_repository().save(&attachment).await?;
                    factory.tag_repository().save(&tag).await?;
                    let repository = factory.question_repository();
                    for question in questions {
                        repository.save(&question).await?;
                    }
                    Ok::<_, RepositoryError>(repository.find_all(false).await?)
                })
            })
            .await
            .unwrap();

        assert_eq!(loaded.len(), EXPECTED_RELATION_QUERY_BATCH_SIZE + 1);
        for id in [first_id, last_id] {
            let question = loaded.iter().find(|question| question.id == id).unwrap();
            assert_eq!(question.attachment_ids(), &[attachment_id]);
            assert_eq!(question.tag_ids(), &[tag_id]);
        }
    }
}
