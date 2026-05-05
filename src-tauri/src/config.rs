//! 应用配置管理模块
//!
//! 支持从 JSON 配置文件读取路径设置，支持环境变量替换。
//!
//! 配置文件路径: `{APP_DATA}/config.json`
//!
//! 支持的环境变量格式: `${VAR_NAME}` 或 `$VAR_NAME`

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::RwLock;
use tauri::AppHandle;
use tauri::Manager;

lazy_static! {
    /// 全局配置单例（线程安全）
    static ref APP_CONFIG: RwLock<Option<AppConfig>> = RwLock::new(None);
}

/// 配置版本（用于未来升级迁移）
const CONFIG_VERSION: &str = "1.0";

/// 默认配置值
pub fn default_config() -> AppConfig {
    AppConfig {
        version: CONFIG_VERSION.to_string(),
        database: DatabaseConfig {
            path: "${APP_DATA}/inovel.db".to_string(),
            backup_dir: "${APP_DATA}/backups".to_string(),
            auto_backup: true,
            vacuum_on_close: false,
        },
        paths: PathsConfig {
            projects_root: "${APP_DATA}/projects".to_string(),
            exports_dir: "${APP_DATA}/exports".to_string(),
        },
        logging: LoggingConfig {
            level: "info".to_string(),
            directory: "${APP_DATA}/logs".to_string(),
            max_file_size_mb: 50,
            retention_days: 30,
            backup_log_enabled: true,
        },
        backup: BackupConfig {
            enabled: true,
            backup_dir: "${APP_DATA}/backups".to_string(),
            max_backups_per_project: 10,
            incremental_threshold_hours: 24,
        },
    }
}

/// 应用根配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub version: String,
    pub database: DatabaseConfig,
    pub paths: PathsConfig,
    pub logging: LoggingConfig,
    pub backup: BackupConfig,
}

/// 数据库相关配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// 数据库文件路径（支持环境变量）
    pub path: String,
    /// 数据库备份目录
    pub backup_dir: String,
    /// 是否在启动时自动备份数据库
    pub auto_backup: bool,
    /// 是否在关闭连接时执行 VACUUM
    pub vacuum_on_close: bool,
}

/// 路径相关配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathsConfig {
    /// 项目根目录
    pub projects_root: String,
    /// 导出文件目录
    pub exports_dir: String,
}

/// 日志相关配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// 日志级别: trace, debug, info, warn, error
    pub level: String,
    /// 日志目录
    pub directory: String,
    /// 单个日志文件最大大小（MB）
    pub max_file_size_mb: u32,
    /// 日志保留天数
    pub retention_days: u32,
    /// 是否启用备份日志
    pub backup_log_enabled: bool,
}

/// 备份相关配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupConfig {
    /// 是否启用备份功能
    pub enabled: bool,
    /// 备份存储目录
    pub backup_dir: String,
    /// 每个项目最大备份数量
    pub max_backups_per_project: u32,
    /// 增量备份阈值（小时）
    pub incremental_threshold_hours: u32,
}

/// 解析配置文件路径
fn get_config_path(app_handle: &AppHandle) -> PathBuf {
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .expect("Failed to get app data dir");
    app_dir.join("config.json")
}

