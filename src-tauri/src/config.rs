//! 应用配置管理模块
//!
//! 支持从 JSON 配置文件读取路径设置，支持环境变量替换。
//!
//! 配置文件路径: `{INSTALL_DIR}/config.json`
//!
//! 支持的环境变量格式: `${VAR_NAME}` 或 `$VAR_NAME`
//! 内置变量: `${APP_INSTALL_DIR}` - 应用安装目录

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
            path: "${APP_INSTALL_DIR}/inovel.db".to_string(),
            backup_dir: "${APP_INSTALL_DIR}/backups".to_string(),
            auto_backup: true,
            vacuum_on_close: false,
        },
        paths: PathsConfig {
            projects_root: "${APP_INSTALL_DIR}/projects".to_string(),
            exports_dir: "${APP_INSTALL_DIR}/exports".to_string(),
        },
        logging: LoggingConfig {
            level: "info".to_string(),
            directory: "${APP_INSTALL_DIR}/logs".to_string(),
            max_file_size_mb: 50,
            retention_days: 30,
            backup_log_enabled: true,
        },
        backup: BackupConfig {
            enabled: true,
            backup_dir: "${APP_INSTALL_DIR}/backups".to_string(),
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

/// 获取配置文件路径（安装目录）
fn get_config_path(app_handle: &AppHandle) -> PathBuf {
    get_install_dir(app_handle).join("config.json")
}

/// 获取应用程序安装目录
/// 优先使用资源目录（打包后），回退到 exe 所在目录
fn get_install_dir(app_handle: &AppHandle) -> PathBuf {
    // 尝试获取资源目录（打包后的应用目录）
    if let Ok(resource_path) = app_handle.path().resource_dir() {
        return resource_path;
    }
    
    // 回退：获取 exe 所在目录
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(parent) = exe_path.parent() {
            return parent.to_path_buf();
        }
    }
    
    // 最后回退到当前目录
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

/// 替换配置字符串中的环境变量
///
/// 支持格式:
/// - `${VAR_NAME}` - 标准格式
/// - `$VAR_NAME` - 简写格式（仅大写字母、数字和下划线）
/// - `${APP_INSTALL_DIR}` - 应用安装目录（自动替换）
fn expand_env_vars(path: &str, install_dir: &Path) -> String {
    let mut result = path.to_string();

    result = result.replace("${APP_INSTALL_DIR}", &install_dir.to_string_lossy());
    result = result.replace("$APP_INSTALL_DIR", &install_dir.to_string_lossy());

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
            if var_name == "APP_INSTALL_DIR" {
                return caps[0].to_string();
            }
            std::env::var(var_name).unwrap_or_else(|_| caps[0].to_string())
        })
        .to_string();

    // 处理 $VAR_NAME 格式
    let re_simple = regex_lite::Regex::new(r"\$([A-Za-z_][A-Za-z0-9_]*)").unwrap();
    result = re_simple
        .replace_all(&result, |caps: &regex_lite::Captures| {
            let var_name = &caps[1];
            if caps[0].starts_with("${") || var_name == "APP_INSTALL_DIR" {
                return caps[0].to_string();
            }
            std::env::var(var_name).unwrap_or_else(|_| caps[0].to_string())
        })
        .to_string();

    result
}

