// 创建科目表

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Subjects::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Subjects::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Subjects::Name).string().not_null())
                    .col(ColumnDef::new(Subjects::Color).string())
                    .col(ColumnDef::new(Subjects::CreatedAt).big_integer().not_null())
                    .col(ColumnDef::new(Subjects::UpdatedAt).big_integer().not_null())
                    .col(ColumnDef::new(Subjects::DeletedAt).big_integer())
                    .col(
                        ColumnDef::new(Subjects::Version)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Subjects::SyncStatus)
                            .string()
                            .not_null()
                            .default("synced"),
                    )
                    .col(ColumnDef::new(Subjects::SyncHash).string())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Subjects::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Subjects {
    Table,
    Id,
    Name,
    Color,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
    Version,
    SyncStatus,
    SyncHash,
}
