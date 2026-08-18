use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        create_user_config(manager).await?;
        create_subjects(manager).await?;
        create_error_questions(manager).await?;
        create_srs_data(manager).await?;
        create_error_tags(manager).await?;
        create_attachments(manager).await?;
        create_sources(manager).await?;

        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        // The legacy migrations own these tables. Rolling back this compatibility
        // marker must not remove tables or data from an upgraded database.
        Ok(())
    }
}

fn column(name: &str) -> ColumnDef {
    ColumnDef::new(name.to_owned())
}

async fn create_user_config(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table("user_config")
                .if_not_exists()
                .col(column("username").string().not_null().primary_key())
                .col(column("email").string().not_null())
                .col(column("student_num").string().not_null())
                .col(column("phone").string())
                .col(column("avatar").string())
                .col(column("theme").string())
                .col(column("password_hash").string())
                .col(column("ai_base_url").string())
                .col(column("ai_key").string())
                .col(column("sync").boolean().not_null().default(false))
                .col(column("created_at").big_integer().not_null())
                .col(column("updated_at").big_integer().not_null())
                .to_owned(),
        )
        .await
}

async fn create_subjects(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table("subjects")
                .if_not_exists()
                .col(column("id").string().not_null().primary_key())
                .col(column("name").string().not_null())
                .col(column("color").string())
                .col(column("created_at").big_integer().not_null())
                .col(column("updated_at").big_integer().not_null())
                .col(column("deleted_at").big_integer())
                .col(column("version").integer().not_null().default(0))
                .col(column("sync_status").string().not_null().default("synced"))
                .col(column("sync_hash").string())
                .to_owned(),
        )
        .await
}

async fn create_error_questions(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table("error_questions")
                .if_not_exists()
                .col(column("id").string().not_null().primary_key())
                .col(column("userid").string().not_null())
                .col(column("subjectid").string().not_null())
                .col(column("prompt").string().not_null())
                .col(column("type").string().not_null())
                .col(column("answer").text())
                .col(column("analysis").text())
                .col(column("error_note").text())
                .col(column("sourceid").string())
                .col(column("created_at").big_integer().not_null())
                .col(column("updated_at").big_integer().not_null())
                .col(column("deleted_at").big_integer())
                .col(column("version").integer().not_null().default(0))
                .col(column("sync_status").string().not_null().default("synced"))
                .col(column("sync_hash").string())
                .to_owned(),
        )
        .await
}

async fn create_srs_data(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table("srs_data")
                .if_not_exists()
                .col(column("id").string().not_null().primary_key())
                .col(column("question_id").string().not_null().unique_key())
                .col(column("stability").float().not_null().default(3.0))
                .col(column("difficulty").float().not_null().default(2.5))
                .col(column("next_review_at").big_integer())
                .col(column("lastreviewed_at").big_integer())
                .col(column("review_count").integer().not_null().default(0))
                .col(column("feedback_history").string().not_null().default("[]"))
                .col(column("created_at").big_integer().not_null())
                .col(column("updated_at").big_integer().not_null())
                .col(column("version").integer().not_null().default(0))
                .col(column("sync_status").string().not_null().default("synced"))
                .col(column("sync_hash").string())
                .col(column("deleted_at").big_integer())
                .to_owned(),
        )
        .await
}

async fn create_error_tags(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table("error_tags")
                .if_not_exists()
                .col(column("id").string().not_null().primary_key())
                .col(column("question_id").string().not_null())
                .col(column("name").string().not_null())
                .col(column("color").string().not_null())
                .col(column("created_at").big_integer().not_null())
                .col(column("updated_at").big_integer().not_null())
                .col(column("deleted_at").big_integer())
                .col(column("version").integer().not_null().default(0))
                .col(column("sync_status").string().not_null().default("synced"))
                .col(column("sync_hash").string())
                .to_owned(),
        )
        .await
}

async fn create_attachments(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table("attachments")
                .if_not_exists()
                .col(column("id").string().not_null().primary_key())
                .col(column("question_id").string().not_null())
                .col(column("type").string().not_null())
                .col(column("file_type").string().not_null())
                .col(column("base64_data").blob().not_null())
                .col(column("hash").string().not_null())
                .col(column("created_at").big_integer().not_null())
                .col(column("updated_at").big_integer().not_null())
                .col(column("deleted_at").big_integer())
                .col(column("version").integer().not_null().default(0))
                .col(column("sync_status").string().not_null().default("synced"))
                .col(column("sync_hash").string())
                .to_owned(),
        )
        .await
}

async fn create_sources(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table("sources")
                .if_not_exists()
                .col(column("id").string().not_null().primary_key())
                .col(column("question_id").string())
                .col(column("subject_id").string())
                .col(column("book").string())
                .col(column("chapter").string())
                .col(column("knowledge").string())
                .col(column("created_at").big_integer().not_null())
                .col(column("updated_at").big_integer().not_null())
                .col(column("deleted_at").big_integer())
                .col(column("version").integer().not_null().default(0))
                .col(column("sync_status").string().not_null().default("synced"))
                .col(column("sync_hash").string())
                .to_owned(),
        )
        .await
}