/// 替换配置字符串中的环境变量
///
/// 支持格式:
/// - `${VAR_NAME}` - 标准格式
/// - `$VAR_NAME` - 简写格式（仅大写字母、数字和下划线）
fn expand_env_vars(path: &str, app_data_dir: &Path) -> String {
    let mut result = path.to_string();

    // 替换 ${APP_DATA} 为实际的 app data 目录
    result = result.replace("${APP_DATA}", &app_data_dir.to_string_lossy());
    result = result.replace("$APP_DATA", &app_data_dir.to_string_lossy());

    // 替换其他常见环境变量
    if let Ok(home) = std::env::var("HOME") {
        result = result.replace("${HOME}", &home);
        result = result.replace("$HOME", &home);
    }

    if let Ok(temp) = std::env::var("TEMP") {
        result = result.replace("${TEMP}", &temp);
        result = result.replace("$TEMP", &temp);
    }

    if let Ok(user) = std::env::var("USERNAME") {
        result = result.replace("${USERNAME}", &user);
        result = result.replace("$USERNAME", &user);
    }

    // 处理其他 ${VAR} 格式的环境变量
    let re_pattern = regex_lite::Regex::new(r"\$\{([A-Za-z_][A-Za-z0-9_]*)\}").unwrap();
    result = re_pattern
        .replace_all(&result, |caps: &regex_lite::Captures| {
            let var_name = &caps[1];
            std::env::var(var_name).unwrap_or_else(|_| caps[0].to_string())
        })
        .to_string();

    // 处理 $VAR_NAME 格式
    let re_simple = regex_lite::Regex::new(r"\$([A-Za-z_][A-Za-z0-9_]*)").unwrap();
    result = re_simple
        .replace_all(&result, |caps: &regex_lite::Captures| {
            let var_name = &caps[1];
            // 跳过已经是 ${VAR} 格式的
            if caps[0].starts_with("${") {
                return caps[0].to_string();
            }
            std::env::var(var_name).unwrap_or_else(|_| caps[0].to_string())
        })
        .to_string();

    result
}

/// 初始化配置系统
///
/// 如果配置文件不存在，则创建默认配置。
pub fn init_config(app_handle: &AppHandle) -> Result<(), String> {
    let config_path = get_config_path(app_handle);
    let app_dir = config_path.parent().unwrap();

    // 确保目录存在
    fs::create_dir_all(app_dir).map_err(|e| format!("创建配置目录失败: {}", e))?;

    let config = if config_path.exists() {
        // 读取现有配置
        let content = fs::read_to_string(&config_path)
            .map_err(|e| format!("读取配置文件失败: {}", e))?;

        // 解析配置（环境变量在 get_config_with_expanded_paths 中展开）
        let mut config: AppConfig = serde_json::from_str(&content)
            .map_err(|e| format!("解析配置文件失败: {}", e))?;

        // 更新版本信息
        config.version = CONFIG_VERSION.to_string();

        config
    } else {
        // 创建默认配置
        let default = default_config();
        let content = serde_json::to_string_pretty(&default)
            .map_err(|e| format!("序列化配置失败: {}", e))?;

        fs::write(&config_path, &content)
            .map_err(|e| format!("写入配置文件失败: {}", e))?;

        default
    };

    // 保存到全局状态
    let mut global_config = APP_CONFIG.write().map_err(|_| "配置锁中毒")?;
    *global_config = Some(config);

    Ok(())
}

/// 获取配置（需先调用 init_config）
pub fn get_config() -> Result<AppConfig, String> {
    let config = APP_CONFIG.read().map_err(|_| "配置锁中毒")?;
    config.clone().ok_or_else(|| "配置未初始化，请先调用 init_config".to_string())
}

/// 获取配置并解析路径中的环境变量
pub fn get_config_with_expanded_paths(app_handle: &AppHandle) -> Result<ExpandedConfig, String> {
    let config = get_config()?;
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("获取应用数据目录失败: {}", e))?;

    Ok(ExpandedConfig {
        version: config.version,
        database: ExpandedDatabaseConfig {
            path: expand_path(&config.database.path, &app_dir),
            backup_dir: expand_path(&config.database.backup_dir, &app_dir),
            auto_backup: config.database.auto_backup,
            vacuum_on_close: config.database.vacuum_on_close,
        },
        paths: ExpandedPathsConfig {
            projects_root: expand_path(&config.paths.projects_root, &app_dir),
            exports_dir: expand_path(&config.paths.exports_dir, &app_dir),
        },
        logging: ExpandedLoggingConfig {
            level: config.logging.level.clone(),
            directory: expand_path(&config.logging.directory, &app_dir),
            max_file_size_mb: config.logging.max_file_size_mb,
            retention_days: config.logging.retention_days,
            backup_log_enabled: config.logging.backup_log_enabled,
        },
        backup: ExpandedBackupConfig {
            enabled: config.backup.enabled,
            backup_dir: expand_path(&config.backup.backup_dir, &app_dir),
            max_backups_per_project: config.backup.max_backups_per_project,
            incremental_threshold_hours: config.backup.incremental_threshold_hours,
        },
    })
}

