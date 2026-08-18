// 重新创建 sources 表，移除外键约束

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 先删除旧表（如果存在）
        manager
            .drop_table(Table::drop().table(Sources::Table).if_exists().to_owned())
            .await?;

        // 创建新表，不包含外键约束
        manager
            .create_table(
                Table::create()
                    .table(Sources::Table)
                    .col(
                        ColumnDef::new(Sources::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Sources::QuestionId).string()) // 可选，无外键约束
                    .col(ColumnDef::new(Sources::SubjectId).string()) // 可选，无外键约束
                    .col(ColumnDef::new(Sources::Book).string())
                    .col(ColumnDef::new(Sources::Chapter).string())
                    .col(ColumnDef::new(Sources::Knowledge).string())
                    .col(ColumnDef::new(Sources::CreatedAt).big_integer().not_null())
                    .col(ColumnDef::new(Sources::UpdatedAt).big_integer().not_null())
                    .col(ColumnDef::new(Sources::DeletedAt).big_integer())
                    .col(
                        ColumnDef::new(Sources::Version)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Sources::SyncStatus)
                            .string()
                            .not_null()
                            .default("synced"),
                    )
                    .col(ColumnDef::new(Sources::SyncHash).string())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Sources::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Sources {
    Table,
    Id,
    QuestionId,
    SubjectId,
    Book,
    Chapter,
    Knowledge,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
    Version,
    SyncStatus,
    SyncHash,
}
