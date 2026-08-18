use crate::OpenedUrls;

/// 返回所有已存储的打开文件 URL（前端在冷启动时调用）
#[tauri::command]
pub fn opened_urls(state: tauri::State<'_, OpenedUrls>) -> Vec<String> {
    state.0.lock().unwrap().clone()
}

/// 读取通过文件关联传递的文件内容
/// 桌面端：直接读取文件路径
/// Android：通过 content:// URI 读取（需 JNI 支持）
#[tauri::command]
pub fn read_opened_file(url: String) -> Result<String, String> {
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