/// 初始化配置系统
///
/// 从安装目录读取 config.json。如果配置文件不存在，则创建默认配置，
/// 版本号从 tauri.conf.json 读取，`${APP_INSTALL_DIR}` 占位符替换为实际安装目录。
pub fn init_config(app_handle: &AppHandle) -> Result<(), String> {
    let install_dir = get_install_dir(app_handle);
    let config_path = install_dir.join("config.json");

    let config = if config_path.exists() {
        let content = fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read config: {}", e))?;

        let mut config: AppConfig = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse config: {}", e))?;

        config.version = CONFIG_VERSION.to_string();
        config
    } else {
        // 验证安装目录有效性
        let validation = validate_install_dir(&install_dir);
        if !validation.valid {
            return Err(format!(
                "Invalid install directory: {} (exists={}, readable={}, writable={}, absolute={})",
                validation.error_message,
                validation.exists,
                validation.is_readable,
                validation.is_writable,
                validation.is_absolute
            ));
        }

        if !install_dir.exists() {
            fs::create_dir_all(&install_dir)
                .map_err(|e| format!("Failed to create install directory: {}", e))?;
        }

        let mut config = default_config();
        config.version = get_app_version_from_tauri_config();

        // 将 ${APP_INSTALL_DIR} 占位符替换为实际安装目录
        let install_dir_str = install_dir.to_string_lossy();
        config.database.path = config.database.path.replace("${APP_INSTALL_DIR}", &install_dir_str);
        config.database.backup_dir = config.database.backup_dir.replace("${APP_INSTALL_DIR}", &install_dir_str);
        config.paths.projects_root = config.paths.projects_root.replace("${APP_INSTALL_DIR}", &install_dir_str);
        config.paths.exports_dir = config.paths.exports_dir.replace("${APP_INSTALL_DIR}", &install_dir_str);
        config.logging.directory = config.logging.directory.replace("${APP_INSTALL_DIR}", &install_dir_str);
        config.backup.backup_dir = config.backup.backup_dir.replace("${APP_INSTALL_DIR}", &install_dir_str);

        let content = serde_json::to_string_pretty(&config)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;

        fs::write(&config_path, &content)
            .map_err(|e| format!("Failed to write config: {}", e))?;

        println!("[Config] First launch, created config at: {}", config_path.display());
        config
    };

    let mut global_config = APP_CONFIG.write().map_err(|_| "Config lock poisoned")?;
    *global_config = Some(config);

    Ok(())
}

/// 验证安装目录有效性的结果
#[derive(Debug, Clone)]
pub struct InstallDirValidationResult {
    pub valid: bool,
    pub exists: bool,
    pub is_readable: bool,
    pub is_writable: bool,
    pub is_absolute: bool,
    pub error_message: String,
}

/// 验证安装目录的有效性
fn validate_install_dir(path: &Path) -> InstallDirValidationResult {
    let mut result = InstallDirValidationResult {
        valid: true,
        exists: path.exists(),
        is_readable: false,
        is_writable: false,
        is_absolute: path.is_absolute(),
        error_message: String::new(),
    };

    // 检查路径是否为绝对路径
    if !result.is_absolute {
        result.valid = false;
        result.error_message = "Path is not absolute".to_string();
        return result;
    }

    // 检查路径是否存在
    if !result.exists {
        result.valid = false;
        result.error_message = "Path does not exist".to_string();
        return result;
    }

    // 检查是否为目录
    if !path.is_dir() {
        result.valid = false;
        result.error_message = "Path is not a directory".to_string();
        return result;
    }

    // 检查读权限
    match std::fs::metadata(path) {
        Ok(meta) => {
            result.is_readable = meta.permissions().readonly() == false;
            // 尝试写入测试文件检查写权限
            let test_file = path.join(".write_test_tmp");
            match fs::write(&test_file, "test") {
                Ok(_) => {
                    let _ = fs::remove_file(&test_file);
                    result.is_writable = true;
                }
                Err(_) => {
                    result.valid = false;
                    result.error_message = "Directory is not writable".to_string();
                }
            }
        }
        Err(e) => {
            result.valid = false;
            result.error_message = format!("Failed to read metadata: {}", e);
        }
    }

    result
}

/// 从 tauri.conf.json 读取应用版本
fn get_app_version_from_tauri_config() -> String {
    // 优先从环境变量获取（CI/CD 注入）
    if let Ok(version) = std::env::var("TAURI_APP_VERSION") {
        return version;
    }

    // 回退到默认值
    "1.0.0".to_string()
}

/// 数据库文件初始化结果
#[derive(Debug, Clone)]
pub struct DbInitResult {
    pub db_path: String,
    pub created: bool,
    pub connected: bool,
    pub message: String,
}

