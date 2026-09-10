#![allow(dead_code)] // Internal use cases include primitives retained for compatibility work.

pub(crate) mod attachment;
pub(crate) mod question;
pub(crate) mod source;
pub(crate) mod srs_data;
pub(crate) mod subject;
pub(crate) mod tag;

pub(crate) mod command;
pub(crate) mod query;
pub(crate) mod result;

pub(crate) mod error;

#[cfg(test)]
use crate::data::SeaOrmRepositoryTransactionExecutor;

pub(crate) use error::UseCaseError;

#[cfg(test)]
async fn test_executor() -> SeaOrmRepositoryTransactionExecutor {
    use sea_orm::{ConnectOptions, Database};

    use crate::data::database::connection::init_database;

    let mut options = ConnectOptions::new("sqlite::memory:");
    options.max_connections(1).min_connections(1);
    let database = Database::connect(options)
        .await
        .expect("connect application test database");
    init_database(&database)
        .await
        .expect("migrate application test database");
    SeaOrmRepositoryTransactionExecutor::new(database)
}

#[cfg(test)]
mod tests {
    use sea_orm::{ConnectionTrait, Database};

    use crate::domain::repository::error::{
        RepositoryError, RepositoryFindError, RepositoryInfrastructureError,
    };

    use super::*;

    #[tokio::test]
    async fn database_failures_are_returned_instead_of_panicking() {
        let database = Database::connect("sqlite::memory:").await.unwrap();
        crate::data::database::connection::init_database(&database)
            .await
            .unwrap();
        database
            .execute_unprepared("DROP TABLE subject")
            .await
            .unwrap();
        let executor = SeaOrmRepositoryTransactionExecutor::new(database);

        assert!(matches!(
            subject::list_subjects(&executor).await,
            Err(UseCaseError::Repository(RepositoryError::Find(
                RepositoryFindError::Infrastructure(RepositoryInfrastructureError {
                    operation: "list subjects",
                    ..
                })
            )))
        ));
    }
}
