// 数据库连接管理

use sea_orm::{Database, DbConn, DbErr};
use std::path::PathBuf;

/// 获取数据库文件路径
pub fn get_database_path() -> PathBuf {
    // 获取应用数据目录
    let mut path = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("SmartErrorNotebook");
    path.push("data");

    // 确保目录存在
    std::fs::create_dir_all(&path).expect("Failed to create data directory");

    path.push("database.db");
    path
}

/// 建立数据库连接
pub async fn establish_connection() -> Result<DbConn, DbErr> {
    let db_path = get_database_path();
    println!("Connecting to database: {}", db_path.display());
    let db_url = format!("sqlite://{}?mode=rwc", db_path.display());

    Database::connect(&db_url).await
}

/// 初始化数据库（运行迁移）
pub async fn init_database(db: &DbConn) -> Result<(), DbErr> {
    use sea_orm_migration::MigratorTrait;

    // 运行所有迁移
    crate::database::migrations::Migrator::up(db, None).await?;

    Ok(())
}