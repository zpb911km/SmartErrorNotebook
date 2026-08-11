use std::sync::Arc;

use sea_orm::DbConn;
use thiserror::Error;

pub mod attachment;
pub mod error_question;
pub mod error_tag;
pub mod source;
pub mod srs_data;
pub mod subject;
pub mod sync;

pub type RepositoryResult<T> = Result<T, RepositoryError>;

#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("{0}")]
    Database(#[from] sea_orm::DbErr),
    #[error("{0}")]
    NotFound(String),
    #[error("{context}: {source}")]
    Context {
        context: String,
        #[source]
        source: sea_orm::DbErr,
    },
    #[error("{0}")]
    Domain(#[from] crate::domain::DomainError),
}

pub(crate) fn to_domain<T, D>(model: T) -> RepositoryResult<D>
where
    D: TryFrom<T, Error = crate::domain::DomainError>,
{
    model.try_into().map_err(RepositoryError::from)
}

pub(crate) fn to_domains<T, D>(models: Vec<T>) -> RepositoryResult<Vec<D>>
where
    D: TryFrom<T, Error = crate::domain::DomainError>,
{
    models.into_iter().map(to_domain).collect()
}

impl RepositoryError {
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::NotFound(message.into())
    }

    pub fn context(context: impl Into<String>, source: sea_orm::DbErr) -> Self {
        Self::Context {
            context: context.into(),
            source,
        }
    }
}

pub(crate) fn accept_uuid_insert_result<T>(
    result: Result<T, sea_orm::DbErr>,
) -> RepositoryResult<()> {
    match result {
        Ok(_) | Err(sea_orm::DbErr::RecordNotFound(_)) => Ok(()),
        Err(error) => Err(RepositoryError::Database(error)),
    }
}

impl From<RepositoryError> for String {
    fn from(error: RepositoryError) -> Self {
        error.to_string()
    }
}

#[derive(Clone)]
pub struct Repositories {
    pub attachments: Arc<dyn attachment::AttachmentRepository>,
    pub error_questions: Arc<dyn error_question::ErrorQuestionRepository>,
    pub error_tags: Arc<dyn error_tag::ErrorTagRepository>,
    pub sources: Arc<dyn source::SourceRepository>,
    pub srs_data: Arc<dyn srs_data::SrsDataRepository>,
    pub subjects: Arc<dyn subject::SubjectRepository>,
    pub sync: Arc<dyn sync::SyncRepository>,
}

impl Repositories {
    pub fn sea_orm(db: Arc<DbConn>) -> Self {
        Self {
            attachments: Arc::new(attachment::SeaOrmAttachmentRepository::new(db.clone())),
            error_questions: Arc::new(error_question::SeaOrmErrorQuestionRepository::new(
                db.clone(),
            )),
            error_tags: Arc::new(error_tag::SeaOrmErrorTagRepository::new(db.clone())),
            sources: Arc::new(source::SeaOrmSourceRepository::new(db.clone())),
            srs_data: Arc::new(srs_data::SeaOrmSrsDataRepository::new(db.clone())),
            subjects: Arc::new(subject::SeaOrmSubjectRepository::new(db.clone())),
            sync: Arc::new(sync::SeaOrmSyncRepository::new(db)),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::sync::Arc;

    use sea_orm::{ConnectOptions, Database, DbErr};

    use super::{accept_uuid_insert_result, Repositories, RepositoryError};
    use crate::repository::error_question::QuestionQuery;

    #[test]
    fn uuid_insert_result_only_accepts_success_and_record_not_found() {
        assert!(accept_uuid_insert_result::<()>(Ok(())).is_ok());
        assert!(accept_uuid_insert_result::<()>(Err(DbErr::RecordNotFound(
            "Failed to find inserted item".to_string()
        )))
        .is_ok());

        let error =
            accept_uuid_insert_result::<()>(Err(DbErr::Custom("boom".to_string()))).unwrap_err();
        assert!(matches!(error, RepositoryError::Database(_)));
    }

    #[test]
    fn repository_error_preserves_category_and_source() {
        let not_found = RepositoryError::not_found("Subject not found");
        assert!(matches!(not_found, RepositoryError::NotFound(_)));
        assert_eq!(not_found.to_string(), "Subject not found");

        let database = RepositoryError::Database(DbErr::Custom("boom".to_string()));
        assert!(database.source().is_some());
        assert!(database.to_string().contains("boom"));

        let context = RepositoryError::context(
            "Failed to query subjects",
            DbErr::Custom("boom".to_string()),
        );
        assert!(matches!(context, RepositoryError::Context { .. }));
        assert!(context.source().is_some());
        let message = context.to_string();
        assert!(message.starts_with("Failed to query subjects:"));
        assert!(message.contains("boom"));
    }

    #[test]
    fn repository_error_maps_to_command_string() {
        let command_error = String::from(RepositoryError::not_found("标签不存在"));
        assert_eq!(command_error, "标签不存在");
    }

    #[tokio::test]
    async fn repositories_preserve_database_error_categories() {
        let mut options = ConnectOptions::new("sqlite::memory:");
        options.max_connections(1).min_connections(1);
        let db = Database::connect(options).await.unwrap();
        crate::database::init_database(&db).await.unwrap();
        let repositories = Repositories::sea_orm(Arc::new(db.clone()));
        db.close().await.unwrap();

        let errors = [
            repositories.subjects.list_active().await.unwrap_err(),
            repositories.error_tags.list_active().await.unwrap_err(),
            repositories
                .attachments
                .list_active_by_question("question".to_string())
                .await
                .unwrap_err(),
            repositories
                .error_questions
                .list_active(QuestionQuery {
                    subject_id: None,
                    search: None,
                    limit: None,
                    offset: None,
                })
                .await
                .unwrap_err(),
            repositories.sources.list_active(None).await.unwrap_err(),
            repositories.srs_data.list_active().await.unwrap_err(),
        ];
        assert!(errors
            .into_iter()
            .all(|error| matches!(error, RepositoryError::Database(_))));

        let sync_error = repositories.sync.all_headers().await.unwrap_err();
        assert!(matches!(sync_error, RepositoryError::Context { .. }));
        assert!(sync_error.source().is_some());
        assert!(sync_error
            .to_string()
            .starts_with("Failed to query error_questions:"));
    }
}
