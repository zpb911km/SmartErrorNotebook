use std::sync::Arc;

mod database;
mod commands;

use database::{establish_connection, init_database};

// 数据库连接状态
pub struct AppState {
    pub db: Arc<sea_orm::DbConn>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 初始化数据库
    let db = tauri::async_runtime::block_on(async {
        let db = establish_connection().await.expect("Failed to connect to database");
        init_database(&db).await.expect("Failed to initialize database");
        db
    });

    let state = AppState {
        db: Arc::new(db),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            commands::greet,
            commands::get_subjects,
            commands::create_subject,
            commands::update_subject,
            commands::delete_subject,
            commands::get_questions,
            commands::get_question,
            commands::create_question,
            commands::update_question,
            commands::delete_question,
            commands::get_question_stats,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}