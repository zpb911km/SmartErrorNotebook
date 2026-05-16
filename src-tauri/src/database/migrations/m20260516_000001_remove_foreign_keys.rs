// 移除外键约束 — 重建 error_questions、srs_data、error_tags、attachments 四张表
// 使用 SeaORM builder 建表，只去掉 .foreign_key()，其他列定义与原始迁移保持一致

use sea_orm::ConnectionTrait;
use sea_orm::Statement;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let backend = manager.get_database_backend();

        // 逐一重建四张表
        rebuild_error_questions(manager, db, backend).await?;
        rebuild_srs_data(manager, db, backend).await?;
        rebuild_error_tags(manager, db, backend).await?;
        rebuild_attachments(manager, db, backend).await?;

        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        // down 迁移不做恢复（移除外键是一致性变更，非破坏性）
        Ok(())
    }
}

// ==================== error_questions ====================

#[derive(DeriveIden)]
enum EqCols {
    Table,
    Id,
    Userid,
    Subjectid,
    Prompt,
    #[sea_orm(iden = "type")]
    Type,
    Answer,
    Analysis,
    ErrorNote,
    Sourceid,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
    Version,
    SyncStatus,
    SyncHash,
}

async fn rebuild_error_questions(
    manager: &SchemaManager<'_>,
    db: &dyn ConnectionTrait,
    backend: sea_orm::DatabaseBackend,
) -> Result<(), DbErr> {
    let table = "error_questions";

    // 创建临时表（不含 FK）
    manager
        .create_table(
            Table::create()
                .table(Alias::new(&format!("{}_new", table)))
                .col(ColumnDef::new(EqCols::Id).string().not_null().primary_key())
                .col(ColumnDef::new(EqCols::Userid).string().not_null())
                .col(ColumnDef::new(EqCols::Subjectid).string().not_null())
                .col(ColumnDef::new(EqCols::Prompt).string().not_null())
                .col(ColumnDef::new(EqCols::Type).string().not_null())
                .col(ColumnDef::new(EqCols::Answer).text())
                .col(ColumnDef::new(EqCols::Analysis).text())
                .col(ColumnDef::new(EqCols::ErrorNote).text())
                .col(ColumnDef::new(EqCols::Sourceid).string())
                .col(ColumnDef::new(EqCols::CreatedAt).big_integer().not_null())
                .col(ColumnDef::new(EqCols::UpdatedAt).big_integer().not_null())
                .col(ColumnDef::new(EqCols::DeletedAt).big_integer())
                .col(ColumnDef::new(EqCols::Version).integer().not_null().default(0))
                .col(ColumnDef::new(EqCols::SyncStatus).string().not_null().default("synced"))
                .col(ColumnDef::new(EqCols::SyncHash).string())
                .to_owned(),
        )
        .await?;

    // 显式列名复制数据
    let cols = "id, userid, subjectid, prompt, \"type\", answer, analysis, error_note, sourceid, created_at, updated_at, deleted_at, version, sync_status, sync_hash";
    copy_and_swap(db, backend, table, cols).await
}

// ==================== srs_data ====================

#[derive(DeriveIden)]
enum SrsCols {
    Table,
    Id,
    QuestionId,
    Stability,
    Difficulty,
    NextReviewAt,
    LastreviewedAt,
    ReviewCount,
    FeedbackHistory,
    CreatedAt,
    UpdatedAt,
    Version,
    SyncStatus,
    SyncHash,
}

async fn rebuild_srs_data(
    manager: &SchemaManager<'_>,
    db: &dyn ConnectionTrait,
    backend: sea_orm::DatabaseBackend,
) -> Result<(), DbErr> {
    let table = "srs_data";

    manager
        .create_table(
            Table::create()
                .table(Alias::new(&format!("{}_new", table)))
                .col(ColumnDef::new(SrsCols::Id).string().not_null().primary_key())
                .col(ColumnDef::new(SrsCols::QuestionId).string().not_null().unique_key())
                .col(ColumnDef::new(SrsCols::Stability).float().not_null().default(3.0))
                .col(ColumnDef::new(SrsCols::Difficulty).float().not_null().default(2.5))
                .col(ColumnDef::new(SrsCols::NextReviewAt).big_integer().null())
                .col(ColumnDef::new(SrsCols::LastreviewedAt).big_integer().null())
                .col(ColumnDef::new(SrsCols::ReviewCount).integer().not_null().default(0))
                .col(ColumnDef::new(SrsCols::FeedbackHistory).string().not_null().default("[]"))
                .col(ColumnDef::new(SrsCols::CreatedAt).big_integer().not_null())
                .col(ColumnDef::new(SrsCols::UpdatedAt).big_integer().not_null())
                .col(ColumnDef::new(SrsCols::Version).integer().not_null().default(0))
                .col(ColumnDef::new(SrsCols::SyncStatus).string().not_null().default("synced"))
                .col(ColumnDef::new(SrsCols::SyncHash).string())
                .to_owned(),
        )
        .await?;

    let cols = "id, question_id, stability, difficulty, next_review_at, lastreviewed_at, review_count, feedback_history, created_at, updated_at, version, sync_status, sync_hash";
    copy_and_swap(db, backend, table, cols).await
}

