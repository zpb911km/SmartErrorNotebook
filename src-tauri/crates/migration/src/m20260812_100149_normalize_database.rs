use base64::{Engine as _, engine::general_purpose};
use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::{ConnectionTrait, DbBackend, Statement, prelude::Uuid};
use sha2::{Digest, Sha256};
use std::fmt::Write;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260812_100149_normalize_database"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let backend = manager.get_database_backend();
        if backend != DbBackend::Sqlite {
            return Err(DbErr::BackendNotSupported {
                db: backend.as_str(),
                ctx: "m20260812_100149_normalize_database only supports SQLite.",
            });
        }

        validate_legacy_uuid_values(manager.get_connection()).await?;
        validate_legacy_srs_references(manager.get_connection()).await?;
        create_subject_table(manager).await?;
        create_source_table(manager).await?;
        create_question_table(manager).await?;
        rename_and_create_srs_data_table(manager).await?;
        create_attachment_table(manager).await?;
        create_question_attachment_cross_ref_table(manager).await?;
        create_tag_table(manager).await?;
        create_question_tag_cross_ref_table(manager).await?;

        migrate_legacy_data(&manager).await?;
        drop_legacy_tables(&manager).await?;

        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Err(DbErr::Migration(
            "m20260812_100149_normalize_database cannot be rolled back because normalization discards legacy-only fields."
                .to_owned(),
        ))
    }

    fn use_transaction(&self) -> Option<bool> {
        Some(true)
    }
}

async fn validate_legacy_uuid_values<C>(connection: &C) -> Result<(), DbErr>
where
    C: ConnectionTrait,
{
    let rows = connection
        .query_all_raw(Statement::from_string(
            DbBackend::Sqlite,
            r#"
            SELECT 'subjects.id' AS field, id AS value FROM subjects
            UNION ALL SELECT 'sources.id', id FROM sources
            UNION ALL SELECT 'sources.question_id', question_id FROM sources WHERE question_id IS NOT NULL
            UNION ALL SELECT 'sources.subject_id', subject_id FROM sources WHERE subject_id IS NOT NULL
            UNION ALL SELECT 'error_questions.id', id FROM error_questions
            UNION ALL SELECT 'error_questions.subjectid', subjectid FROM error_questions
            UNION ALL SELECT 'error_questions.sourceid', sourceid FROM error_questions WHERE sourceid IS NOT NULL
            UNION ALL SELECT 'srs_data.question_id', question_id FROM srs_data
            UNION ALL SELECT 'attachments.id', id FROM attachments
            UNION ALL SELECT 'attachments.question_id', question_id FROM attachments
            UNION ALL SELECT 'error_tags.id', id FROM error_tags
            UNION ALL SELECT 'error_tags.question_id', question_id FROM error_tags
            "#,
        ))
        .await?;

    let mut invalid = Vec::new();
    let mut total = 0usize;
    for row in rows {
        let field: String = row.try_get("", "field")?;
        let value: String = row.try_get("", "value")?;
        let canonical = Uuid::parse_str(&value)
            .map(|uuid| uuid.hyphenated().to_string())
            .is_ok_and(|uuid| uuid.eq_ignore_ascii_case(&value));
        if !canonical {
            total += 1;
            if invalid.len() < 20 {
                invalid.push(format!("{field}={value}"));
            }
        }
    }
    if total == 0 {
        return Ok(());
    }

    Err(DbErr::Migration(format!(
        "found {total} invalid UUID value(s); correct them before migration ({})",
        invalid.join(", ")
    )))
}

fn uuid_blob(value: &str, field: &str) -> Result<Vec<u8>, DbErr> {
    Uuid::parse_str(value)
        .map(|uuid| uuid.as_bytes().to_vec())
        .map_err(|error| DbErr::Migration(format!("invalid {field} UUID {value}: {error}")))
}

async fn validate_legacy_srs_references<C>(connection: &C) -> Result<(), DbErr>
where
    C: ConnectionTrait,
{
    let rows = connection
        .query_all_raw(Statement::from_string(
            DbBackend::Sqlite,
            r#"
            SELECT legacy.id, legacy.question_id
            FROM srs_data legacy
            LEFT JOIN error_questions question ON question.id = legacy.question_id
            WHERE question.id IS NULL
            ORDER BY legacy.id
            "#,
        ))
        .await?;
    if rows.is_empty() {
        return Ok(());
    }

    let total = rows.len();
    let examples = rows
        .iter()
        .take(20)
        .map(|row| {
            Ok(format!(
                "{}/{}",
                row.try_get::<String>("", "id")?,
                row.try_get::<String>("", "question_id")?
            ))
        })
        .collect::<Result<Vec<_>, DbErr>>()?
        .join(", ");
    Err(DbErr::Migration(format!(
        "found {total} orphan SRS record(s); restore or remove their questions before migration (srs_id/question_id: {examples})"
    )))
}

#[derive(DeriveIden)]
enum Metadata {
    CreatedAt,
    UpdatedAt,
    DeletedAt,
    SyncStatus,
    SyncVersion,
}

#[derive(DeriveIden)]
enum Subject {
    Table,
    Id,
    Name,
    Color,
}

#[derive(DeriveIden)]
enum Source {
    Table,
    Id,
    SubjectId,
    Book,
    Chapter,
    Knowledge,
}

#[derive(DeriveIden)]
enum Question {
    Table,
    Id,
    QuestionType,
    SourceId,
    Stem,
    CorrectAnswer,
    Explanation,
    Note,
}

#[derive(DeriveIden)]
enum SrsData {
    Table,
    QuestionId,
    Stability,
    Difficulty,
    NextReviewAt,
    LastReviewedAt,
    ReviewCount,
    FeedbackHistory,
}

#[derive(DeriveIden)]
enum Attachment {
    Table,
    Id,
    MimeType,
    Data,
    Sha256,
}

#[derive(DeriveIden)]
enum Tag {
    Table,
    Id,
    Name,
    Color,
}

#[derive(DeriveIden)]
enum QuestionAttachmentCrossRef {
    Table,
    QuestionId,
    AttachmentId,
}

#[derive(DeriveIden)]
enum QuestionTagCrossRef {
    Table,
    QuestionId,
    TagId,
}

fn column(name: impl IntoIden) -> ColumnDef {
    ColumnDef::new(name)
}

trait AddMetadataColumns {
    fn add_metadata_col(&mut self) -> &mut Self;
}

impl AddMetadataColumns for TableCreateStatement {
    fn add_metadata_col(&mut self) -> &mut Self {
        self.col(column(Metadata::CreatedAt).timestamp().not_null())
            .col(column(Metadata::UpdatedAt).timestamp().not_null())
            .col(column(Metadata::DeletedAt).timestamp())
            .col(column(Metadata::SyncStatus).string().not_null())
            .col(column(Metadata::SyncVersion).big_integer().not_null())
    }
}

