use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::{ConnectionTrait, DbBackend, Statement, prelude::Uuid};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260828_105409_normalize_srs_state"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let backend = manager.get_database_backend();
        if backend != DbBackend::Sqlite {
            return Err(DbErr::BackendNotSupported {
                db: backend.as_str(),
                ctx: "m20260828_105409_normalize_srs_state only supports SQLite.",
            });
        }

        let connection = manager.get_connection();
        let rows = connection
            .query_all_raw(Statement::from_string(
                backend,
                "SELECT question_id, stability, difficulty, review_count, \
                 CAST(feedback_history AS TEXT) AS feedback_history FROM srs_data",
            ))
            .await?;
        for row in rows {
            let question_id: Uuid = row.try_get("", "question_id")?;
            let state = normalize_srs_state(
                row.try_get("", "stability")?,
                row.try_get("", "difficulty")?,
                row.try_get("", "review_count")?,
                &row.try_get::<String>("", "feedback_history")?,
            );
            let feedback_history = serde_json::to_string(&state.feedback_history)
                .map_err(|error| DbErr::Migration(error.to_string()))?;
            connection
                .execute_raw(Statement::from_sql_and_values(
                    backend,
                    "UPDATE srs_data SET stability = ?, difficulty = ?, review_count = ?, \
                     feedback_history = ? WHERE question_id = ?",
                    [
                        state.stability.into(),
                        state.difficulty.into(),
                        state.review_count.into(),
                        feedback_history.into(),
                        question_id.into(),
                    ],
                ))
                .await?;
        }
        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Err(DbErr::Migration(
            "m20260828_105409_normalize_srs_state cannot restore discarded invalid SRS values"
                .to_owned(),
        ))
    }

    fn use_transaction(&self) -> Option<bool> {
        Some(true)
    }
}

const INITIAL_STABILITY: f32 = 3.173;
const INITIAL_DIFFICULTY: f32 = 7.1949;
const MAX_FEEDBACK_HISTORY_LEN: usize = 5;

#[derive(Debug, PartialEq)]
struct NormalizedSrsState {
    stability: f32,
    difficulty: f32,
    review_count: i64,
    feedback_history: Vec<f32>,
}

fn normalize_srs_state(
    stability: f32,
    difficulty: f32,
    review_count: i64,
    feedback_history: &str,
) -> NormalizedSrsState {
    let mut feedback_history = serde_json::from_str::<Vec<f32>>(feedback_history)
        .unwrap_or_default()
        .into_iter()
        .filter(|feedback| feedback.is_finite() && (0.0..=1.0).contains(feedback))
        .collect::<Vec<_>>();
    if feedback_history.len() > MAX_FEEDBACK_HISTORY_LEN {
        feedback_history.drain(0..feedback_history.len() - MAX_FEEDBACK_HISTORY_LEN);
    }
    NormalizedSrsState {
        stability: if stability.is_finite() && stability > 0.0 {
            stability
        } else {
            INITIAL_STABILITY
        },
        difficulty: if difficulty.is_finite() && (1.0..=10.0).contains(&difficulty) {
            difficulty
        } else {
            INITIAL_DIFFICULTY
        },
        review_count: std::cmp::Ord::max(review_count, 0),
        feedback_history,
    }
}

#[cfg(test)]
mod tests {
    use sea_orm_migration::sea_orm::{ConnectionTrait, Database};

    use super::*;

    struct TestMigrator;

    #[async_trait::async_trait]
    impl MigratorTrait for TestMigrator {
        fn migrations() -> Vec<Box<dyn MigrationTrait>> {
            vec![Box::new(Migration)]
        }
    }

    #[tokio::test]
    async fn normalizes_existing_srs_state_without_changing_sync_metadata() {
        let database = Database::connect("sqlite::memory:").await.unwrap();
        database
            .execute_unprepared(
                "CREATE TABLE srs_data (\
                    question_id BLOB PRIMARY KEY NOT NULL, \
                    stability REAL NOT NULL, difficulty REAL NOT NULL, \
                    review_count INTEGER NOT NULL, feedback_history JSON NOT NULL, \
                    updated_at TEXT NOT NULL, sync_status TEXT NOT NULL, sync_version INTEGER NOT NULL\
                )",
            )
            .await
            .unwrap();
        let question_id = Uuid::new_v4();
        database
            .execute_raw(Statement::from_sql_and_values(
                DbBackend::Sqlite,
                "INSERT INTO srs_data VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                [
                    question_id.into(),
                    (-1.0_f32).into(),
                    0.3_f32.into(),
                    (-2_i64).into(),
                    "[-0.1,0.0,0.1,0.2,0.3,0.4,0.5,1.1]".into(),
                    "2026-08-28 00:00:00".into(),
                    "SYNCED".into(),
                    7_i64.into(),
                ],
            ))
            .await
            .unwrap();

        TestMigrator::up(&database, None).await.unwrap();

        let row = database
            .query_one_raw(Statement::from_sql_and_values(
                DbBackend::Sqlite,
                "SELECT stability, difficulty, review_count, \
                 CAST(feedback_history AS TEXT) AS feedback_history, \
                 updated_at, sync_status, sync_version FROM srs_data WHERE question_id = ?",
                [question_id.into()],
            ))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            row.try_get::<f32>("", "stability").unwrap(),
            INITIAL_STABILITY
        );
        assert_eq!(
            row.try_get::<f32>("", "difficulty").unwrap(),
            INITIAL_DIFFICULTY
        );
        assert_eq!(row.try_get::<i64>("", "review_count").unwrap(), 0);
        assert_eq!(
            row.try_get::<String>("", "feedback_history").unwrap(),
            "[0.1,0.2,0.3,0.4,0.5]"
        );
        assert_eq!(
            row.try_get::<String>("", "updated_at").unwrap(),
            "2026-08-28 00:00:00"
        );
        assert_eq!(row.try_get::<String>("", "sync_status").unwrap(), "SYNCED");
        assert_eq!(row.try_get::<i64>("", "sync_version").unwrap(), 7);
    }
}
