use chrono::Utc;
use uuid::Uuid;

use crate::domain::model::{Attachment, Metadata};
use crate::domain::repository::{
    AttachmentRepository, RepositoryFactory, RepositoryTransactionExecutor,
};
use crate::util::codec::sha256;

use super::{
    command::{CreateAttachmentCommand, DeleteAttachmentCommand},
    query::{GetAttachmentQuery, ListAttachmentsQuery},
    UseCaseError,
};

pub(crate) async fn create_attachment(
    executor: &impl RepositoryTransactionExecutor,
    cmd: CreateAttachmentCommand,
) -> Result<Uuid, UseCaseError> {
    executor
        .execute(|factory, _| {
            Box::pin(async move {
                let repository = factory.attachment_repository();
                let id = loop {
                    let candidate_id = Uuid::new_v4();
                    if repository.find_by_id(&candidate_id, true).await?.is_none() {
                        break candidate_id;
                    }
                };
                repository
                    .save(&Attachment {
                        id,
                        metadata: Metadata::new(Utc::now()),
                        mime_type: cmd.mime_type,
                        sha256: sha256(&cmd.data),
                        data: cmd.data,
                    })
                    .await?;
                Ok(id)
            })
        })
        .await
}

pub(crate) async fn delete_attachment(
    executor: &impl RepositoryTransactionExecutor,
    cmd: DeleteAttachmentCommand,
) -> Result<(), UseCaseError> {
    executor
        .execute(|factory, _| {
            Box::pin(async move {
                let repository = factory.attachment_repository();
                let mut attachment =
                    repository
                        .find_by_id(&cmd.id, false)
                        .await?
                        .ok_or(UseCaseError::NotFound {
                            entity: "attachment",
                            id: Some(cmd.id),
                        })?;
                attachment.metadata.mark_as_deleted(Utc::now());
                repository.save(&attachment).await?;
                Ok(())
            })
        })
        .await
}

pub(crate) async fn get_attachment(
    executor: &impl RepositoryTransactionExecutor,
    query: GetAttachmentQuery,
) -> Result<Attachment, UseCaseError> {
    executor
        .execute(|factory, _| {
            Box::pin(async move {
                match query {
                    GetAttachmentQuery::ById(id) => factory
                        .attachment_repository()
                        .find_by_id(&id, false)
                        .await?
                        .ok_or(UseCaseError::NotFound {
                            entity: "attachment",
                            id: Some(id),
                        }),
                }
            })
        })
        .await
}

pub(crate) async fn list_attachments(
    executor: &impl RepositoryTransactionExecutor,
    query: ListAttachmentsQuery,
) -> Result<Vec<Attachment>, UseCaseError> {
    executor
        .execute(|factory, _| {
            Box::pin(async move {
                match query {
                    ListAttachmentsQuery::ByQuestionId(question_id) => Ok(factory
                        .attachment_repository()
                        .find_by_question_id(&question_id, false)
                        .await?),
                }
            })
        })
        .await
}
