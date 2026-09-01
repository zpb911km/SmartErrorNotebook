use crate::model::Subject;
use crate::repository::{RepositoryFactory, RepositoryTransactionExecutor, SubjectRepository};

use super::{
    command::{DeleteSubjectCommand, SaveSubjectCommand},
    query::GetSubjectQuery,
    UseCaseError,
};

pub(crate) async fn save_subject(
    executor: &impl RepositoryTransactionExecutor,
    cmd: SaveSubjectCommand,
) -> Result<(), UseCaseError> {
    executor
        .execute(|factory, _| {
            Box::pin(async move {
                factory
                    .subject_repository()
                    .save(&Subject {
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

pub(crate) async fn delete_subject(
    executor: &impl RepositoryTransactionExecutor,
    cmd: DeleteSubjectCommand,
) -> Result<(), UseCaseError> {
    executor
        .execute(|factory, _| {
            Box::pin(async move {
                let repository = factory.subject_repository();
                if repository.find_by_id(&cmd.id).await?.is_none() {
                    return Err(UseCaseError::NotFound("subject"));
                }
                repository.delete_by_id(&cmd.id).await?;
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
                factory
                    .subject_repository()
                    .find_by_id(&query.id)
                    .await?
                    .filter(|value| value.metadata.deleted_at.is_none())
                    .ok_or(UseCaseError::NotFound("subject"))
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