async fn create_attachment_table(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(Attachment::Table)
                .if_not_exists()
                .col(column(Attachment::Id).uuid().not_null().primary_key())
                .add_metadata_col()
                .col(column(Attachment::MimeType).string().not_null())
                .col(column(Attachment::Data).blob().not_null())
                .col(column(Attachment::Sha256).string_len(64).not_null())
                .check(Expr::cust(
                    "length(sha256) = 64 AND sha256 NOT GLOB '*[^0-9a-f]*'",
                ))
                .to_owned(),
        )
        .await
}

async fn create_question_table(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(Question::Table)
                .if_not_exists()
                .col(column(Question::Id).uuid().not_null().primary_key())
                .add_metadata_col()
                .col(column(Question::QuestionType).string())
                .col(column(Question::SourceId).uuid())
                .col(column(Question::Stem).text().not_null())
                .col(column(Question::CorrectAnswer).text().not_null())
                .col(column(Question::Explanation).text())
                .col(column(Question::Note).text())
                .foreign_key(
                    ForeignKey::create()
                        .from(Question::Table, Question::SourceId)
                        .to(Source::Table, Source::Id)
                        .on_delete(ForeignKeyAction::SetNull),
                )
                .to_owned(),
        )
        .await
}

async fn create_source_table(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(Source::Table)
                .if_not_exists()
                .col(column(Source::Id).uuid().not_null().primary_key())
                .add_metadata_col()
                .col(column(Source::SubjectId).uuid())
                .col(column(Source::Book).string())
                .col(column(Source::Chapter).string())
                .col(column(Source::Knowledge).string())
                .foreign_key(
                    ForeignKey::create()
                        .from(Source::Table, Source::SubjectId)
                        .to(Subject::Table, Subject::Id)
                        .on_delete(ForeignKeyAction::SetNull),
                )
                .to_owned(),
        )
        .await
}

async fn rename_and_create_srs_data_table(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .rename_table(
            Table::rename()
                .table(SrsData::Table, "srs_data_legacy_table")
                .to_owned(),
        )
        .await?;
    manager
        .create_table(
            Table::create()
                .table(SrsData::Table)
                .if_not_exists()
                .col(column(SrsData::QuestionId).uuid().not_null().primary_key())
                .add_metadata_col()
                .col(column(SrsData::Stability).float().not_null())
                .col(column(SrsData::Difficulty).float().not_null())
                .col(column(SrsData::NextReviewAt).timestamp())
                .col(column(SrsData::LastReviewedAt).timestamp())
                .col(column(SrsData::ReviewCount).integer().not_null())
                .col(column(SrsData::FeedbackHistory).json().not_null())
                .foreign_key(
                    ForeignKey::create()
                        .from(SrsData::Table, SrsData::QuestionId)
                        .to(Question::Table, Question::Id)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .to_owned(),
        )
        .await
}

async fn create_subject_table(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(Subject::Table)
                .if_not_exists()
                .col(column(Subject::Id).uuid().not_null().primary_key())
                .add_metadata_col()
                .col(column(Subject::Name).string().not_null())
                .col(column(Subject::Color).string().not_null())
                .to_owned(),
        )
        .await
}

async fn create_tag_table(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(Tag::Table)
                .if_not_exists()
                .col(column(Tag::Id).uuid().not_null().primary_key())
                .add_metadata_col()
                .col(column(Tag::Name).string().not_null())
                .col(column(Tag::Color).string().not_null())
                .to_owned(),
        )
        .await
}

async fn create_question_attachment_cross_ref_table(
    manager: &SchemaManager<'_>,
) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(QuestionAttachmentCrossRef::Table)
                .if_not_exists()
                .col(
                    column(QuestionAttachmentCrossRef::QuestionId)
                        .uuid()
                        .not_null(),
                )
                .col(
                    column(QuestionAttachmentCrossRef::AttachmentId)
                        .uuid()
                        .not_null(),
                )
                .primary_key(
                    Index::create()
                        .col(QuestionAttachmentCrossRef::QuestionId)
                        .col(QuestionAttachmentCrossRef::AttachmentId),
                )
                .foreign_key(
                    ForeignKey::create()
                        .from(
                            QuestionAttachmentCrossRef::Table,
                            QuestionAttachmentCrossRef::QuestionId,
                        )
                        .to(Question::Table, Question::Id)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .foreign_key(
                    ForeignKey::create()
                        .from(
                            QuestionAttachmentCrossRef::Table,
                            QuestionAttachmentCrossRef::AttachmentId,
                        )
                        .to(Attachment::Table, Attachment::Id)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .to_owned(),
        )
        .await
}