/// 展开路径中的环境变量
fn expand_path(path: &str, app_data_dir: &Path) -> PathBuf {
    let expanded = expand_env_vars(path, app_data_dir);
    PathBuf::from(expanded)
}

/// 展开后的数据库配置（路径已解析）
#[derive(Debug, Clone)]
pub struct ExpandedDatabaseConfig {
    pub path: PathBuf,
    pub backup_dir: PathBuf,
    pub auto_backup: bool,
    pub vacuum_on_close: bool,
}

/// 展开后的路径配置
#[derive(Debug, Clone)]
pub struct ExpandedPathsConfig {
    pub projects_root: PathBuf,
    pub exports_dir: PathBuf,
}

/// 展开后的日志配置
#[derive(Debug, Clone)]
pub struct ExpandedLoggingConfig {
    pub level: String,
    pub directory: PathBuf,
    pub max_file_size_mb: u32,
    pub retention_days: u32,
    pub backup_log_enabled: bool,
}

/// 展开后的备份配置
#[derive(Debug, Clone)]
pub struct ExpandedBackupConfig {
    pub enabled: bool,
    pub backup_dir: PathBuf,
    pub max_backups_per_project: u32,
    pub incremental_threshold_hours: u32,
}

/// 展开后的完整配置
#[derive(Debug, Clone)]
pub struct ExpandedConfig {
    pub version: String,
    pub database: ExpandedDatabaseConfig,
    pub paths: ExpandedPathsConfig,
    pub logging: ExpandedLoggingConfig,
    pub backup: ExpandedBackupConfig,
}

/// 获取数据库路径（使用配置）
pub fn get_db_path(app_handle: &AppHandle) -> Result<PathBuf, String> {
    let config = get_config_with_expanded_paths(app_handle)?;

    // 确保目录存在
    if let Some(parent) = config.database.path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("创建数据库目录失败: {}", e))?;
    }

    Ok(config.database.path)
}

/// 获取项目根目录（使用配置）
pub fn get_projects_root(app_handle: &AppHandle) -> Result<PathBuf, String> {
    let config = get_config_with_expanded_paths(app_handle)?;
    let path = config.paths.projects_root;

    fs::create_dir_all(&path)
        .map_err(|e| format!("创建项目目录失败: {}", e))?;

    Ok(path)
}

/// 获取导出目录（使用配置）
pub fn get_exports_dir(app_handle: &AppHandle) -> Result<PathBuf, String> {
    let config = get_config_with_expanded_paths(app_handle)?;
    let path = config.paths.exports_dir;

    fs::create_dir_all(&path)
        .map_err(|e| format!("创建导出目录失败: {}", e))?;

    Ok(path)
}

/// 获取日志目录（使用配置）
pub fn get_log_dir(app_handle: &AppHandle) -> Result<PathBuf, String> {
    let config = get_config_with_expanded_paths(app_handle)?;
    let path = config.logging.directory;

    fs::create_dir_all(&path)
        .map_err(|e| format!("创建日志目录失败: {}", e))?;

    Ok(path)
}

/// 获取备份目录（使用配置）
pub fn get_backup_dir(app_handle: &AppHandle) -> Result<PathBuf, String> {
    let config = get_config_with_expanded_paths(app_handle)?;
    let path = config.backup.backup_dir;

    fs::create_dir_all(&path)
        .map_err(|e| format!("创建备份目录失败: {}", e))?;

    Ok(path)
}

/// 获取数据库备份目录
pub fn get_db_backup_dir(app_handle: &AppHandle) -> Result<PathBuf, String> {
    let config = get_config_with_expanded_paths(app_handle)?;
    let path = config.database.backup_dir;

    fs::create_dir_all(&path)
        .map_err(|e| format!("创建数据库备份目录失败: {}", e))?;

    Ok(path)
}

