// 文件导出命令（PDF / 其他二进制文件）

use base64::{engine::general_purpose::STANDARD, Engine};

/// 写入 PDF 文件到指定路径
/// 接收 base64 编码的文件数据，解码后写入磁盘
#[tauri::command]
pub async fn write_pdf_file(path: String, data_base64: String) -> Result<(), String> {
    let bytes = STANDARD
        .decode(&data_base64)
        .map_err(|e| format!("Base64 解码失败: {}", e))?;

    if let Some(parent) = std::path::Path::new(&path).parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {}", e))?;
    }

    std::fs::write(&path, &bytes).map_err(|e| format!("写入文件失败: {}", e))?;
    Ok(())
}
