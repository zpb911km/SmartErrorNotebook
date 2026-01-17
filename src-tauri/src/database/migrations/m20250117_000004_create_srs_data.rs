// 创建 SRS 数据表

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(SrsData::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(SrsData::Id).string().not_null().primary_key())
                    .col(ColumnDef::new(SrsData::QuestionId).string().not_null().unique_key())
                    .col(ColumnDef::new(SrsData::Difficulty).float().not_null().default(2.5))
                    .col(ColumnDef::new(SrsData::Mastery).float().not_null().default(0.0))
                    .col(ColumnDef::new(SrsData::LastreviewedAt).big_integer().not_null())
                    .col(ColumnDef::new(SrsData::ReviewCount).integer().not_null().default(0))
                    .col(ColumnDef::new(SrsData::CreatedAt).big_integer().not_null())
                    .col(ColumnDef::new(SrsData::UpdatedAt).big_integer().not_null())
                    .col(ColumnDef::new(SrsData::Version).integer().not_null().default(0))
                    .col(ColumnDef::new(SrsData::SyncStatus).string().not_null().default("synced"))
                    .col(ColumnDef::new(SrsData::SyncHash).string())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_srs_data_error_questions")
                            .from(SrsData::Table, SrsData::QuestionId)
                            .to(ErrorQuestions::Table, ErrorQuestions::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(SrsData::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum SrsData {
    Table,
    Id,
    QuestionId,
    Difficulty,
    Mastery,
    LastreviewedAt,
    ReviewCount,
    CreatedAt,
    UpdatedAt,
    Version,
    SyncStatus,
    SyncHash,
}

#[derive(DeriveIden)]
enum ErrorQuestions {
    Table,
    Id,
}