// ============== Tauri Commands ==============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigInfo {
    pub version: String,
    pub database: DatabaseConfig,
    pub paths: PathsConfig,
    pub logging: LoggingConfig,
    pub backup: BackupConfig,
    /// 展开后的实际路径（用于前端显示）
    pub expanded_paths: ExpandedPathsDisplay,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpandedPathsDisplay {
    pub database_path: String,
    pub projects_root: String,
    pub exports_dir: String,
    pub log_directory: String,
    pub backup_directory: String,
}

/// 读取当前配置（原始格式）
#[tauri::command]
pub async fn get_app_config(app_handle: AppHandle) -> Result<ConfigInfo, String> {
    let config = get_config()?;
    let expanded = get_config_with_expanded_paths(&app_handle)?;

    Ok(ConfigInfo {
        version: config.version,
        database: config.database,
        paths: config.paths,
        logging: config.logging,
        backup: config.backup,
        expanded_paths: ExpandedPathsDisplay {
            database_path: expanded.database.path.to_string_lossy().to_string(),
            projects_root: expanded.paths.projects_root.to_string_lossy().to_string(),
            exports_dir: expanded.paths.exports_dir.to_string_lossy().to_string(),
            log_directory: expanded.logging.directory.to_string_lossy().to_string(),
            backup_directory: expanded.backup.backup_dir.to_string_lossy().to_string(),
        },
    })
}

/// 更新配置（部分更新）
#[tauri::command]
pub async fn update_app_config(
    app_handle: AppHandle,
    updates: serde_json::Value,
) -> Result<ConfigInfo, String> {
    let config_path = get_config_path(&app_handle);

    // 读取当前配置
    let mut config = get_config()?;

    // 应用更新（浅合并）
    if let Some(db) = updates.get("database") {
        if let Some(path) = db.get("path").and_then(|v| v.as_str()) {
            config.database.path = path.to_string();
        }
        if let Some(backup_dir) = db.get("backup_dir").and_then(|v| v.as_str()) {
            config.database.backup_dir = backup_dir.to_string();
        }
        if let Some(auto_backup) = db.get("auto_backup").and_then(|v| v.as_bool()) {
            config.database.auto_backup = auto_backup;
        }
        if let Some(vacuum) = db.get("vacuum_on_close").and_then(|v| v.as_bool()) {
            config.database.vacuum_on_close = vacuum;
        }
    }

    if let Some(paths) = updates.get("paths") {
        if let Some(projects) = paths.get("projects_root").and_then(|v| v.as_str()) {
            config.paths.projects_root = projects.to_string();
        }
        if let Some(exports) = paths.get("exports_dir").and_then(|v| v.as_str()) {
            config.paths.exports_dir = exports.to_string();
        }
    }

    if let Some(logging) = updates.get("logging") {
        if let Some(level) = logging.get("level").and_then(|v| v.as_str()) {
            config.logging.level = level.to_string();
        }
        if let Some(dir) = logging.get("directory").and_then(|v| v.as_str()) {
            config.logging.directory = dir.to_string();
        }
        if let Some(size) = logging.get("max_file_size_mb").and_then(|v| v.as_u64()) {
            config.logging.max_file_size_mb = size as u32;
        }
        if let Some(days) = logging.get("retention_days").and_then(|v| v.as_u64()) {
            config.logging.retention_days = days as u32;
        }
    }

    if let Some(backup) = updates.get("backup") {
        if let Some(enabled) = backup.get("enabled").and_then(|v| v.as_bool()) {
            config.backup.enabled = enabled;
        }
        if let Some(dir) = backup.get("backup_dir").and_then(|v| v.as_str()) {
            config.backup.backup_dir = dir.to_string();
        }
        if let Some(max) = backup.get("max_backups_per_project").and_then(|v| v.as_u64()) {
            config.backup.max_backups_per_project = max as u32;
        }
    }

    // 保存配置
    let content = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("序列化配置失败: {}", e))?;

    fs::write(&config_path, &content)
        .map_err(|e| format!("写入配置文件失败: {}", e))?;

    // 更新内存中的配置
    {
        let mut global_config = APP_CONFIG.write().map_err(|_| "配置锁中毒")?;
        *global_config = Some(config.clone());
    }

    // 返回更新后的配置
    get_app_config(app_handle).await
}

