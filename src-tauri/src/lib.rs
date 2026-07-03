use std::sync::{Arc, Mutex};

mod commands;
mod database;
mod srs;

use database::{establish_connection, init_database};
use tauri::Manager;

/// 存储通过文件关联打开的 URL
/// 覆盖两种场景：
///   - 冷启动：应用从文件打开启动，RunEvent::Opened 在 setup 之前或之后触发
///   - 热启动：应用已在运行，通过文件关联被唤起
struct OpenedUrls(Mutex<Vec<String>>);

/// 返回所有已存储的打开文件 URL（前端在冷启动时调用）
#[tauri::command]
fn opened_urls(state: tauri::State<'_, OpenedUrls>) -> Vec<String> {
    state.0.lock().unwrap().clone()
}

/// 读取通过文件关联传递的文件内容
/// 桌面端：直接读取文件路径
/// Android：通过 content:// URI 读取（需 JNI 支持）
#[tauri::command]
fn read_opened_file(url: String) -> Result<String, String> {
    // 尝试 1：解析为 URL，如果是 file:// 协议则提取路径
    if let Ok(parsed) = tauri::Url::parse(&url) {
        if parsed.scheme() == "file" {
            if let Ok(path) = parsed.to_file_path() {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    return Ok(content);
                }
            }
        }
    }

    // 尝试 2：直接作为文件路径读取（Linux/macOS 上的常规路径）
    if let Ok(content) = std::fs::read_to_string(&url) {
        return Ok(content);
    }

    // 尝试 3：移除 file:// 前缀后读取
    if let Some(path) = url.strip_prefix("file://") {
        if let Ok(content) = std::fs::read_to_string(path) {
            return Ok(content);
        }
    }

    // 尝试 4：Android content:// URI——需通过 JNI 使用 ContentResolver
    #[cfg(target_os = "android")]
    {
        return read_android_content_uri(&url);
    }

    Err(format!(
        "无法读取文件。文件路径: {}\n\
         提示：在移动端，请尝试在应用内使用「选择文件」按钮导入。",
        url
    ))
}

/// Android 上通过 ContentResolver 读取 content:// URI
#[cfg(target_os = "android")]
fn read_android_content_uri(_url: &str) -> Result<String, String> {
    // TODO: 通过 JNI 调用 Android ContentResolver.openInputStream()
    // 需要 tauri::android::Uri 和 JNI 桥接
    // 参考：https://docs.rs/tauri/latest/tauri/android/struct.Uri.html
    Err("Android content:// URI 读取暂未实现。请使用应用内「选择文件」按钮。".to_string())
}

// 数据库连接状态
pub struct AppState {
    pub db: Arc<sea_orm::DbConn>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_share::init())
        .manage(OpenedUrls(Mutex::new(Vec::new())))
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
            // Sync
            commands::get_all_pending_records,
            commands::get_record_for_upload,
            commands::set_record_sync_status_version,
            commands::get_all_records,
            commands::purge_synced_deletions,
            commands::check_orphan_records,
            // Subject
            commands::get_subjects,
            commands::create_subject,
            commands::update_subject,
            commands::delete_subject,
            commands::upsert_subject,
            // Error Question
            commands::get_questions,
            commands::get_question,
            commands::create_question,
            commands::update_question,
            commands::delete_question,
            commands::upsert_error_question,
            commands::get_question_stats,
            // Error Tag
            commands::create_error_tags_for_question,
            commands::get_error_tags,
            commands::get_full_error_tags,
            commands::get_error_tags_for_question,
            commands::delete_error_tag,
            commands::update_error_tag_by_id,
            commands::update_error_tag_by_name,
            commands::upsert_error_tag,
            // SRS Data
            commands::create_srs_data,
            commands::get_due_questions,
            commands::submit_review_result,
            commands::get_question_srs_status,
            commands::reset_srs_progress,
            commands::upsert_srs_data,
            // SRS Tools
            commands::get_due_count,
            commands::get_srs_statistics,
            commands::get_all_cards,
            // Attachment
            commands::create_attachment,
            commands::create_attachments_for_question,
            commands::get_attachments_by_question,
            commands::delete_attachment,
            commands::upsert_attachment,
            // Source
            commands::get_sources,
            commands::get_books,
            commands::get_chapters,
            commands::get_knowledges,
            commands::create_source,
            commands::update_source,
            commands::delete_source,
            commands::get_source,
            commands::get_or_create_source_id,
            commands::upsert_source,
            // File Association
            opened_urls,
            read_opened_file,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|app_handle, event| {
        // macOS/iOS/Android: 处理通过文件关联打开应用的事件
        #[cfg(any(target_os = "macos", target_os = "ios", target_os = "android"))]
        if let tauri::RunEvent::Opened { ref urls } = event {
            use tauri::Emitter;

            let urls_str: Vec<String> = urls.iter().map(|u| u.as_str().to_string()).collect();

            // 存入 State，供冷启动时前端通过 opened_urls 命令读取
            app_handle
                .state::<OpenedUrls>()
                .0
                .lock()
                .unwrap()
                .extend(urls_str.clone());

            // 向前端发送事件，供热启动时前端通过 listen('opened') 接收
            let _ = app_handle.emit("opened", urls_str);
        }

        // 在其他平台上静默忽略未使用的绑定（保持 API 一致）
        let _ = (app_handle, event);
    });
}
