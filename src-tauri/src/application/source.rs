use chrono::Utc;
use uuid::Uuid;

use crate::domain::model::{Metadata, Source};
use crate::domain::repository::{
    QuestionRepository, RepositoryFactory, RepositoryTransactionExecutor, SourceRepository,
    SubjectRepository,
};

use super::{
    command::{
        CreateSourceCommand, DeleteSourceCommand, DeleteSourcesCommand, UpdateSourceCommand,
    },
    query::{GetSourceQuery, ListSourcesQuery, QuestionFilter},
    UseCaseError,
};

pub(crate) async fn create_source(
    executor: &impl RepositoryTransactionExecutor,
    cmd: CreateSourceCommand,
) -> Result<Source, UseCaseError> {
    executor
        .execute(|factory, _| {
            Box::pin(async move {
                let repository = factory.source_repository();
                let id = loop {
                    let candidate = Uuid::new_v4();
                    if repository.find_by_id(&candidate, true).await?.is_none() {
                        break candidate;
                    }
                };
                let source = Source {
                    id,
                    metadata: Metadata::new(Utc::now()),
                    subject_id: cmd.subject_id,
                    book: cmd.book,
                    chapter: cmd.chapter,
                    knowledge: cmd.knowledge,
                };
                repository.save(&source).await?;
                Ok(source)
            })
        })
        .await
}

pub(crate) async fn update_source(
    executor: &impl RepositoryTransactionExecutor,
    cmd: UpdateSourceCommand,
) -> Result<Source, UseCaseError> {
    executor
        .execute(|factory, _| {
            Box::pin(async move {
                let repository = factory.source_repository();
                let mut source =
                    repository
                        .find_by_id(&cmd.id, false)
                        .await?
                        .ok_or(UseCaseError::NotFound {
                            entity: "source",
                            id: Some(cmd.id),
                        })?;
                source.subject_id = cmd.subject_id;
                source.book = cmd.book;
                source.chapter = cmd.chapter;
                source.knowledge = cmd.knowledge;
                source.metadata.touch(Utc::now());
                repository.save(&source).await?;
                Ok(source)
            })
        })
        .await
}

pub(crate) async fn delete_source(
    executor: &impl RepositoryTransactionExecutor,
    cmd: DeleteSourceCommand,
) -> Result<(), UseCaseError> {
    delete_sources(
        executor,
        DeleteSourcesCommand {
            ids: vec![cmd.id],
            deleted_at: cmd.deleted_at,
        },
    )
    .await
}

pub(crate) async fn delete_sources(
    executor: &impl RepositoryTransactionExecutor,
    cmd: DeleteSourcesCommand,
) -> Result<(), UseCaseError> {
    executor
        .execute(|factory, _| {
            Box::pin(async move {
                let mut ids = cmd.ids;
                ids.sort_unstable();
                ids.dedup();
                if ids.is_empty() {
                    return Ok(());
                }

                // Validate the whole batch before changing anything.
                let mut sources = Vec::with_capacity(ids.len());
                for id in &ids {
                    sources.push(
                        factory
                            .source_repository()
                            .find_by_id(id, false)
                            .await?
                            .ok_or(UseCaseError::NotFound {
                                entity: "source",
                                id: Some(*id),
                            })?,
                    );
                }

                let deleted_at = cmd.deleted_at;
                let source_ids = ids.iter().copied().collect();
                let mut questions = factory
                    .question_repository()
                    .find_filtered(
                        QuestionFilter {
                            source_ids,
                            ..Default::default()
                        },
                        Vec::new(),
                        None,
                        None,
                    )
                    .await?;
                for question in &mut questions {
                    question.source_id = None;
                    question.metadata.touch(deleted_at);
                    factory.question_repository().save(question).await?;
                }
                for source in &mut sources {
                    source.metadata.mark_as_deleted(deleted_at);
                    factory.source_repository().save(source).await?;
                }
                Ok(())
            })
        })
        .await
}