// ==================== error_tags ====================

#[derive(DeriveIden)]
enum TagCols {
    Table,
    Id,
    QuestionId,
    Name,
    Color,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
    Version,
    SyncStatus,
    SyncHash,
}

async fn rebuild_error_tags(
    manager: &SchemaManager<'_>,
    db: &dyn ConnectionTrait,
    backend: sea_orm::DatabaseBackend,
) -> Result<(), DbErr> {
    let table = "error_tags";

    manager
        .create_table(
            Table::create()
                .table(Alias::new(&format!("{}_new", table)))
                .col(ColumnDef::new(TagCols::Id).string().not_null().primary_key())
                .col(ColumnDef::new(TagCols::QuestionId).string().not_null())
                .col(ColumnDef::new(TagCols::Name).string().not_null())
                .col(ColumnDef::new(TagCols::Color).string().not_null())
                .col(ColumnDef::new(TagCols::CreatedAt).big_integer().not_null())
                .col(ColumnDef::new(TagCols::UpdatedAt).big_integer().not_null())
                .col(ColumnDef::new(TagCols::DeletedAt).big_integer())
                .col(ColumnDef::new(TagCols::Version).integer().not_null().default(0))
                .col(ColumnDef::new(TagCols::SyncStatus).string().not_null().default("synced"))
                .col(ColumnDef::new(TagCols::SyncHash).string())
                .to_owned(),
        )
        .await?;

    let cols = "id, question_id, name, color, created_at, updated_at, deleted_at, version, sync_status, sync_hash";
    copy_and_swap(db, backend, table, cols).await
}

// ==================== attachments ====================

#[derive(DeriveIden)]
enum AttCols {
    Table,
    Id,
    QuestionId,
    #[sea_orm(iden = "type")]
    Type,
    FileType,
    Base64Data,
    Hash,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
    Version,
    SyncStatus,
    SyncHash,
}

async fn rebuild_attachments(
    manager: &SchemaManager<'_>,
    db: &dyn ConnectionTrait,
    backend: sea_orm::DatabaseBackend,
) -> Result<(), DbErr> {
    let table = "attachments";

    manager
        .create_table(
            Table::create()
                .table(Alias::new(&format!("{}_new", table)))
                .col(ColumnDef::new(AttCols::Id).string().not_null().primary_key())
                .col(ColumnDef::new(AttCols::QuestionId).string().not_null())
                .col(ColumnDef::new(AttCols::Type).string().not_null())
                .col(ColumnDef::new(AttCols::FileType).string().not_null())
                .col(ColumnDef::new(AttCols::Base64Data).blob().not_null())
                .col(ColumnDef::new(AttCols::Hash).string().not_null())
                .col(ColumnDef::new(AttCols::CreatedAt).big_integer().not_null())
                .col(ColumnDef::new(AttCols::UpdatedAt).big_integer().not_null())
                .col(ColumnDef::new(AttCols::DeletedAt).big_integer())
                .col(ColumnDef::new(AttCols::Version).integer().not_null().default(0))
                .col(ColumnDef::new(AttCols::SyncStatus).string().not_null().default("synced"))
                .col(ColumnDef::new(AttCols::SyncHash).string())
                .to_owned(),
        )
        .await?;

    let cols = "id, question_id, \"type\", file_type, base64_data, hash, created_at, updated_at, deleted_at, version, sync_status, sync_hash";
    copy_and_swap(db, backend, table, cols).await
}

// ==================== 工具函数 ====================

/// 用显式列名复制数据，然后 DROP 旧表、RENAME 新表
async fn copy_and_swap(
    db: &dyn ConnectionTrait,
    backend: sea_orm::DatabaseBackend,
    table_name: &str,
    columns: &str,
) -> Result<(), DbErr> {
    let new = format!("{}_new", table_name);

    // INSERT ... SELECT (显式列名，避免列顺序不匹配)
    let sql = format!("INSERT INTO \"{}\" ({}) SELECT {} FROM \"{}\"", new, columns, columns, table_name);
    db.execute(Statement::from_string(backend, sql)).await?;

    // DROP old
    let sql = format!("DROP TABLE \"{}\"", table_name);
    db.execute(Statement::from_string(backend, sql)).await?;

    // RENAME new → old
    let sql = format!("ALTER TABLE \"{}\" RENAME TO \"{}\"", new, table_name);
    db.execute(Statement::from_string(backend, sql)).await?;

    Ok(())
}
