use chrono::Utc;
use uuid::Uuid;

use crate::domain::model::{Metadata, Tag};
use crate::domain::repository::{RepositoryFactory, RepositoryTransactionExecutor, TagRepository};

use super::{
    command::{CreateTagCommand, DeleteTagCommand, UpdateTagCommand},
    query::{GetTagQuery, ListTagsQuery},
    UseCaseError,
};

pub(crate) async fn create_tag(
    executor: &impl RepositoryTransactionExecutor,
    cmd: CreateTagCommand,
) -> Result<Tag, UseCaseError> {
    executor
        .execute(|factory, _| {
            Box::pin(async move {
                let repository = factory.tag_repository();
                let id = loop {
                    let candidate = Uuid::new_v4();
                    if repository.find_by_id(&candidate, true).await?.is_none() {
                        break candidate;
                    }
                };
                let tag = Tag {
                    id,
                    metadata: Metadata::new(Utc::now()),
                    name: cmd.name,
                    color: cmd.color,
                };
                repository.save(&tag).await?;
                Ok(tag)
            })
        })
        .await
}

pub(crate) async fn update_tag(
    executor: &impl RepositoryTransactionExecutor,
    cmd: UpdateTagCommand,
) -> Result<Tag, UseCaseError> {
    executor
        .execute(|factory, _| {
            Box::pin(async move {
                let repository = factory.tag_repository();
                let mut tag =
                    repository
                        .find_by_id(&cmd.id, false)
                        .await?
                        .ok_or(UseCaseError::NotFound {
                            entity: "tag",
                            id: Some(cmd.id),
                        })?;
                tag.name = cmd.name;
                tag.color = cmd.color;
                tag.metadata.touch(Utc::now());
                repository.save(&tag).await?;
                Ok(tag)
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
                let mut tag =
                    repository
                        .find_by_id(&cmd.id, false)
                        .await?
                        .ok_or(UseCaseError::NotFound {
                            entity: "tag",
                            id: Some(cmd.id),
                        })?;
                tag.metadata.mark_as_deleted(Utc::now());
                repository.save(&tag).await?;
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
                match query {
                    GetTagQuery::ById(id) => factory
                        .tag_repository()
                        .find_by_id(&id, false)
                        .await?
                        .ok_or(UseCaseError::NotFound {
                            entity: "tag",
                            id: Some(id),
                        }),
                    GetTagQuery::ByIdIncludingDeleted(id) => {
                        factory.tag_repository().find_by_id(&id, true).await?.ok_or(
                            UseCaseError::NotFound {
                                entity: "tag",
                                id: Some(id),
                            },
                        )
                    }
                }
            })
        })
        .await
}

pub(crate) async fn list_tags(
    executor: &impl RepositoryTransactionExecutor,
    query: ListTagsQuery,
) -> Result<Vec<Tag>, UseCaseError> {
    executor
        .execute(|factory, _| {
            Box::pin(async move {
                let repository = factory.tag_repository();
                match query {
                    ListTagsQuery::All => Ok(repository.find_all(false).await?),
                    ListTagsQuery::ByAttribute { name, color } => Ok(repository
                        .find_all(false)
                        .await?
                        .into_iter()
                        .filter(|tag| tag.name == name && tag.color == color)
                        .collect()),
                    ListTagsQuery::ByQuestionId(question_id) => {
                        Ok(repository.find_by_question_id(&question_id, false).await?)
                    }
                }
            })
        })
        .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::command::{CreateTagCommand, UpdateTagCommand};

    #[tokio::test]
    async fn attribute_queries_do_not_create_tags() {
        let executor = super::super::test_executor().await;
        create_tag(
            &executor,
            CreateTagCommand {
                name: "existing".into(),
                color: "blue".into(),
            },
        )
        .await
        .unwrap();

        assert!(list_tags(
            &executor,
            ListTagsQuery::ByAttribute {
                name: "missing".into(),
                color: "red".into(),
            },
        )
        .await
        .unwrap()
        .is_empty());
        assert_eq!(
            list_tags(&executor, ListTagsQuery::All)
                .await
                .unwrap()
                .len(),
            1
        );
    }

    #[tokio::test]
    async fn update_cannot_revive_a_deleted_tag() {
        let executor = super::super::test_executor().await;
        let created = create_tag(
            &executor,
            CreateTagCommand {
                name: "tag".into(),
                color: "#000000".into(),
            },
        )
        .await
        .unwrap();
        delete_tag(&executor, DeleteTagCommand { id: created.id })
            .await
            .unwrap();
        assert_eq!(
            update_tag(
                &executor,
                UpdateTagCommand {
                    id: created.id,
                    name: "revived".into(),
                    color: String::new(),
                },
            )
            .await,
            Err(UseCaseError::NotFound {
                entity: "tag",
                id: Some(created.id),
            })
        );
    }
}
