use crate::domain::model::SrsData;
use crate::domain::repository::{
    RepositoryFactory, RepositoryTransactionExecutor, SrsDataRepository,
};

use super::{command::UpdateSrsDataCommand, query::GetSrsDataQuery, UseCaseError};

pub(crate) async fn update_srs_data(
    executor: &impl RepositoryTransactionExecutor,
    cmd: UpdateSrsDataCommand,
) -> Result<(), UseCaseError> {
    executor
        .execute(|factory, _| {
            Box::pin(async move {
                let repository = factory.srs_data_repository();
                let mut data = repository
                    .find_by_question_id(&cmd.question_id)
                    .await?
                    .ok_or(UseCaseError::NotFound("srs data"))?;
                data.review(cmd.reviewed_at, cmd.review_feedback)?;
                repository.save(&data).await?;
                Ok(())
            })
        })
        .await
}

pub(crate) async fn get_srs_data(
    executor: &impl RepositoryTransactionExecutor,
    query: GetSrsDataQuery,
) -> Result<SrsData, UseCaseError> {
    executor
        .execute(|factory, _| {
            Box::pin(async move {
                factory
                    .srs_data_repository()
                    .find_by_question_id(&query.question_id)
                    .await?
                    .filter(|value| value.metadata.deleted_at.is_none())
                    .ok_or(UseCaseError::NotFound("srs data"))
            })
        })
        .await
}

