mod sea_orm_attachment_repository;
mod sea_orm_question_repository;
mod sea_orm_source_repository;
mod sea_orm_srs_data_repository;
mod sea_orm_subject_repository;
mod sea_orm_sync_repository;
mod sea_orm_tag_repository;

use std::future::Future;
use std::pin::Pin;

use chrono::{DateTime, Utc};
use sea_orm::{ConnectionTrait, DatabaseConnection, DatabaseTransaction, TransactionTrait};

use crate::domain::repository::{
    legacy, AttachmentRepository, QuestionRepository, RepositoryFactory,
    RepositoryTransactionExecutor, SourceRepository, SrsDataRepository, SubjectRepository,
    TagRepository,
};
use crate::util::parsing::parse_uuid;

use sea_orm_attachment_repository::SeaOrmAttachmentRepository;
use sea_orm_question_repository::SeaOrmQuestionRepository;
use sea_orm_source_repository::SeaOrmSourceRepository;
use sea_orm_srs_data_repository::SeaOrmSrsDataRepository;
use sea_orm_subject_repository::SeaOrmSubjectRepository;
use sea_orm_sync_repository::SeaOrmSyncRepository;
use sea_orm_tag_repository::SeaOrmTagRepository;

fn uuid(value: &str, field: &str) -> uuid::Uuid {
    parse_uuid(value, field).unwrap_or_else(|error| panic!("{error}: {value}"))
}

fn timestamp(value: i64, field: &str) -> DateTime<Utc> {
    DateTime::from_timestamp(value, 0).unwrap_or_else(|| panic!("invalid {field}: {value}"))
}

pub struct SeaOrmRepositoryFactory<'c, C: ConnectionTrait> {
    connection: &'c C,
}
impl<'c, C: ConnectionTrait> SeaOrmRepositoryFactory<'c, C> {
    pub fn new(connection: &'c C) -> Self {
        Self { connection }
    }
}

impl<'c, C: ConnectionTrait> RepositoryFactory for SeaOrmRepositoryFactory<'c, C> {
    fn attachment_repository(&self) -> impl AttachmentRepository {
        SeaOrmAttachmentRepository::new(self.connection)
    }

    fn question_repository(&self) -> impl QuestionRepository {
        SeaOrmQuestionRepository::new(self.connection)
    }

    fn tag_repository(&self) -> impl TagRepository {
        SeaOrmTagRepository::new(self.connection)
    }

    fn source_repository(&self) -> impl SourceRepository {
        SeaOrmSourceRepository::new(self.connection)
    }

    fn srs_data_repository(&self) -> impl SrsDataRepository {
        SeaOrmSrsDataRepository::new(self.connection)
    }

    fn subject_repository(&self) -> impl SubjectRepository {
        SeaOrmSubjectRepository::new(self.connection)
    }

    fn legacy_attachment_repository(&self) -> impl legacy::AttachmentRepository {
        SeaOrmAttachmentRepository::new(self.connection)
    }

    fn legacy_error_question_repository(&self) -> impl legacy::ErrorQuestionRepository {
        SeaOrmQuestionRepository::new(self.connection)
    }

    fn legacy_error_tag_repository(&self) -> impl legacy::ErrorTagRepository {
        SeaOrmTagRepository::new(self.connection)
    }

    fn legacy_source_repository(&self) -> impl legacy::SourceRepository {
        SeaOrmSourceRepository::new(self.connection)
    }

    fn legacy_srs_data_repository(&self) -> impl legacy::SrsDataRepository {
        SeaOrmSrsDataRepository::new(self.connection)
    }

    fn legacy_subject_repository(&self) -> impl legacy::SubjectRepository {
        SeaOrmSubjectRepository::new(self.connection)
    }

    fn legacy_sync_repository(&self) -> impl legacy::SyncRepository {
        SeaOrmSyncRepository::new(self.connection)
    }
}

pub struct SeaOrmRepositoryTransactionExecutor {
    connection: DatabaseConnection,
}
impl SeaOrmRepositoryTransactionExecutor {
    pub fn new(connection: DatabaseConnection) -> Self {
        Self { connection }
    }
}

#[async_trait::async_trait]
impl RepositoryTransactionExecutor for SeaOrmRepositoryTransactionExecutor {
    type Factory<'tx> = SeaOrmRepositoryFactory<'tx, DatabaseTransaction>;

    async fn execute<T, E, F>(&self, f: F) -> Result<T, E>
    where
        T: Send,
        E: Send,
        F: Send,
        for<'tx> F: FnOnce(
            Self::Factory<'tx>,
            &'tx (),
        ) -> Pin<Box<dyn Future<Output = Result<T, E>> + Send + 'tx>>,
    {
        let transaction = self
            .connection
            .begin()
            .await
            .expect("failed to begin repository transaction");
        let factory = Self::Factory::new(&transaction);
        let result = f(factory, &()).await;

        match result {
            Ok(value) => {
                transaction
                    .commit()
                    .await
                    .expect("failed to commit repository transaction");
                Ok(value)
            }
            Err(error) => {
                transaction
                    .rollback()
                    .await
                    .expect("failed to roll back repository transaction");
                Err(error)
            }
        }
    }
}