pub(crate) async fn get_source(
    executor: &impl RepositoryTransactionExecutor,
    query: GetSourceQuery,
) -> Result<Source, UseCaseError> {
    executor
        .execute(|factory, _| {
            Box::pin(async move {
                match query {
                    GetSourceQuery::ById(id) => factory
                        .source_repository()
                        .find_by_id(&id, false)
                        .await?
                        .ok_or(UseCaseError::NotFound {
                            entity: "source",
                            id: Some(id),
                        }),
                    GetSourceQuery::ByIdIncludingDeleted(id) => factory
                        .source_repository()
                        .find_by_id(&id, true)
                        .await?
                        .ok_or(UseCaseError::NotFound {
                            entity: "source",
                            id: Some(id),
                        }),
                    GetSourceQuery::ByAttributes {
                        subject_id,
                        book,
                        chapter,
                        knowledge,
                    } => {
                        if let Some(subject_id) = subject_id {
                            if factory
                                .subject_repository()
                                .find_by_id(&subject_id, false)
                                .await?
                                .is_none()
                            {
                                return Err(UseCaseError::NotFound {
                                    entity: "subject",
                                    id: Some(subject_id),
                                });
                            }
                        }
                        let repository = factory.source_repository();
                        if let Some(found) =
                            repository.find_all(false).await?.into_iter().find(|value| {
                                value.subject_id == subject_id
                                    && value.book == book
                                    && value.chapter == chapter
                                    && value.knowledge == knowledge
                            })
                        {
                            return Ok(found);
                        }
                        Err(UseCaseError::NotFound {
                            entity: "source",
                            id: None,
                        })
                    }
                }
            })
        })
        .await
}

pub(crate) async fn list_sources(
    executor: &impl RepositoryTransactionExecutor,
    query: ListSourcesQuery,
) -> Result<Vec<Source>, UseCaseError> {
    executor
        .execute(|factory, _| {
            Box::pin(async move {
                let repository = factory.source_repository();
                Ok(match query {
                    ListSourcesQuery::All => repository.find_all(false).await?,
                    ListSourcesQuery::BySubjectId(subject_id) => {
                        repository.find_by_subject_id(&subject_id, false).await?
                    }
                })
            })
        })
        .await
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use uuid::Uuid;

    use crate::domain::repository::error::{
        EntityReference, MissingReference, RepositoryError, RepositorySaveError,
    };

    use super::super::{
        command::{CreateSourceCommand, UpdateSourceCommand},
        query::GetSourceQuery,
    };
    use super::*;

    #[tokio::test]
    async fn reports_missing_subject_details() {
        let executor = super::super::test_executor().await;
        let subject_id = Uuid::new_v4();
        let result = create_source(
            &executor,
            CreateSourceCommand {
                subject_id: Some(subject_id),
                book: None,
                chapter: None,
                knowledge: None,
            },
        )
        .await;
        match result {
            Err(UseCaseError::Repository(RepositoryError::Save(
                RepositorySaveError::MissingReference(MissingReference { owner, missing }),
            ))) => {
                assert_eq!(owner.entity, "source");
                assert_ne!(owner.id, Uuid::nil());
                assert_eq!(
                    missing,
                    vec![EntityReference {
                        entity: "subject",
                        id: subject_id,
                    }]
                );
            }
            other => panic!("unexpected result: {other:?}"),
        }
        assert_eq!(
            get_source(
                &executor,
                GetSourceQuery::ByAttributes {
                    subject_id: None,
                    book: Some("missing".into()),
                    chapter: None,
                    knowledge: None,
                },
            )
            .await,
            Err(UseCaseError::NotFound {
                entity: "source",
                id: None
            })
        );
        assert!(list_sources(&executor, ListSourcesQuery::All)
            .await
            .unwrap()
            .is_empty());
    }

    #[tokio::test]
    async fn update_cannot_revive_a_deleted_source() {
        let executor = super::super::test_executor().await;
        let created = create_source(
            &executor,
            CreateSourceCommand {
                subject_id: None,
                book: Some("book".into()),
                chapter: None,
                knowledge: None,
            },
        )
        .await
        .unwrap();
        delete_source(
            &executor,
            DeleteSourceCommand {
                id: created.id,
                deleted_at: Utc::now(),
            },
        )
        .await
        .unwrap();
        assert_eq!(
            update_source(
                &executor,
                UpdateSourceCommand {
                    id: created.id,
                    subject_id: None,
                    book: Some("revived".into()),
                    chapter: None,
                    knowledge: None,
                },
            )
            .await,
            Err(UseCaseError::NotFound {
                entity: "source",
                id: Some(created.id),
            })
        );
    }
}
