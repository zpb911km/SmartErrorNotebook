use crate::domain::model::Tag;
use crate::domain::repository::{RepositoryFactory, RepositoryTransactionExecutor, TagRepository};

use super::{
    command::{DeleteTagCommand, SaveTagCommand},
    query::GetTagQuery,
    UseCaseError,
};

pub(crate) async fn save_tag(
    executor: &impl RepositoryTransactionExecutor,
    cmd: SaveTagCommand,
) -> Result<(), UseCaseError> {
    executor
        .execute(|factory, _| {
            Box::pin(async move {
                factory
                    .tag_repository()
                    .save(&Tag {
                        id: cmd.id,
                        metadata: cmd.metadata,
                        name: cmd.name,
                        color: cmd.color,
                    })
                    .await?;
                Ok(())
            })
        })
        .await
}

pub(crate) async fn delete_tag(
    executor: &impl RepositoryTransactionExecutor,
    cmd: DeleteTagCommand,
) -> Result<(), UseCaseError> {
    executor
        .execute(|factory, _| {
            Box::pin(async move {
                let repository = factory.tag_repository();
                if repository.find_by_id(&cmd.id).await?.is_none() {
                    return Err(UseCaseError::NotFound("tag"));
                }
                repository.delete_by_id(&cmd.id).await?;
                Ok(())
            })
        })
        .await
}

pub(crate) async fn get_tag(
    executor: &impl RepositoryTransactionExecutor,
    query: GetTagQuery,
) -> Result<Tag, UseCaseError> {
    executor
        .execute(|factory, _| {
            Box::pin(async move {
                factory
                    .tag_repository()
                    .find_by_id(&query.id)
                    .await?
                    .filter(|value| value.metadata.deleted_at.is_none())
                    .ok_or(UseCaseError::NotFound("tag"))
            })
        })
        .await
}

pub(crate) async fn list_tags(
    executor: &impl RepositoryTransactionExecutor,
) -> Result<Vec<Tag>, UseCaseError> {
    executor
        .execute(|factory, _| {
            Box::pin(async move { Ok(factory.tag_repository().find_all(false).await?) })
        })
        .await
}