/// 重置配置为默认值
#[tauri::command]
pub async fn reset_app_config(app_handle: AppHandle) -> Result<ConfigInfo, String> {
    let config_path = get_config_path(&app_handle);
    let default = default_config();

    // 保存默认配置
    let content = serde_json::to_string_pretty(&default)
        .map_err(|e| format!("序列化配置失败: {}", e))?;

    fs::write(&config_path, &content)
        .map_err(|e| format!("写入配置文件失败: {}", e))?;

    // 更新内存中的配置
    {
        let mut global_config = APP_CONFIG.write().map_err(|_| "配置锁中毒")?;
        *global_config = Some(default.clone());
    }

    // 返回默认配置
    get_app_config(app_handle).await
}

/// 获取配置文件的原始路径（用于前端打开编辑器）
#[tauri::command]
pub async fn get_config_file_path(app_handle: AppHandle) -> Result<String, String> {
    let config_path = get_config_path(&app_handle);
    Ok(config_path.to_string_lossy().to_string())
}

/// 验证配置路径是否可访问
#[tauri::command]
pub async fn validate_config_paths(app_handle: AppHandle) -> Result<Vec<PathValidationResult>, String> {
    let expanded = get_config_with_expanded_paths(&app_handle)?;
    let mut results = Vec::new();

    // 验证数据库路径
    let db_result = validate_single_path(
        "database.path",
        &expanded.database.path,
        true, // 需要可写
    );
    results.push(db_result);

    // 验证数据库备份目录
    let db_backup_result = validate_single_path(
        "database.backup_dir",
        &expanded.database.backup_dir,
        true,
    );
    results.push(db_backup_result);

    // 验证项目根目录
    let projects_result = validate_single_path(
        "paths.projects_root",
        &expanded.paths.projects_root,
        true,
    );
    results.push(projects_result);

    // 验证导出目录
    let exports_result = validate_single_path(
        "paths.exports_dir",
        &expanded.paths.exports_dir,
        true,
    );
    results.push(exports_result);

    // 验证日志目录
    let log_result = validate_single_path(
        "logging.directory",
        &expanded.logging.directory,
        true,
    );
    results.push(log_result);

    // 验证备份目录
    let backup_result = validate_single_path(
        "backup.backup_dir",
        &expanded.backup.backup_dir,
        true,
    );
    results.push(backup_result);

    Ok(results)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathValidationResult {
    pub key: String,
    pub path: String,
    pub exists: bool,
    pub writable: bool,
    pub is_directory: bool,
    pub error: Option<String>,
}

fn validate_single_path(key: &str, path: &PathBuf, require_writable: bool) -> PathValidationResult {
    let exists = path.exists();
    let is_directory = path.is_dir();

    let mut writable = false;
    let mut error = None;

    if exists {
        // 检查是否可写（尝试创建测试文件）
        if require_writable {
            let test_file = path.join(".write_test");
            writable = fs::write(&test_file, "test").is_ok();
            if writable {
                let _ = fs::remove_file(&test_file);
            } else {
                error = Some("目录不可写".to_string());
            }
        }
    } else {
        // 尝试创建父目录
        if let Some(parent) = path.parent() {
            match fs::create_dir_all(parent) {
                Ok(_) => {
                    writable = true;
                    if path.to_string_lossy().ends_with('/') || is_directory {
                        match fs::create_dir_all(path) {
                            Ok(_) => {}
                            Err(e) => error = Some(format!("无法创建目录: {}", e)),
                        }
                    }
                }
                Err(e) => {
                    writable = false;
                    error = Some(format!("无法创建目录: {}", e));
                }
            }
        }
    }

    PathValidationResult {
        key: key.to_string(),
        path: path.to_string_lossy().to_string(),
        exists,
        writable,
        is_directory,
        error,
    }
}
