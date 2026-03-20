// 数据库连接管理

use sea_orm::{Database, DbConn, DbErr};
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

/// 获取数据库文件路径
pub async fn get_database_path(app: Option<&AppHandle>) -> PathBuf {
    let base_dir = if let Some(app) = app {
        // 使用 Tauri 的 path API 获取跨平台的应用数据目录
        app.path()
            .resolve("", tauri::path::BaseDirectory::AppData)
            .expect("Failed to get app data directory")
    } else {
        // 如果没有 AppHandle，使用系统默认方式获取数据目录
        let base_dir = if cfg!(target_os = "windows") {
            dirs::data_local_dir()
        } else if cfg!(target_os = "macos") {
            dirs::data_dir()
        } else {
            // Linux 和其他系统
            dirs::data_dir()
        };

        base_dir.unwrap_or_else(|| PathBuf::from("."))
    };

    let mut path = base_dir;
    path.push("SmartErrorNotebook");
    path.push("data");

    // 确保目录存在
    std::fs::create_dir_all(&path).expect("Failed to create data directory");

    path.push("database.db");
    path
}

/// 建立数据库连接
pub async fn establish_connection(app: &AppHandle) -> Result<DbConn, DbErr> {
    let db_path = get_database_path(Some(app)).await;
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
