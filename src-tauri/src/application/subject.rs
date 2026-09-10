use chrono::Utc;
use uuid::Uuid;

use crate::domain::model::{Metadata, Subject};
use crate::domain::repository::{
    RepositoryFactory, RepositoryTransactionExecutor, SourceRepository, SubjectRepository,
};

use super::{
    command::{CreateSubjectCommand, DeleteSubjectCommand, UpdateSubjectCommand},
    query::GetSubjectQuery,
    UseCaseError,
};

pub(crate) async fn create_subject(
    executor: &impl RepositoryTransactionExecutor,
    cmd: CreateSubjectCommand,
) -> Result<Subject, UseCaseError> {
    executor
        .execute(|factory, _| {
            Box::pin(async move {
                let repository = factory.subject_repository();
                let id = loop {
                    let candidate = Uuid::new_v4();
                    if repository.find_by_id(&candidate, true).await?.is_none() {
                        break candidate;
                    }
                };
                let subject = Subject {
                    id,
                    metadata: Metadata::new(Utc::now()),
                    name: cmd.name,
                    color: cmd.color,
                };
                repository.save(&subject).await?;
                Ok(subject)
            })
        })
        .await
}

pub(crate) async fn update_subject(
    executor: &impl RepositoryTransactionExecutor,
    cmd: UpdateSubjectCommand,
) -> Result<Subject, UseCaseError> {
    executor
        .execute(|factory, _| {
            Box::pin(async move {
                let repository = factory.subject_repository();
                let mut subject =
                    repository
                        .find_by_id(&cmd.id, false)
                        .await?
                        .ok_or(UseCaseError::NotFound {
                            entity: "subject",
                            id: Some(cmd.id),
                        })?;
                subject.name = cmd.name;
                subject.color = cmd.color;
                subject.metadata.touch(Utc::now());
                repository.save(&subject).await?;
                Ok(subject)
            })
        })
        .await
}

pub(crate) async fn delete_subject(
    executor: &impl RepositoryTransactionExecutor,
    cmd: DeleteSubjectCommand,
) -> Result<(), UseCaseError> {
    executor
        .execute(|factory, _| {
            Box::pin(async move {
                let mut subject = factory
                    .subject_repository()
                    .find_by_id(&cmd.id, false)
                    .await?
                    .ok_or(UseCaseError::NotFound {
                        entity: "subject",
                        id: Some(cmd.id),
                    })?;
                let mut sources = factory
                    .source_repository()
                    .find_by_subject_id(&cmd.id, false)
                    .await?;
                for source in &mut sources {
                    source.subject_id = None;
                    source.metadata.touch(cmd.deleted_at);
                    factory.source_repository().save(source).await?;
                }
                subject.metadata.mark_as_deleted(cmd.deleted_at);
                factory.subject_repository().save(&subject).await?;
                Ok(())
            })
        })
        .await
}

pub(crate) async fn get_subject(
    executor: &impl RepositoryTransactionExecutor,
    query: GetSubjectQuery,
) -> Result<Subject, UseCaseError> {
    executor
        .execute(|factory, _| {
            Box::pin(async move {
                match query {
                    GetSubjectQuery::ById(id) => factory
                        .subject_repository()
                        .find_by_id(&id, false)
                        .await?
                        .ok_or(UseCaseError::NotFound {
                            entity: "subject",
                            id: Some(id),
                        }),
                    GetSubjectQuery::ByIdIncludingDeleted(id) => factory
                        .subject_repository()
                        .find_by_id(&id, true)
                        .await?
                        .ok_or(UseCaseError::NotFound {
                            entity: "subject",
                            id: Some(id),
                        }),
                }
            })
        })
        .await
}

pub(crate) async fn list_subjects(
    executor: &impl RepositoryTransactionExecutor,
) -> Result<Vec<Subject>, UseCaseError> {
    executor
        .execute(|factory, _| {
            Box::pin(async move { Ok(factory.subject_repository().find_all(false).await?) })
        })
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn update_is_atomic_and_cannot_revive_a_deleted_subject() {
        let executor = super::super::test_executor().await;
        let created = create_subject(
            &executor,
            CreateSubjectCommand {
                name: "original".into(),
                color: "#000000".into(),
            },
        )
        .await
        .unwrap();
        let updated = update_subject(
            &executor,
            UpdateSubjectCommand {
                id: created.id,
                name: "updated".into(),
                color: "#ffffff".into(),
            },
        )
        .await
        .unwrap();
        assert_eq!(updated.name, "updated");

        delete_subject(
            &executor,
            DeleteSubjectCommand {
                id: created.id,
                deleted_at: Utc::now(),
            },
        )
        .await
        .unwrap();
        assert_eq!(
            update_subject(
                &executor,
                UpdateSubjectCommand {
                    id: created.id,
                    name: "revived".into(),
                    color: String::new(),
                },
            )
            .await,
            Err(UseCaseError::NotFound {
                entity: "subject",
                id: Some(created.id),
            })
        );
    }
}
