use crate::config;
use crate::logging::operation::{self, OperationLogFilter, RecordOperationParams};
use tauri::AppHandle;

#[tauri::command]
pub async fn record_operation_log(
    app_handle: AppHandle,
    params: RecordOperationParams,
) -> Result<crate::logging::OperationLog, String> {
    operation::record_operation(&app_handle, params)
}

#[tauri::command]
pub async fn query_operation_logs(
    app_handle: AppHandle,
    filter: Option<OperationLogFilter>,
) -> Result<Vec<crate::logging::OperationLog>, String> {
    operation::query_operation_logs(
        &app_handle,
        filter.unwrap_or_default(),
    )
}

#[tauri::command]
pub async fn get_operation_stats(
    app_handle: AppHandle,
    days: Option<u32>,
) -> Result<Vec<crate::logging::operation::OperationStat>, String> {
    operation::get_operation_stats(&app_handle, days.unwrap_or(30))
}

#[tauri::command]
pub async fn get_error_logs(
    app_handle: AppHandle,
) -> Result<Vec<String>, String> {
    let log_dir = config::get_log_dir(&app_handle);
    let paths = crate::logging::error_log::get_error_log_paths(&log_dir);

    let mut contents = Vec::new();
    for path in &paths {
        match std::fs::read_to_string(path) {
            Ok(content) => {
                if !content.is_empty() {
                    contents.push(format!("=== {} ===\n{}", path.display(), content));
                }
            }
            Err(_) => continue,
        }
    }

    Ok(contents)
}

#[tauri::command]
pub async fn clear_error_logs(
    app_handle: AppHandle,
) -> Result<(), String> {
    let log_dir = config::get_log_dir(&app_handle);
    let paths = crate::logging::error_log::get_error_log_paths(&log_dir);

    for path in &paths {
        std::fs::remove_file(path)
            .map_err(|e| format!("删除错误日志失败 {}: {}", path.display(), e))?;
    }

    Ok(())
}
