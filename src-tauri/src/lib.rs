mod application;
mod command;
mod data;
mod model;
mod repository;
mod util;

use command::CommandRegistry;
use data::database::connection::{establish_connection, init_database};
use data::SeaOrmRepositoryTransactionExecutor;
use repository::RepositoryTransactionExecutor;
use std::sync::Mutex;
use tauri::Manager;

// 数据库连接状态
pub(crate) struct AppState<T: RepositoryTransactionExecutor = SeaOrmRepositoryTransactionExecutor> {
    pub repository_transaction_executor: T,
}

/// 存储通过文件关联打开的 URL
/// 覆盖两种场景：
///   - 冷启动：应用从文件打开启动，RunEvent::Opened 在 setup 之前或之后触发
///   - 热启动：应用已在运行，通过文件关联被唤起
pub struct OpenedUrls(Mutex<Vec<String>>);

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

            let state = AppState {
                repository_transaction_executor: SeaOrmRepositoryTransactionExecutor::new(db),
            };
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
        .register_command()
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

#[cfg(test)]
mod tests;
