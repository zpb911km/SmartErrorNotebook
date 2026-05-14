use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(ErrorQuestions::Table)
                    .drop_column(ErrorQuestions::Prompt)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(ErrorQuestions::Table)
                    .add_column(ColumnDef::new(ErrorQuestions::Prompt).text().not_null())
                    .to_owned(),
            )
            .await?;
        

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}

#[derive(DeriveIden)]
enum ErrorQuestions {
    Table,
    Prompt,
}