async fn create_question_tag_cross_ref_table(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(QuestionTagCrossRef::Table)
                .if_not_exists()
                .col(column(QuestionTagCrossRef::QuestionId).uuid().not_null())
                .col(column(QuestionTagCrossRef::TagId).uuid().not_null())
                .primary_key(
                    Index::create()
                        .col(QuestionTagCrossRef::QuestionId)
                        .col(QuestionTagCrossRef::TagId),
                )
                .foreign_key(
                    ForeignKey::create()
                        .from(QuestionTagCrossRef::Table, QuestionTagCrossRef::QuestionId)
                        .to(Question::Table, Question::Id)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .foreign_key(
                    ForeignKey::create()
                        .from(QuestionTagCrossRef::Table, QuestionTagCrossRef::TagId)
                        .to(Tag::Table, Tag::Id)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .to_owned(),
        )
        .await
}

async fn migrate_legacy_data(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    let connection = manager.get_connection();

    connection
        .execute_unprepared(
            r#"
            INSERT INTO subject (
                id, created_at, updated_at, deleted_at, sync_status, sync_version, name, color
            )
            SELECT
                unhex(replace(id, '-', '')),
                datetime(created_at, 'unixepoch'),
                datetime(updated_at, 'unixepoch'),
                CASE WHEN deleted_at IS NULL THEN NULL ELSE datetime(deleted_at, 'unixepoch') END,
                CASE sync_status
                    WHEN 'pending' THEN 'PENDING'
                    WHEN 'synced' THEN 'SYNCED'
                    ELSE 'PENDING'
                END,
                COALESCE(version, 0),
                name,
                COALESCE(color, '')
            FROM subjects
            "#,
        )
        .await?;

    connection
        .execute_unprepared(
            r#"
            INSERT INTO source (
                id, created_at, updated_at, deleted_at, sync_status, sync_version,
                subject_id, book, chapter, knowledge
            )
            SELECT
                unhex(replace(legacy_source.id, '-', '')),
                datetime(legacy_source.created_at, 'unixepoch'),
                datetime(legacy_source.updated_at, 'unixepoch'),
                CASE
                    WHEN legacy_source.deleted_at IS NULL THEN NULL
                    ELSE datetime(legacy_source.deleted_at, 'unixepoch')
                END,
                CASE legacy_source.sync_status
                    WHEN 'pending' THEN 'PENDING'
                    WHEN 'synced' THEN 'SYNCED'
                    ELSE 'PENDING'
                END,
                COALESCE(legacy_source.version, 0),
                COALESCE(
                    (SELECT subject.id
                     FROM subject
                     WHERE subject.id = unhex(replace(legacy_source.subject_id, '-', ''))
                     LIMIT 1),
                    (SELECT subject.id
                     FROM error_questions eq
                     JOIN subject
                       ON subject.id = unhex(replace(eq.subjectid, '-', ''))
                     WHERE eq.id = legacy_source.question_id
                     LIMIT 1),
                    (SELECT subject.id
                     FROM error_questions eq
                     JOIN subject
                       ON subject.id = unhex(replace(eq.subjectid, '-', ''))
                     WHERE eq.sourceid = legacy_source.id
                     ORDER BY eq.id
                     LIMIT 1)
                ),
                legacy_source.book,
                legacy_source.chapter,
                legacy_source.knowledge
            FROM sources legacy_source
            "#,
        )
        .await?;

    connection
        .execute_unprepared(
            r#"
            INSERT INTO question (
                id, created_at, updated_at, deleted_at, sync_status, sync_version,
                question_type, source_id, stem, correct_answer, explanation, note
            )
            SELECT
                unhex(replace(eq.id, '-', '')),
                datetime(eq.created_at, 'unixepoch'),
                datetime(eq.updated_at, 'unixepoch'),
                CASE WHEN eq.deleted_at IS NULL THEN NULL ELSE datetime(eq.deleted_at, 'unixepoch') END,
                CASE eq.sync_status
                    WHEN 'pending' THEN 'PENDING'
                    WHEN 'synced' THEN 'SYNCED'
                    ELSE 'PENDING'
                END,
                COALESCE(eq.version, 0),
                CASE eq."type"
                    WHEN '单选题' THEN 'SINGLE_SELECT'
                    WHEN 'SINGLE_SELECT' THEN 'SINGLE_SELECT'
                    WHEN 'SigleChoice' THEN 'SINGLE_SELECT'
                    WHEN 'SingleChoice' THEN 'SINGLE_SELECT'
                    WHEN '多选题' THEN 'MULTIPLE_SELECT'
                    WHEN 'MULTIPLE_SELECT' THEN 'MULTIPLE_SELECT'
                    WHEN 'MultipleChoice' THEN 'MULTIPLE_SELECT'
                    WHEN '判断题' THEN 'TRUE_FALSE'
                    WHEN 'TRUE_FALSE' THEN 'TRUE_FALSE'
                    WHEN 'TrueFalse' THEN 'TRUE_FALSE'
                    WHEN '填空题' THEN 'FILL_IN_THE_BLANK'
                    WHEN 'FILL_IN_THE_BLANK' THEN 'FILL_IN_THE_BLANK'
                    WHEN 'FillInTheBlank' THEN 'FILL_IN_THE_BLANK'
                    WHEN '简答题' THEN 'SHORT_ANSWER'
                    WHEN 'SHORT_ANSWER' THEN 'SHORT_ANSWER'
                    WHEN 'ShortAnswer' THEN 'SHORT_ANSWER'
                    WHEN '计算题' THEN 'CALCULATION'
                    WHEN 'CALCULATION' THEN 'CALCULATION'
                    WHEN 'Calculation' THEN 'CALCULATION'
                    WHEN '论述题' THEN 'ESSAY'
                    WHEN 'ESSAY' THEN 'ESSAY'
                    WHEN 'Essay' THEN 'ESSAY'
                    ELSE NULL
                END,
                COALESCE(
                    (SELECT s.id
                     FROM source s
                     WHERE s.id = unhex(replace(eq.sourceid, '-', ''))
                     LIMIT 1),
                    (SELECT s.id
                     FROM sources legacy_source
                     JOIN source s
                       ON s.id = unhex(replace(legacy_source.id, '-', ''))
                     WHERE legacy_source.question_id = eq.id
                     ORDER BY s.id
                     LIMIT 1)
                ),
                eq.prompt,
                COALESCE(eq.answer, ''),
                eq.analysis,
                eq.error_note
            FROM error_questions eq
            "#,
        )
        .await?;

    connection
        .execute_unprepared(
            r#"
            INSERT INTO srs_data (
                question_id, created_at, updated_at, deleted_at, sync_status, sync_version,
                stability, difficulty, next_review_at, last_reviewed_at, review_count,
                feedback_history
            )
            SELECT
                unhex(replace(legacy.question_id, '-', '')),
                datetime(legacy.created_at, 'unixepoch'),
                datetime(legacy.updated_at, 'unixepoch'),
                CASE WHEN legacy.deleted_at IS NULL THEN NULL ELSE datetime(legacy.deleted_at, 'unixepoch') END,
                CASE legacy.sync_status
                    WHEN 'pending' THEN 'PENDING'
                    WHEN 'synced' THEN 'SYNCED'
                    ELSE 'PENDING'
                END,
                COALESCE(legacy.version, 0),
                legacy.stability,
                legacy.difficulty,
                CASE WHEN legacy.next_review_at IS NULL THEN NULL ELSE datetime(legacy.next_review_at, 'unixepoch') END,
                CASE WHEN legacy.lastreviewed_at IS NULL THEN NULL ELSE datetime(legacy.lastreviewed_at, 'unixepoch') END,
                legacy.review_count,
                CASE
                    WHEN json_valid(legacy.feedback_history) THEN legacy.feedback_history
                    ELSE '[]'
                END
            FROM srs_data_legacy_table legacy
            JOIN question q ON q.id = unhex(replace(legacy.question_id, '-', ''))
            "#,
        )
        .await?;

    migrate_attachments(connection).await?;

    connection
        .execute_unprepared(
            r#"
            INSERT OR IGNORE INTO question_attachment_cross_ref (question_id, attachment_id)
            SELECT
                unhex(replace(a.question_id, '-', '')),
                unhex(replace(a.id, '-', ''))
            FROM attachments a
            JOIN question q ON q.id = unhex(replace(a.question_id, '-', ''))
            JOIN attachment normalized_attachment
              ON normalized_attachment.id = unhex(replace(a.id, '-', ''))
            "#,
        )
        .await?;

    connection
        .execute_unprepared(
            r#"
            INSERT INTO tag (
                id, created_at, updated_at, deleted_at, sync_status, sync_version, name, color
            )
            SELECT
                unhex(replace(id, '-', '')),
                datetime(created_at, 'unixepoch'),
                datetime(updated_at, 'unixepoch'),
                CASE WHEN deleted_at IS NULL THEN NULL ELSE datetime(deleted_at, 'unixepoch') END,
                CASE sync_status
                    WHEN 'pending' THEN 'PENDING'
                    WHEN 'synced' THEN 'SYNCED'
                    ELSE 'PENDING'
                END,
                COALESCE(version, 0),
                name,
                COALESCE(color, '')
            FROM error_tags
            "#,
        )
        .await?;

    connection
        .execute_unprepared(
            r#"
            INSERT OR IGNORE INTO question_tag_cross_ref (question_id, tag_id)
            SELECT
                unhex(replace(t.question_id, '-', '')),
                unhex(replace(t.id, '-', ''))
            FROM error_tags t
            JOIN question q ON q.id = unhex(replace(t.question_id, '-', ''))
            JOIN tag normalized_tag
              ON normalized_tag.id = unhex(replace(t.id, '-', ''))
            "#,
        )
        .await?;

    Ok(())
}

async fn migrate_attachments<C>(connection: &C) -> Result<(), DbErr>
where
    C: ConnectionTrait,
{
    // Only keep lightweight IDs in memory. Legacy attachment payloads can be
    // several megabytes each, so fetching every Base64 BLOB at once can OOM.
    let attachment_ids = connection
        .query_all_raw(Statement::from_string(
            DbBackend::Sqlite,
            "SELECT id FROM attachments ORDER BY id",
        ))
        .await?;

    for id_row in attachment_ids {
        let id: String = id_row.try_get("", "id")?;
        let row = connection
            .query_one_raw(Statement::from_sql_and_values(
                DbBackend::Sqlite,
                r#"
            SELECT
                id,
                datetime(created_at, 'unixepoch') AS created_at,
                datetime(updated_at, 'unixepoch') AS updated_at,
                CASE WHEN deleted_at IS NULL THEN NULL ELSE datetime(deleted_at, 'unixepoch') END AS deleted_at,
                CASE sync_status
                    WHEN 'pending' THEN 'PENDING'
                    WHEN 'synced' THEN 'SYNCED'
                    ELSE 'PENDING'
                END AS sync_status,
                COALESCE(version, 0) AS sync_version,
                COALESCE(file_type, '') AS file_type,
                base64_data
            FROM attachments
            WHERE id = ?
            "#,
                [id.clone().into()],
            ))
            .await?
            .ok_or_else(|| DbErr::Migration(format!("attachment disappeared during migration: {id}")))?;
        let encoded: Vec<u8> = row.try_get("", "base64_data")?;
        let data = decode_legacy_attachment(&id, &encoded)?;
        let legacy_file_type: String = row.try_get("", "file_type")?;
        let mime_type = normalized_mime_type(&data, &legacy_file_type);
        let sha256 = attachment_sha256(&data);

        connection
            .execute_raw(Statement::from_sql_and_values(
                DbBackend::Sqlite,
                r#"
                INSERT INTO attachment (
                    id, created_at, updated_at, deleted_at, sync_status, sync_version,
                    mime_type, data, sha256
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#,
                [
                    uuid_blob(&id, "attachment id")?.into(),
                    row.try_get::<String>("", "created_at")?.into(),
                    row.try_get::<String>("", "updated_at")?.into(),
                    row.try_get::<Option<String>>("", "deleted_at")?.into(),
                    row.try_get::<String>("", "sync_status")?.into(),
                    row.try_get::<i64>("", "sync_version")?.into(),
                    mime_type.into(),
                    data.into(),
                    sha256.into(),
                ],
            ))
            .await?;
    }

    Ok(())
}

fn decode_legacy_attachment(id: &str, value: &[u8]) -> Result<Vec<u8>, DbErr> {
    let encoded = std::str::from_utf8(value).map_err(|error| {
        DbErr::Migration(format!(
            "attachment {id} contains non-UTF-8 base64 data: {error}"
        ))
    })?;
    let payload = encoded
        .trim()
        .strip_prefix("data:")
        .and_then(|data_url| data_url.split_once(',').map(|(_, payload)| payload))
        .unwrap_or_else(|| encoded.trim());
    let compact: String = payload.chars().filter(|c| !c.is_whitespace()).collect();
    general_purpose::STANDARD
        .decode(&compact)
        .or_else(|_| general_purpose::STANDARD_NO_PAD.decode(&compact))
        .map_err(|error| {
            DbErr::Migration(format!(
                "attachment {id} contains invalid base64 data: {error}"
            ))
        })
}

fn normalized_mime_type(data: &[u8], legacy_hint: &str) -> String {
    if let Some(kind) = infer::get(data) {
        return kind.mime_type().to_owned();
    }

    let hint = legacy_hint.trim().to_ascii_lowercase();
    if hint.contains('/') {
        return hint;
    }
    match hint.as_str() {
        "jpg" | "jpeg" => "image/jpeg".to_owned(),
        "png" => "image/png".to_owned(),
        "gif" => "image/gif".to_owned(),
        "webp" => "image/webp".to_owned(),
        "svg" => "image/svg+xml".to_owned(),
        "pdf" => "application/pdf".to_owned(),
        _ => "application/octet-stream".to_owned(),
    }
}

fn attachment_sha256(data: &[u8]) -> String {
    let mut output = String::with_capacity(64);
    for byte in Sha256::digest(data) {
        write!(&mut output, "{byte:02x}").expect("writing to a String cannot fail");
    }
    output
}

async fn drop_legacy_tables(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    for table in [
        "attachments",
        "error_tags",
        "srs_data_legacy_table",
        "error_questions",
        "sources",
        "subjects",
        "user_config",
    ] {
        manager
            .drop_table(Table::drop().table(Alias::new(table)).to_owned())
            .await?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm_migration::{
        async_trait::async_trait,
        sea_orm::{ConnectionTrait, Database, DatabaseConnection, Statement},
    };

    const TEST_UUIDS: &[(&str, &str)] = &[
        ("subject-valid", "00000000-0000-4000-8000-000000000001"),
        ("subject-missing", "00000000-0000-4000-8000-000000000002"),
        ("source-missing", "00000000-0000-4000-8000-000000000100"),
        ("source-fallback", "00000000-0000-4000-8000-000000000101"),
        (
            "source-question-fallback",
            "00000000-0000-4000-8000-000000000102",
        ),
        (
            "source-reference-fallback",
            "00000000-0000-4000-8000-000000000103",
        ),
        (
            "source-without-subject",
            "00000000-0000-4000-8000-000000000104",
        ),
        ("question-valid", "00000000-0000-4000-8000-000000000201"),
        ("question-missing", "00000000-0000-4000-8000-000000000202"),
        ("question-type-01", "00000000-0000-4000-8000-000000000211"),
        ("question-type-02", "00000000-0000-4000-8000-000000000212"),
        ("question-type-03", "00000000-0000-4000-8000-000000000213"),
        ("question-type-04", "00000000-0000-4000-8000-000000000214"),
        ("question-type-05", "00000000-0000-4000-8000-000000000215"),
        ("question-type-06", "00000000-0000-4000-8000-000000000216"),
        ("question-type-07", "00000000-0000-4000-8000-000000000217"),
        ("question-type-08", "00000000-0000-4000-8000-000000000218"),
        ("question-type-09", "00000000-0000-4000-8000-000000000219"),
        ("question-type-10", "00000000-0000-4000-8000-000000000220"),
        ("srs-valid", "00000000-0000-4000-8000-000000000301"),
        ("srs-orphan", "00000000-0000-4000-8000-000000000302"),
        ("attachment-valid", "00000000-0000-4000-8000-000000000401"),
        ("attachment-orphan", "00000000-0000-4000-8000-000000000402"),
        ("tag-valid", "00000000-0000-4000-8000-000000000501"),
        ("tag-orphan", "00000000-0000-4000-8000-000000000502"),
    ];

    fn legacy_test_sql(sql: &str) -> String {
        TEST_UUIDS
            .iter()
            .fold(sql.to_owned(), |sql, (name, uuid)| sql.replace(name, uuid))
    }

    fn normalized_test_sql(sql: &str) -> String {
        TEST_UUIDS.iter().fold(sql.to_owned(), |sql, (name, uuid)| {
            sql.replace(
                &format!("'{name}'"),
                &format!("X'{}'", uuid.replace('-', "")),
            )
        })
    }

    fn test_name_for_uuid(value: &[u8]) -> Option<&'static str> {
        let uuid = Uuid::from_slice(value).ok()?.hyphenated().to_string();
        TEST_UUIDS
            .iter()
            .find_map(|(name, expected)| uuid.eq_ignore_ascii_case(expected).then_some(*name))
    }

    fn test_uuid(name: &str) -> &'static str {
        TEST_UUIDS
            .iter()
            .find_map(|(candidate, uuid)| (*candidate == name).then_some(*uuid))
            .unwrap()
    }

    async fn setup_legacy_database() -> DatabaseConnection {
        let database = Database::connect("sqlite::memory:").await.unwrap();
        database
            .execute_unprepared(&legacy_test_sql(
                r#"
                PRAGMA foreign_keys = ON;

                CREATE TABLE user_config (username TEXT PRIMARY KEY NOT NULL);
                INSERT INTO user_config (username) VALUES ('kept');

                CREATE TABLE subjects (
                    id TEXT PRIMARY KEY NOT NULL,
                    name TEXT NOT NULL,
                    color TEXT,
                    created_at INTEGER NOT NULL,
                    updated_at INTEGER NOT NULL,
                    deleted_at INTEGER,
                    version INTEGER NOT NULL,
                    sync_status TEXT NOT NULL,
                    sync_hash TEXT
                );
                CREATE TABLE error_questions (
                    id TEXT PRIMARY KEY NOT NULL,
                    userid TEXT NOT NULL,
                    subjectid TEXT NOT NULL,
                    prompt TEXT NOT NULL,
                    "type" TEXT NOT NULL,
                    answer TEXT,
                    analysis TEXT,
                    error_note TEXT,
                    sourceid TEXT,
                    created_at INTEGER NOT NULL,
                    updated_at INTEGER NOT NULL,
                    deleted_at INTEGER,
                    version INTEGER NOT NULL,
                    sync_status TEXT NOT NULL,
                    sync_hash TEXT
                );
                CREATE TABLE srs_data (
                    id TEXT PRIMARY KEY NOT NULL,
                    question_id TEXT NOT NULL UNIQUE,
                    stability REAL NOT NULL,
                    difficulty REAL NOT NULL,
                    next_review_at INTEGER,
                    lastreviewed_at INTEGER,
                    review_count INTEGER NOT NULL,
                    feedback_history TEXT NOT NULL,
                    created_at INTEGER NOT NULL,
                    updated_at INTEGER NOT NULL,
                    version INTEGER NOT NULL,
                    sync_status TEXT NOT NULL,
                    sync_hash TEXT,
                    deleted_at INTEGER
                );
                CREATE TABLE error_tags (
                    id TEXT PRIMARY KEY NOT NULL,
                    question_id TEXT NOT NULL,
                    name TEXT NOT NULL,
                    color TEXT NOT NULL,
                    created_at INTEGER NOT NULL,
                    updated_at INTEGER NOT NULL,
                    deleted_at INTEGER,
                    version INTEGER NOT NULL,
                    sync_status TEXT NOT NULL,
                    sync_hash TEXT
                );
                CREATE TABLE attachments (
                    id TEXT PRIMARY KEY NOT NULL,
                    question_id TEXT NOT NULL,
                    "type" TEXT NOT NULL,
                    file_type TEXT NOT NULL,
                    base64_data BLOB NOT NULL,
                    hash TEXT NOT NULL,
                    created_at INTEGER NOT NULL,
                    updated_at INTEGER NOT NULL,
                    deleted_at INTEGER,
                    version INTEGER NOT NULL,
                    sync_status TEXT NOT NULL,
                    sync_hash TEXT
                );
                CREATE TABLE sources (
                    id TEXT PRIMARY KEY NOT NULL,
                    question_id TEXT,
                    subject_id TEXT,
                    book TEXT,
                    chapter TEXT,
                    knowledge TEXT,
                    created_at INTEGER NOT NULL,
                    updated_at INTEGER NOT NULL,
                    deleted_at INTEGER,
                    version INTEGER NOT NULL,
                    sync_status TEXT NOT NULL,
                    sync_hash TEXT
                );

                INSERT INTO subjects VALUES
                    ('subject-valid', 'Math', NULL, 1, 2, NULL, 3, 'pending', 'discarded');
                INSERT INTO sources VALUES
                    ('source-fallback', 'question-valid', 'subject-valid', 'Book', 'Chapter',
                     'Knowledge', 1, 2, NULL, 4, 'synced', 'discarded'),
                    ('source-question-fallback', 'question-type-01', 'subject-missing', NULL,
                     NULL, NULL, 1, 2, NULL, 0, 'synced', NULL),
                    ('source-reference-fallback', NULL, NULL, NULL, NULL, NULL,
                     1, 2, NULL, 0, 'synced', NULL),
                    ('source-without-subject', 'question-valid', 'subject-missing', NULL,
                     NULL, NULL, 1, 2, NULL, 0, 'synced', NULL);
                INSERT INTO error_questions VALUES
                    ('question-valid', 'user', 'subject-missing', 'What?', '未知题型', NULL,
                     'Because', 'A note', 'source-missing', 1, 2, NULL, 5, 'pending', 'discarded'),
                    ('question-type-01', 'user', 'subject-valid', 'Q1', '单选题', NULL,
                     NULL, NULL, NULL, 1, 2, NULL, 0, 'synced', NULL),
                    ('question-type-02', 'user', 'subject-valid', 'Q2', '多选题', NULL,
                     NULL, NULL, 'source-reference-fallback', 1, 2, NULL, 0, 'synced', NULL),
                    ('question-type-03', 'user', 'subject-valid', 'Q3', '判断题', NULL,
                     NULL, NULL, NULL, 1, 2, NULL, 0, 'synced', NULL),
                    ('question-type-04', 'user', 'subject-valid', 'Q4', '填空题', NULL,
                     NULL, NULL, NULL, 1, 2, NULL, 0, 'synced', NULL),
                    ('question-type-05', 'user', 'subject-valid', 'Q5', '简答题', NULL,
                     NULL, NULL, NULL, 1, 2, NULL, 0, 'synced', NULL),
                    ('question-type-06', 'user', 'subject-valid', 'Q6', '计算题', NULL,
                     NULL, NULL, NULL, 1, 2, NULL, 0, 'synced', NULL),
                    ('question-type-07', 'user', 'subject-valid', 'Q7', '论述题', NULL,
                     NULL, NULL, NULL, 1, 2, NULL, 0, 'unexpected', NULL),
                    ('question-type-08', 'user', 'subject-valid', 'Q8', 'ShortAnswer', NULL,
                     NULL, NULL, NULL, 1, 2, NULL, 0, 'synced', NULL),
                    ('question-type-09', 'user', 'subject-valid', 'Q9', 'SigleChoice', NULL,
                     NULL, NULL, NULL, 1, 2, NULL, 0, 'synced', NULL),
                    ('question-type-10', 'user', 'subject-valid', 'Q10', '', NULL,
                     NULL, NULL, NULL, 1, 2, NULL, 0, 'synced', NULL);
                INSERT INTO srs_data VALUES
                    ('srs-valid', 'question-valid', 3.0, 2.5, 10, 9, 2, 'not-json',
                     1, 2, 6, 'synced', 'discarded', NULL),
                    ('srs-orphan', 'question-missing', 3.0, 2.5, NULL, NULL, 0, '[]',
                     1, 2, 0, 'synced', NULL, NULL);
                INSERT INTO attachments VALUES
                    ('attachment-valid', 'question-valid', 'image', 'image', CAST('/9j/2Q==' AS BLOB),
                     'abc', 1, 2, NULL, 7, 'pending', NULL),
                    ('attachment-orphan', 'question-missing', 'image', 'image/png', CAST('Aw==' AS BLOB),
                     'def', 1, 2, NULL, 0, 'synced', NULL);
                INSERT INTO error_tags VALUES
                    ('tag-valid', 'question-valid', 'Important', '#fff', 1, 2, NULL, 8, 'pending', NULL),
                    ('tag-orphan', 'question-missing', 'Orphan', '#000', 1, 2, NULL, 0, 'unexpected', NULL);
                "#,
            ))
            .await
            .unwrap();
        database
    }

    async fn scalar_i64(database: &DatabaseConnection, sql: &str) -> i64 {
        database
            .query_one_raw(Statement::from_string(
                DbBackend::Sqlite,
                normalized_test_sql(sql),
            ))
            .await
            .unwrap()
            .unwrap()
            .try_get_by_index(0)
            .unwrap()
    }

    async fn scalar_string(database: &DatabaseConnection, sql: &str) -> String {
        let row = database
            .query_one_raw(Statement::from_string(
                DbBackend::Sqlite,
                normalized_test_sql(sql),
            ))
            .await
            .unwrap()
            .unwrap();
        if let Ok(value) = row.try_get_by_index(0) {
            return value;
        }
        let value: Vec<u8> = row.try_get_by_index(0).unwrap();
        test_name_for_uuid(&value)
            .map(str::to_owned)
            .unwrap_or_else(|| Uuid::from_slice(&value).unwrap().hyphenated().to_string())
    }

    async fn remove_orphan_srs(database: &DatabaseConnection) {
        database
            .execute_unprepared(&legacy_test_sql(
                "DELETE FROM srs_data WHERE id = 'srs-orphan'",
            ))
            .await
            .unwrap();
    }

    struct TestMigrator;

    #[async_trait]
    impl MigratorTrait for TestMigrator {
        fn migrations() -> Vec<Box<dyn MigrationTrait>> {
            vec![Box::new(Migration)]
        }
    }

    #[tokio::test]
    async fn normalizes_legacy_data_and_relationships() {
        let database = setup_legacy_database().await;
        remove_orphan_srs(&database).await;
        TestMigrator::up(&database, None).await.unwrap();

        for sql in [
            "SELECT COUNT(*) FROM subject WHERE typeof(id) != 'blob' OR length(id) != 16",
            "SELECT COUNT(*) FROM source WHERE typeof(id) != 'blob' OR length(id) != 16 OR \
             (subject_id IS NOT NULL AND (typeof(subject_id) != 'blob' OR length(subject_id) != 16))",
            "SELECT COUNT(*) FROM question WHERE typeof(id) != 'blob' OR length(id) != 16 OR \
             (source_id IS NOT NULL AND (typeof(source_id) != 'blob' OR length(source_id) != 16))",
            "SELECT COUNT(*) FROM srs_data WHERE typeof(question_id) != 'blob' OR length(question_id) != 16",
            "SELECT COUNT(*) FROM attachment WHERE typeof(id) != 'blob' OR length(id) != 16",
            "SELECT COUNT(*) FROM question_attachment_cross_ref WHERE \
             typeof(question_id) != 'blob' OR length(question_id) != 16 OR \
             typeof(attachment_id) != 'blob' OR length(attachment_id) != 16",
            "SELECT COUNT(*) FROM tag WHERE typeof(id) != 'blob' OR length(id) != 16",
            "SELECT COUNT(*) FROM question_tag_cross_ref WHERE \
             typeof(question_id) != 'blob' OR length(question_id) != 16 OR \
             typeof(tag_id) != 'blob' OR length(tag_id) != 16",
        ] {
            assert_eq!(scalar_i64(&database, sql).await, 0, "{sql}");
        }
        assert_eq!(
            scalar_i64(
                &database,
                "SELECT COUNT(*) FROM (\
                 SELECT type FROM pragma_table_info('subject') WHERE name = 'id' \
                 UNION ALL SELECT type FROM pragma_table_info('source') WHERE name IN ('id', 'subject_id') \
                 UNION ALL SELECT type FROM pragma_table_info('question') WHERE name IN ('id', 'source_id') \
                 UNION ALL SELECT type FROM pragma_table_info('srs_data') WHERE name = 'question_id' \
                 UNION ALL SELECT type FROM pragma_table_info('attachment') WHERE name = 'id' \
                 UNION ALL SELECT type FROM pragma_table_info('question_attachment_cross_ref') \
                   WHERE name IN ('question_id', 'attachment_id') \
                 UNION ALL SELECT type FROM pragma_table_info('tag') WHERE name = 'id' \
                 UNION ALL SELECT type FROM pragma_table_info('question_tag_cross_ref') \
                   WHERE name IN ('question_id', 'tag_id')) WHERE lower(type) = 'uuid_text'"
            )
            .await,
            12
        );
        let decoded_subject: Uuid = database
            .query_one_raw(Statement::from_string(
                DbBackend::Sqlite,
                "SELECT id FROM subject LIMIT 1",
            ))
            .await
            .unwrap()
            .unwrap()
            .try_get_by_index(0)
            .unwrap();
        assert_eq!(
            decoded_subject.hyphenated().to_string(),
            test_uuid("subject-valid")
        );

        assert_eq!(
            scalar_string(
                &database,
                "SELECT color FROM subject WHERE id = 'subject-valid'"
            )
            .await,
            ""
        );
        assert_eq!(
            scalar_string(
                &database,
                "SELECT created_at FROM subject WHERE id = 'subject-valid'"
            )
            .await,
            "1970-01-01 00:00:01"
        );
        assert_eq!(
            scalar_i64(
                &database,
                "SELECT sync_version FROM question WHERE id = 'question-valid'"
            )
            .await,
            5
        );
        assert_eq!(
            scalar_string(
                &database,
                "SELECT correct_answer FROM question WHERE id = 'question-valid'"
            )
            .await,
            ""
        );
        assert_eq!(
            scalar_i64(
                &database,
                "SELECT question_type IS NULL FROM question WHERE id = 'question-valid'"
            )
            .await,
            1
        );
        assert_eq!(
            scalar_string(
                &database,
                "SELECT group_concat(question_type, ',') FROM (\
                 SELECT question_type FROM question \
                 WHERE stem LIKE 'Q%' ORDER BY CAST(substr(stem, 2) AS INTEGER))"
            )
            .await,
            "SINGLE_SELECT,MULTIPLE_SELECT,TRUE_FALSE,FILL_IN_THE_BLANK,\
             SHORT_ANSWER,CALCULATION,ESSAY,SHORT_ANSWER,SINGLE_SELECT"
        );
        assert_eq!(
            scalar_i64(
                &database,
                "SELECT question_type IS NULL FROM question WHERE id = 'question-type-10'"
            )
            .await,
            1
        );
        assert_eq!(
            scalar_i64(
                &database,
                "SELECT COUNT(*) FROM pragma_table_info('question') WHERE name = 'subject_id'"
            )
            .await,
            0
        );
        for (source_id, expected_subject_id) in [
            ("source-fallback", "subject-valid"),
            ("source-question-fallback", "subject-valid"),
            ("source-reference-fallback", "subject-valid"),
        ] {
            assert_eq!(
                scalar_string(
                    &database,
                    &format!("SELECT subject_id FROM source WHERE id = '{source_id}'"),
                )
                .await,
                expected_subject_id
            );
        }
        assert_eq!(
            scalar_i64(
                &database,
                "SELECT subject_id IS NULL FROM source WHERE id = 'source-without-subject'"
            )
            .await,
            1
        );
        assert_eq!(
            scalar_string(
                &database,
                "SELECT source_id FROM question WHERE id = 'question-valid'"
            )
            .await,
            "source-fallback"
        );
        assert_eq!(
            scalar_string(
                &database,
                "SELECT source_id FROM question WHERE id = 'question-type-01'"
            )
            .await,
            "source-question-fallback"
        );
        assert_eq!(
            scalar_string(
                &database,
                "SELECT source_id FROM question WHERE id = 'question-type-02'"
            )
            .await,
            "source-reference-fallback"
        );
        assert_eq!(
            scalar_string(
                &database,
                "SELECT feedback_history FROM srs_data WHERE question_id = 'question-valid'"
            )
            .await,
            "[]"
        );
        for (sql, expected) in [
            (
                "SELECT sync_status FROM subject WHERE id = 'subject-valid'",
                "PENDING",
            ),
            (
                "SELECT sync_status FROM source WHERE id = 'source-fallback'",
                "SYNCED",
            ),
            (
                "SELECT sync_status FROM question WHERE id = 'question-valid'",
                "PENDING",
            ),
            (
                "SELECT sync_status FROM srs_data WHERE question_id = 'question-valid'",
                "SYNCED",
            ),
            (
                "SELECT sync_status FROM attachment WHERE id = 'attachment-valid'",
                "PENDING",
            ),
            (
                "SELECT sync_status FROM tag WHERE id = 'tag-valid'",
                "PENDING",
            ),
            (
                "SELECT sync_status FROM question WHERE id = 'question-type-07'",
                "PENDING",
            ),
            (
                "SELECT sync_status FROM tag WHERE id = 'tag-orphan'",
                "PENDING",
            ),
        ] {
            assert_eq!(scalar_string(&database, sql).await, expected);
        }
        assert_eq!(
            scalar_i64(&database, "SELECT COUNT(*) FROM srs_data").await,
            1
        );
        assert_eq!(
            scalar_i64(&database, "SELECT COUNT(*) FROM attachment").await,
            2
        );
        assert_eq!(
            scalar_i64(
                &database,
                "SELECT COUNT(*) FROM question_attachment_cross_ref"
            )
            .await,
            1
        );
        assert_eq!(
            scalar_string(
                &database,
                "SELECT mime_type FROM attachment WHERE id = 'attachment-valid'"
            )
            .await,
            "image/jpeg"
        );
        assert_eq!(
            scalar_string(
                &database,
                "SELECT hex(data) FROM attachment WHERE id = 'attachment-valid'"
            )
            .await,
            "FFD8FFD9"
        );
        assert_eq!(
            scalar_i64(
                &database,
                "SELECT length(sha256) FROM attachment WHERE id = 'attachment-valid'"
            )
            .await,
            64
        );
        assert_eq!(scalar_i64(&database, "SELECT COUNT(*) FROM tag").await, 2);
        assert_eq!(
            scalar_i64(&database, "SELECT COUNT(*) FROM question_tag_cross_ref").await,
            1
        );
        assert_eq!(
            scalar_i64(
                &database,
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name IN \
                 ('subjects', 'sources', 'error_questions', 'srs_data_legacy_table', \
                  'attachments', 'error_tags', 'user_config')"
            )
            .await,
            0
        );

        database
            .execute_unprepared(&normalized_test_sql(
                "DELETE FROM subject WHERE id = 'subject-valid'",
            ))
            .await
            .unwrap();
        assert_eq!(
            scalar_i64(
                &database,
                "SELECT COUNT(*) FROM source WHERE subject_id IS NOT NULL"
            )
            .await,
            0
        );

        database
            .execute_unprepared(&normalized_test_sql(
                "DELETE FROM source WHERE id = 'source-fallback'",
            ))
            .await
            .unwrap();
        assert_eq!(
            scalar_i64(
                &database,
                "SELECT source_id IS NULL FROM question WHERE id = 'question-valid'"
            )
            .await,
            1
        );
        database
            .execute_unprepared(&normalized_test_sql(
                "DELETE FROM question WHERE id = 'question-valid'",
            ))
            .await
            .unwrap();
        assert_eq!(
            scalar_i64(&database, "SELECT COUNT(*) FROM srs_data").await,
            0
        );
        assert_eq!(
            scalar_i64(
                &database,
                "SELECT COUNT(*) FROM question_attachment_cross_ref"
            )
            .await,
            0
        );
        assert_eq!(
            scalar_i64(&database, "SELECT COUNT(*) FROM question_tag_cross_ref").await,
            0
        );

        let error = TestMigrator::down(&database, Some(1)).await.unwrap_err();
        assert!(error.to_string().contains("cannot be rolled back"));
        assert_eq!(
            scalar_i64(&database, "SELECT COUNT(*) FROM attachment").await,
            2
        );
        assert_eq!(
            scalar_i64(
                &database,
                "SELECT COUNT(*) FROM seaql_migrations \
                 WHERE version = 'm20260812_100149_normalize_database'"
            )
            .await,
            1
        );
    }

    #[tokio::test]
    async fn rolls_back_all_changes_when_migration_fails() {
        let database = setup_legacy_database().await;
        remove_orphan_srs(&database).await;
        database
            .execute_unprepared(&legacy_test_sql(
                "UPDATE sources SET created_at = 'invalid-unix-timestamp' \
                 WHERE id = 'source-fallback'",
            ))
            .await
            .unwrap();

        let error = TestMigrator::up(&database, None).await.unwrap_err();
        assert!(error.to_string().contains("NOT NULL constraint failed"));

        assert_eq!(
            scalar_i64(
                &database,
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name IN \
                 ('subjects', 'sources', 'error_questions', 'srs_data', \
                  'attachments', 'error_tags', 'user_config')"
            )
            .await,
            7
        );
        assert_eq!(
            scalar_i64(
                &database,
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name IN \
                 ('subject', 'source', 'question', 'srs_data_legacy_table', 'attachment', \
                  'question_attachment_cross_ref', 'tag', 'question_tag_cross_ref')"
            )
            .await,
            0
        );
        assert_eq!(
            scalar_i64(&database, "SELECT COUNT(*) FROM subjects").await,
            1
        );
        assert_eq!(
            scalar_i64(&database, "SELECT COUNT(*) FROM sources").await,
            4
        );
        assert_eq!(
            scalar_i64(&database, "SELECT COUNT(*) FROM srs_data").await,
            1
        );
        assert_eq!(
            scalar_i64(&database, "SELECT COUNT(*) FROM user_config").await,
            1
        );
        assert_eq!(
            scalar_i64(
                &database,
                "SELECT COUNT(*) FROM seaql_migrations \
                 WHERE version = 'm20260812_100149_normalize_database'"
            )
            .await,
            0
        );
        assert_eq!(
            TestMigrator::get_pending_migrations(&database)
                .await
                .unwrap()
                .len(),
            1
        );
    }

    #[tokio::test]
    async fn rolls_back_when_an_attachment_is_not_valid_base64() {
        let database = setup_legacy_database().await;
        remove_orphan_srs(&database).await;
        database
            .execute_unprepared(&legacy_test_sql(
                "UPDATE attachments SET base64_data = X'FF' WHERE id = 'attachment-valid'",
            ))
            .await
            .unwrap();

        let error = TestMigrator::up(&database, None).await.unwrap_err();
        assert!(error.to_string().contains(test_uuid("attachment-valid")));
        assert!(error.to_string().contains("base64"));
        assert_eq!(
            scalar_i64(&database, "SELECT COUNT(*) FROM attachments").await,
            2
        );
        assert_eq!(
            scalar_i64(
                &database,
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'attachment'"
            )
            .await,
            0
        );
        assert_eq!(
            scalar_i64(
                &database,
                "SELECT COUNT(*) FROM seaql_migrations WHERE version = 'm20260812_100149_normalize_database'"
            )
            .await,
            0
        );
    }

    #[tokio::test]
    async fn rejects_invalid_uuid_values_before_schema_changes() {
        for (update, expected) in [
            (
                "UPDATE subjects SET id = 'not-a-uuid' WHERE id = 'subject-valid'",
                "subjects.id=not-a-uuid",
            ),
            (
                "UPDATE sources SET question_id = 'not-a-uuid' \
                 WHERE id = 'source-fallback'",
                "sources.question_id=not-a-uuid",
            ),
        ] {
            let database = setup_legacy_database().await;
            remove_orphan_srs(&database).await;
            database
                .execute_unprepared(&legacy_test_sql(update))
                .await
                .unwrap();

            let error = TestMigrator::up(&database, None).await.unwrap_err();
            let message = error.to_string();
            assert!(message.contains("found 1 invalid UUID value"), "{message}");
            assert!(message.contains(expected), "{message}");
            assert_eq!(
                scalar_i64(
                    &database,
                    "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name IN \
                     ('subjects', 'sources', 'error_questions', 'srs_data', \
                      'attachments', 'error_tags', 'user_config')"
                )
                .await,
                7
            );
            assert_eq!(
                scalar_i64(
                    &database,
                    "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name IN \
                     ('subject', 'source', 'question', 'srs_data_legacy_table', 'attachment', \
                      'question_attachment_cross_ref', 'tag', 'question_tag_cross_ref')"
                )
                .await,
                0
            );
            assert_eq!(
                scalar_i64(
                    &database,
                    "SELECT COUNT(*) FROM seaql_migrations \
                     WHERE version = 'm20260812_100149_normalize_database'"
                )
                .await,
                0
            );
        }
    }

    #[tokio::test]
    async fn rejects_orphan_srs_before_schema_changes() {
        let database = setup_legacy_database().await;

        let error = TestMigrator::up(&database, None).await.unwrap_err();
        let message = error.to_string();
        assert!(message.contains("found 1 orphan SRS record"));
        assert!(message.contains(&format!(
            "{}/{}",
            test_uuid("srs-orphan"),
            test_uuid("question-missing")
        )));

        assert_eq!(
            scalar_i64(
                &database,
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name IN \
                 ('subjects', 'sources', 'error_questions', 'srs_data', \
                  'attachments', 'error_tags', 'user_config')"
            )
            .await,
            7
        );
        assert_eq!(
            scalar_i64(
                &database,
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name IN \
                 ('subject', 'source', 'question', 'srs_data_legacy_table', 'attachment', \
                  'question_attachment_cross_ref', 'tag', 'question_tag_cross_ref')"
            )
            .await,
            0
        );
        assert_eq!(
            scalar_i64(&database, "SELECT COUNT(*) FROM srs_data").await,
            2
        );
        assert_eq!(
            scalar_i64(
                &database,
                "SELECT COUNT(*) FROM seaql_migrations \
                 WHERE version = 'm20260812_100149_normalize_database'"
            )
            .await,
            0
        );
        assert_eq!(
            TestMigrator::get_pending_migrations(&database)
                .await
                .unwrap()
                .len(),
            1
        );
    }
}
