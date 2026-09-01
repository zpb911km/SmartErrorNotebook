use chrono::Utc;
use uuid::Uuid;

use crate::model::{Attachment, Metadata};
use crate::repository::{AttachmentRepository, RepositoryFactory, RepositoryTransactionExecutor};
use crate::util::codec::sha256;

use super::{
    command::{CreateAttachmentCommand, DeleteAttachmentCommand},
    query::GetAttachmentQuery,
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
                    if repository.find_by_id(&candidate_id).await?.is_none() {
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
                if repository.find_by_id(&cmd.id).await?.is_none() {
                    return Err(UseCaseError::NotFound("attachment"));
                }
                repository.delete_by_id(&cmd.id).await?;
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
                factory
                    .attachment_repository()
                    .find_by_id(&query.id)
                    .await?
                    .filter(|value| value.metadata.deleted_at.is_none())
                    .ok_or(UseCaseError::NotFound("attachment"))
            })
        })
        .await
}
