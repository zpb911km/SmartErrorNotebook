// 修改 attachments 表的 base64_data 列类型从 text 改为 blob（支持大图片）

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 先删除旧的 base64_data 列
        manager
            .alter_table(
                Table::alter()
                    .table(Attachments::Table)
                    .drop_column(Attachments::Base64Data)
                    .to_owned(),
            )
            .await?;

        // 添加新的 BLOB 类型的 base64_data 列
        // blob() 在 MySQL 中生成 MEDIUMBLOB，在 SQLite 中生成 BLOB
        manager
            .alter_table(
                Table::alter()
                    .table(Attachments::Table)
                    .add_column(ColumnDef::new(Attachments::Base64Data).blob().not_null())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 回滚：删除 BLOB 列
        manager
            .alter_table(
                Table::alter()
                    .table(Attachments::Table)
                    .drop_column(Attachments::Base64Data)
                    .to_owned(),
            )
            .await?;

        // 添加回 TEXT 列
        manager
            .alter_table(
                Table::alter()
                    .table(Attachments::Table)
                    .add_column(ColumnDef::new(Attachments::Base64Data).text().not_null())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Attachments {
    Table,
    Base64Data,
}
