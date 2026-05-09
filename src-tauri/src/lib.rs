use std::sync::Arc;

mod commands;
mod database;
mod srs;

use database::{establish_connection, init_database};
use tauri::Manager;

// 数据库连接状态
pub struct AppState {
    pub db: Arc<sea_orm::DbConn>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            // 初始化数据库
            let db = tauri::async_runtime::block_on(async {
                let app_handle_clone = app.handle().clone();
                let db = establish_connection(&app_handle_clone)
                    .await
                    .expect("Failed to connect to database");
                init_database(&db)
                    .await
                    .expect("Failed to initialize database");
                db
            });

            let state = AppState { db: Arc::new(db) };
            app.manage(state);

            #[cfg(debug_assertions)] // 仅在调试构建时包含此代码
            {
                let window = app.get_webview_window("main").unwrap();
                window.open_devtools();
                window.close_devtools();
            }
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            // Subject
            commands::get_subjects,
            commands::create_subject,
            commands::update_subject,
            commands::delete_subject,
            // Error Question
            commands::get_questions,
            commands::get_question,
            commands::create_question,
            commands::update_question,
            commands::delete_question,
            commands::get_question_stats,
            // Error Tag
            commands::create_error_tags_for_question,
            commands::get_error_tags,
            commands::delete_error_tag_by_name,
            commands::update_error_tag_by_name,
            commands::delete_error_tag_by_id,
            commands::update_error_tag_by_id,
            // SRS Data
            commands::create_srs_data,
            commands::get_due_questions,
            commands::submit_review_result,
            commands::get_question_srs_status,
            commands::reset_srs_progress,
            // SRS Tools
            commands::get_due_count,
            commands::get_srs_statistics,
            commands::get_all_cards,
            // Attachment
            commands::create_attachment,
            commands::create_attachments_for_question,
            commands::get_attachments_by_question,
            commands::delete_attachment,
            // Source
            commands::get_sources,
            commands::get_books,
            commands::get_chapters,
            commands::get_knowledges,
            commands::create_source,
            commands::update_source,
            commands::delete_source,
            commands::get_source,
            commands::get_or_create_source_id
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
