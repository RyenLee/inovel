use std::fs;
use std::path::PathBuf;

/// 读取文本文件内容
///
/// 读取指定路径的文本文件并返回内容。文件大小限制为 10MB。
///
/// # 参数
/// - `file_path`: 文件的绝对路径
///
/// # 返回值
/// 成功返回文件内容，失败返回错误信息（包括文件不存在、路径不是文件、文件过大等）
#[tauri::command]
pub async fn read_text_file(file_path: String) -> Result<String, String> {
    let path = PathBuf::from(&file_path);

    // 验证路径
    if !path.exists() {
        return Err("文件不存在".to_string());
    }

    if !path.is_file() {
        return Err("路径不是文件".to_string());
    }

    // 获取文件大小
    let metadata = fs::metadata(&path).map_err(|e| format!("获取文件信息失败: {}", e))?;

    let file_size = metadata.len();

    // 限制文件大小（10MB）
    const MAX_SIZE: u64 = 10 * 1024 * 1024;
    if file_size > MAX_SIZE {
        return Err(format!("文件过大，最大支持 10MB"));
    }

    // 读取文件内容
    let content = fs::read_to_string(&path).map_err(|e| format!("读取文件失败: {}", e))?;

    Ok(content)
}

/// 检查文件是否存在
///
/// # 参数
/// - `file_path`: 文件的绝对路径
///
/// # 返回值
/// 存在返回 true，不存在返回 false
#[tauri::command]
pub async fn check_file_exists(file_path: String) -> Result<bool, String> {
    let path = PathBuf::from(&file_path);
    Ok(path.exists())
}

/// 获取文件大小
///
/// # 参数
/// - `file_path`: 文件的绝对路径
///
/// # 返回值
/// 成功返回文件大小（字节），失败返回错误信息
#[tauri::command]
pub async fn get_file_size(file_path: String) -> Result<u64, String> {
    let path = PathBuf::from(&file_path);
    let metadata = fs::metadata(&path).map_err(|e| format!("获取文件信息失败: {}", e))?;
    Ok(metadata.len())
}