pub(crate) async fn list_srs_data(
    executor: &impl RepositoryTransactionExecutor,
) -> Result<Vec<SrsData>, UseCaseError> {
    executor
        .execute(|factory, _| {
            Box::pin(async move { Ok(factory.srs_data_repository().find_all(false).await?) })
        })
        .await
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, Utc};
    use uuid::Uuid;

    use crate::domain::model::error::{DomainError, InvalidFeedback, InvalidReviewTime};
    use crate::domain::model::{Metadata, Question, SrsData, SyncStatus};
    use crate::domain::repository::error::RepositoryError;
    use crate::domain::repository::legacy::repository_model::srs_data::SyncedSrsData;
    use crate::domain::repository::legacy::SrsDataRepository as LegacySrsDataRepository;
    use crate::domain::repository::{
        QuestionRepository, RepositoryFactory, RepositoryTransactionExecutor, SrsDataRepository,
    };

    use super::super::command::UpdateSrsDataCommand;
    use super::*;

    #[tokio::test]
    async fn reports_missing_srs_data() {
        let executor = super::super::test_executor().await;
        let question_id = Uuid::new_v4();
        assert_eq!(
            update_srs_data(
                &executor,
                UpdateSrsDataCommand {
                    question_id,
                    review_feedback: 0.5,
                    reviewed_at: Utc::now(),
                },
            )
            .await,
            Err(UseCaseError::NotFound("srs data"))
        );
    }

    #[tokio::test]
    async fn reports_invalid_feedback_without_changing_data() {
        let executor = super::super::test_executor().await;
        let initial_time = Utc::now() - Duration::days(1);
        let question = Question::new(
            Uuid::new_v4(),
            Metadata::new(initial_time),
            None,
            None,
            "stem".into(),
            "answer".into(),
            None,
            None,
            Vec::new(),
            Vec::new(),
        );
        let question_id = question.id;
        let reviewed_at = initial_time + Duration::days(1);
        let mut metadata = Metadata::new(initial_time);
        metadata.sync_status = SyncStatus::Synced;
        let srs_data = SrsData::new(question_id, metadata);
        let original = srs_data.clone();
        executor
            .execute(|factory, _| {
                Box::pin(async move {
                    factory.question_repository().save(&question).await?;
                    factory.srs_data_repository().save(&srs_data).await?;
                    Ok::<_, RepositoryError>(())
                })
            })
            .await
            .unwrap();

        assert_eq!(
            update_srs_data(
                &executor,
                UpdateSrsDataCommand {
                    question_id,
                    review_feedback: -0.1,
                    reviewed_at,
                },
            )
            .await,
            Err(UseCaseError::Domain(DomainError::InvalidFeedback(
                InvalidFeedback { feedback: -0.1 }
            )))
        );
        assert_eq!(
            get_srs_data(&executor, GetSrsDataQuery { question_id })
                .await
                .unwrap(),
            original
        );

        let out_of_order_time = initial_time - Duration::seconds(1);
        assert_eq!(
            update_srs_data(
                &executor,
                UpdateSrsDataCommand {
                    question_id,
                    review_feedback: 0.5,
                    reviewed_at: out_of_order_time,
                },
            )
            .await,
            Err(UseCaseError::Domain(DomainError::InvalidReviewTime(
                InvalidReviewTime {
                    reviewed_at: out_of_order_time,
                    last_review_at: initial_time,
                }
            )))
        );
        assert_eq!(
            get_srs_data(&executor, GetSrsDataQuery { question_id })
                .await
                .unwrap(),
            original
        );
    }

    #[tokio::test]
    async fn applies_review_feedback_and_persists_the_updated_schedule() {
        let executor = super::super::test_executor().await;
        let initial_time = Utc::now() - Duration::days(1);
        let question = Question::new(
            Uuid::new_v4(),
            Metadata::new(initial_time),
            None,
            None,
            "stem".into(),
            "answer".into(),
            None,
            None,
            Vec::new(),
            Vec::new(),
        );
        let question_id = question.id;
        let reviewed_at = initial_time + Duration::days(1);
        let mut metadata = Metadata::new(initial_time);
        metadata.sync_status = SyncStatus::Synced;
        let srs_data = SrsData::new(question_id, metadata);
        let original_stability = srs_data.stability();
        let original_difficulty = srs_data.difficulty();
        executor
            .execute(|factory, _| {
                Box::pin(async move {
                    factory.question_repository().save(&question).await?;
                    factory.srs_data_repository().save(&srs_data).await?;
                    Ok::<_, RepositoryError>(())
                })
            })
            .await
            .unwrap();

        update_srs_data(
            &executor,
            UpdateSrsDataCommand {
                question_id,
                review_feedback: 0.7,
                reviewed_at,
            },
        )
        .await
        .unwrap();
        let saved = get_srs_data(&executor, GetSrsDataQuery { question_id })
            .await
            .unwrap();
        assert_eq!(saved.review_count(), 2);
        assert_eq!(saved.feedback_history(), &[0.7]);
        assert!(saved.stability() > original_stability);
        assert_ne!(saved.difficulty(), original_difficulty);
        assert_eq!(saved.last_review_at(), Some(reviewed_at));
        assert!(saved.next_review_at() > saved.last_review_at());
        assert_eq!(saved.metadata.updated_at, reviewed_at);
        assert_eq!(saved.metadata.sync_status, SyncStatus::Pending);
        assert_eq!(saved.metadata.deleted_at, None);
    }

    #[tokio::test]
    async fn reads_srs_state_normalized_at_the_legacy_write_boundary() {
        let executor = super::super::test_executor().await;
        let now = Utc::now();
        let question = Question::new(
            Uuid::new_v4(),
            Metadata::new(now),
            None,
            None,
            "stem".into(),
            "answer".into(),
            None,
            None,
            Vec::new(),
            Vec::new(),
        );
        let question_id = question.id;
        executor
            .execute(|factory, _| {
                Box::pin(async move {
                    factory.question_repository().save(&question).await?;
                    factory
                        .legacy_srs_data_repository()
                        .upsert_synced(SyncedSrsData {
                            id: question_id.to_string(),
                            version: 4,
                            deleted_at: None,
                            question_id: question_id.to_string(),
                            stability: -1.0,
                            difficulty: 0.3,
                            next_review_at: None,
                            last_reviewed_at: None,
                            review_count: -1,
                            feedback_history: "[-0.1,0.1,0.2,0.3,0.4,0.5,1.1]".into(),
                            now: now.timestamp(),
                        })
                        .await;
                    Ok::<_, RepositoryError>(())
                })
            })
            .await
            .unwrap();

        let saved = get_srs_data(&executor, GetSrsDataQuery { question_id })
            .await
            .unwrap();
        assert_eq!(saved.stability(), SrsData::INITIAL_STABILITY);
        assert_eq!(saved.difficulty(), SrsData::INITIAL_DIFFICULTY);
        assert_eq!(saved.review_count(), 0);
        assert_eq!(saved.feedback_history(), &[0.1, 0.2, 0.3, 0.4, 0.5]);
    }
}
