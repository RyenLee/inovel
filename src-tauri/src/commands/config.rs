use super::super::config_manager::api::ConfigManager;
use super::super::config_manager::model::ConfigItem;
use super::super::settings::{AppConfig, get_config_file_path, save_to_file};
use crate::settings::SharedConfig;
use std::collections::HashMap;
use tauri::{Manager, State};
use tracing::info;

lazy_static::lazy_static! {
    static ref CONFIG_MANAGER: std::sync::Mutex<ConfigManager> = std::sync::Mutex::new(ConfigManager::new());
}

/// 获取默认配置文件路径
///
/// 尝试从应用资源目录获取，如果不存在则回退到 src-tauri/resources 目录。
///
/// # 参数
/// - `app`: Tauri 应用句柄
///
/// # 返回值
/// 默认配置文件路径
fn get_default_config_path(app: &tauri::AppHandle) -> std::path::PathBuf {
    // 尝试从应用资源目录获取
    if let Ok(resource_dir) = app.path().resource_dir() {
        let config_path = resource_dir.join("resources/default_config.toml");
        if config_path.exists() {
            return config_path;
        }
    }

    // 回退到 src-tauri/resources 目录（开发环境）
    std::path::PathBuf::from("src-tauri/resources/default_config.toml")
}

#[tauri::command(rename_all = "snake_case")]
pub async fn read_toml_config(config: State<'_, SharedConfig>) -> Result<AppConfig, String> {
    let cfg = config.read().map_err(|e| format!("读取配置失败: {}", e))?;
    Ok(cfg.clone())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn write_toml_config(
    new_config: AppConfig,
    config: State<'_, SharedConfig>,
    app_state: State<'_, crate::state::AppState>,
) -> Result<AppConfig, String> {
    let validation = crate::settings::validate_config(&new_config);
    if !validation.valid {
        return Err(format!("配置验证失败: {}", validation.errors.join("; ")));
    }

    let config_path = get_config_file_path();

    save_to_file(&new_config, &config_path).map_err(|e| format!("保存配置文件失败: {}", e))?;

    {
        let mut cfg = config.write().map_err(|e| format!("写入配置失败: {}", e))?;
        *cfg = new_config.clone();
    }

    app_state.optimization.refresh(&config.inner().clone());

    info!("TOML配置文件已更新");
    let cfg = config.read().map_err(|e| format!("读取配置失败: {}", e))?;
    Ok(cfg.clone())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn reset_to_default_config(
    app: tauri::AppHandle,
    config: State<'_, SharedConfig>,
    app_state: State<'_, crate::state::AppState>,
) -> Result<AppConfig, String> {
    // 获取默认配置文件路径
    let default_config_path = get_default_config_path(&app);

    info!("从默认配置文件重置: {:?}", default_config_path);

    let default_config = crate::settings::load_from_file(&default_config_path)
        .map_err(|e| format!("加载默认配置文件失败: {}", e))?;

    let config_path = get_config_file_path();
    save_to_file(&default_config, &config_path).map_err(|e| format!("保存配置文件失败: {}", e))?;

    {
        let mut cfg = config.write().map_err(|e| format!("写入配置失败: {}", e))?;
        *cfg = default_config.clone();
    }

    app_state.optimization.refresh(&config.inner().clone());

    info!("配置已重置为默认值");
    Ok(default_config)
}

#[tauri::command(rename_all = "snake_case")]
pub fn get_config() -> Result<super::super::config_manager::model::AppConfig, String> {
    let manager = CONFIG_MANAGER
        .lock()
        .map_err(|_| "Failed to lock config manager")?;
    Ok(manager.get_config())
}

#[tauri::command(rename_all = "snake_case")]
pub fn get_config_value(
    key: &str,
) -> Result<Option<super::super::config_manager::model::ConfigValue>, String> {
    let manager = CONFIG_MANAGER
        .lock()
        .map_err(|_| "Failed to lock config manager")?;
    Ok(manager.get_value(key))
}

#[tauri::command(rename_all = "snake_case")]
pub fn get_config_by_category(category: &str) -> Result<Vec<ConfigItem>, String> {
    let manager = CONFIG_MANAGER
        .lock()
        .map_err(|_| "Failed to lock config manager")?;
    Ok(manager.get_values_by_category(category))
}

#[tauri::command(rename_all = "snake_case")]
pub fn set_config_value(key: &str, value: &str, encrypted: bool) -> Result<bool, String> {
    let mut manager = CONFIG_MANAGER
        .lock()
        .map_err(|_| "Failed to lock config manager")?;
    Ok(manager.set_value(key, value, encrypted))
}

#[tauri::command(rename_all = "snake_case")]
pub fn set_config_values(
    values: HashMap<String, String>,
) -> Result<super::super::config_manager::api::ConfigUpdateResult, String> {
    let mut manager = CONFIG_MANAGER
        .lock()
        .map_err(|_| "Failed to lock config manager")?;
    Ok(manager.set_values(values))
}

#[tauri::command(rename_all = "snake_case")]
pub fn update_app_version(new_version: &str) -> Result<bool, String> {
    let mut manager = CONFIG_MANAGER
        .lock()
        .map_err(|_| "Failed to lock config manager")?;
    Ok(manager.update_version(new_version))
}

#[tauri::command(rename_all = "snake_case")]
pub fn reload_config() -> Result<bool, String> {
    let mut manager = CONFIG_MANAGER
        .lock()
        .map_err(|_| "Failed to lock config manager")?;
    Ok(manager.reload())
}

#[tauri::command(rename_all = "snake_case")]
pub fn export_config(
    path: &str,
) -> Result<super::super::config_manager::api::ExportResult, String> {
    let manager = CONFIG_MANAGER
        .lock()
        .map_err(|_| "Failed to lock config manager")?;
    Ok(manager.export(path))
}

#[tauri::command(rename_all = "snake_case")]
pub fn import_config(
    path: &str,
) -> Result<super::super::config_manager::api::ConfigQueryResult, String> {
    let mut manager = CONFIG_MANAGER
        .lock()
        .map_err(|_| "Failed to lock config manager")?;
    Ok(manager.import(path))
}

#[tauri::command(rename_all = "snake_case")]
pub fn reset_config() -> Result<super::super::config_manager::api::ConfigQueryResult, String> {
    let mut manager = CONFIG_MANAGER
        .lock()
        .map_err(|_| "Failed to lock config manager")?;
    Ok(manager.reset())
}

#[tauri::command(rename_all = "snake_case")]
pub fn get_config_history(
    page: usize,
    page_size: usize,
) -> Result<super::super::config_manager::api::HistoryQueryResult, String> {
    let manager = CONFIG_MANAGER
        .lock()
        .map_err(|_| "Failed to lock config manager")?;
    Ok(manager.get_history(page, page_size))
}

#[tauri::command(rename_all = "snake_case")]
pub fn rollback_config(
    history_id: &str,
) -> Result<super::super::config_manager::api::ConfigQueryResult, String> {
    let mut manager = CONFIG_MANAGER
        .lock()
        .map_err(|_| "Failed to lock config manager")?;
    Ok(manager.rollback_to(history_id))
}