/// 初始化数据库文件（首次启动时创建）
///
/// 根据 config.json 中 database.path 指定的路径创建 SQLite 数据库文件。
/// 在创建前验证路径有效性（目录存在、可写），创建后进行连接测试。
pub fn init_db_file(_app_handle: &AppHandle) -> Result<DbInitResult, String> {
    let config = get_config()?;
    let db_path_str = &config.database.path;
    let db_path = Path::new(db_path_str);

    // 验证父目录是否存在
    let parent_dir = db_path.parent().ok_or_else(|| {
        format!("Invalid database path: {} (no parent directory)", db_path_str)
    })?;

    if !parent_dir.exists() {
        fs::create_dir_all(parent_dir)
            .map_err(|e| format!("Failed to create database directory: {}", e))?;
    }

    // 验证目录可写
    let test_file = parent_dir.join(".write_test_db");
    fs::write(&test_file, "test").map_err(|e| {
        format!("Database directory is not writable: {}", e)
    })?;
    let _ = fs::remove_file(&test_file);

    // 检查数据库文件是否已存在
    let already_exists = db_path.exists();

    // 创建数据库连接（自动创建数据库文件）
    let conn = rusqlite::Connection::open(db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;

    // 连接测试：执行简单查询验证可读写
    conn.execute("SELECT 1", [])
        .map_err(|e| format!("Database connection test failed: {}", e))?;

    // 创建基础表结构（如果不存在）
    conn.execute(
        "CREATE TABLE IF NOT EXISTS _db_init_check_ (
            id INTEGER PRIMARY KEY
        )",
        [],
    ).ok();

    // 关闭连接
    drop(conn);

    println!("[DbInit] Database file initialized at: {}", db_path.display());

    Ok(DbInitResult {
        db_path: db_path_str.clone(),
        created: !already_exists,
        connected: true,
        message: if already_exists {
            "Database file already exists, connection verified".to_string()
        } else {
            "Database file created and connection verified".to_string()
        },
    })
}

/// 获取配置（需先调用 init_config）
pub fn get_config() -> Result<AppConfig, String> {
    let config = APP_CONFIG.read().map_err(|_| "配置锁中毒")?;
    config.clone().ok_or_else(|| "配置未初始化，请先调用 init_config".to_string())
}

/// 获取配置并解析路径中的环境变量
pub fn get_config_with_expanded_paths(app_handle: &AppHandle) -> Result<ExpandedConfig, String> {
    let config = get_config()?;
    let install_dir = get_install_dir(app_handle);

    Ok(ExpandedConfig {
        version: config.version,
        database: ExpandedDatabaseConfig {
            path: expand_path(&config.database.path, &install_dir),
            backup_dir: expand_path(&config.database.backup_dir, &install_dir),
            auto_backup: config.database.auto_backup,
            vacuum_on_close: config.database.vacuum_on_close,
        },
        paths: ExpandedPathsConfig {
            projects_root: expand_path(&config.paths.projects_root, &install_dir),
            exports_dir: expand_path(&config.paths.exports_dir, &install_dir),
        },
        logging: ExpandedLoggingConfig {
            level: config.logging.level.clone(),
            directory: expand_path(&config.logging.directory, &install_dir),
            max_file_size_mb: config.logging.max_file_size_mb,
            retention_days: config.logging.retention_days,
            backup_log_enabled: config.logging.backup_log_enabled,
        },
        backup: ExpandedBackupConfig {
            enabled: config.backup.enabled,
            backup_dir: expand_path(&config.backup.backup_dir, &install_dir),
            max_backups_per_project: config.backup.max_backups_per_project,
            incremental_threshold_hours: config.backup.incremental_threshold_hours,
        },
    })
}

/// 展开路径中的环境变量
fn expand_path(path: &str, install_dir: &Path) -> PathBuf {
    let expanded = expand_env_vars(path, install_dir);
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

/// 安装配置信息结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallConfigInfo {
    /// 安装目录
    pub install_dir: String,
    /// 安装配置文件路径
    pub config_path: String,
    /// 配置文件是否已存在
    pub config_exists: bool,
    /// ${APP_INSTALL_DIR} 变量的实际值
    pub app_install_dir_value: String,
}

/// 获取安装目录配置信息（用于前端显示）
#[tauri::command]
pub async fn get_install_config_info(app_handle: AppHandle) -> Result<InstallConfigInfo, String> {
    let install_dir = get_install_dir(&app_handle);
    let config_path = get_config_path(&app_handle);

    Ok(InstallConfigInfo {
        install_dir: install_dir.to_string_lossy().to_string(),
        config_path: config_path.to_string_lossy().to_string(),
        config_exists: config_path.exists(),
        app_install_dir_value: install_dir.to_string_lossy().to_string(),
    })
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
