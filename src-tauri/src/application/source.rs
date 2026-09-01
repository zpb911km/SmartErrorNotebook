use crate::model::Source;
use crate::repository::{RepositoryFactory, RepositoryTransactionExecutor, SourceRepository};

use super::{
    command::{DeleteSourceCommand, SaveSourceCommand},
    query::{GetSourceQuery, ListSourcesQuery},
    UseCaseError,
};

pub(crate) async fn save_source(
    executor: &impl RepositoryTransactionExecutor,
    cmd: SaveSourceCommand,
) -> Result<(), UseCaseError> {
    executor
        .execute(|factory, _| {
            Box::pin(async move {
                factory
                    .source_repository()
                    .save(&Source {
                        id: cmd.id,
                        metadata: cmd.metadata,
                        subject_id: cmd.subject_id,
                        book: cmd.book,
                        chapter: cmd.chapter,
                        knowledge: cmd.knowledge,
                    })
                    .await?;
                Ok(())
            })
        })
        .await
}

pub(crate) async fn delete_source(
    executor: &impl RepositoryTransactionExecutor,
    cmd: DeleteSourceCommand,
) -> Result<(), UseCaseError> {
    executor
        .execute(|factory, _| {
            Box::pin(async move {
                let repository = factory.source_repository();
                if repository.find_by_id(&cmd.id).await?.is_none() {
                    return Err(UseCaseError::NotFound("source"));
                }
                repository.delete_by_id(&cmd.id).await?;
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
                factory
                    .source_repository()
                    .find_by_id(&query.id)
                    .await?
                    .filter(|value| value.metadata.deleted_at.is_none())
                    .ok_or(UseCaseError::NotFound("source"))
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
                Ok(factory
                    .source_repository()
                    .find_by_subject_id(&query.subject_id)
                    .await?
                    .into_iter()
                    .filter(|value| value.metadata.deleted_at.is_none())
                    .collect())
            })
        })
        .await
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use uuid::Uuid;

    use crate::model::{Metadata, Source};
    use crate::repository::error::{
        EntityReference, MissingReference, RepositoryError, RepositorySaveError,
    };

    use super::super::{command::SaveSourceCommand, query::GetSourceQuery};
    use super::*;

    #[tokio::test]
    async fn reports_missing_subject_details() {
        let executor = super::super::test_executor().await;
        let subject_id = Uuid::new_v4();
        let source = Source {
            id: Uuid::new_v4(),
            metadata: Metadata::new(Utc::now()),
            subject_id: Some(subject_id),
            book: None,
            chapter: None,
            knowledge: None,
        };
        assert_eq!(
            save_source(
                &executor,
                SaveSourceCommand {
                    id: source.id,
                    metadata: source.metadata.clone(),
                    subject_id: source.subject_id,
                    book: source.book.clone(),
                    chapter: source.chapter.clone(),
                    knowledge: source.knowledge.clone(),
                },
            )
            .await,
            Err(UseCaseError::Repository(RepositoryError::Save(
                RepositorySaveError::MissingReference(MissingReference {
                    owner: EntityReference {
                        entity: "source",
                        id: source.id,
                    },
                    missing: vec![EntityReference {
                        entity: "subject",
                        id: subject_id,
                    }],
                }),
            )))
        );
        assert_eq!(
            get_source(&executor, GetSourceQuery { id: source.id }).await,
            Err(UseCaseError::NotFound("source"))
        );
    }
}
