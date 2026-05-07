//! 应用文件系统路径管理模块
//!
//! 所有路径基于应用程序安装目录计算，无需外部配置文件。
//!
//! 文件结构:
//! ```text
//! {INSTALL_DIR}/                  (应用程序安装目录)
//! ├── data/                        (应用数据目录)
//! │   ├── inovel.db               (SQLite 数据库文件)
//! ├── projects/                   (项目文件夹)
//! ├── logs/                       (日志文件夹)
//! ├── backups/                    (备份文件夹)
//! └── database/                   (数据库相关文件夹)
//!     └── backups/                (数据库备份)
//! ```

use std::fs;
use std::path::PathBuf;
use tauri::AppHandle;
use tauri::Manager;

pub fn get_install_dir(app_handle: &AppHandle) -> PathBuf {
    if let Ok(resource_path) = app_handle.path().resource_dir() {
        return resource_path;
    }

    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(parent) = exe_path.parent() {
            return parent.to_path_buf();
        }
    }

    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

pub fn get_data_dir_for_webview() -> PathBuf {
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(parent) = exe_path.parent() {
            let dir = parent.join("data");
            fs::create_dir_all(&dir).ok();
            return dir;
        }
    }
    
    let dir = PathBuf::from("data");
    fs::create_dir_all(&dir).ok();
    dir
}

pub fn get_data_dir(app_handle: &AppHandle) -> PathBuf {
    let dir = get_install_dir(app_handle).join("data");
    fs::create_dir_all(&dir).ok();
    dir
}

pub fn get_db_path(app_handle: &AppHandle) -> PathBuf {
    let data_dir = get_data_dir(app_handle);
    data_dir.join("inovel.db")
}

pub fn get_projects_root(app_handle: &AppHandle) -> PathBuf {
    let dir = get_install_dir(app_handle).join("projects");
    fs::create_dir_all(&dir).ok();
    dir
}

pub fn get_exports_dir(app_handle: &AppHandle) -> PathBuf {
    let dir = get_install_dir(app_handle).join("exports");
    fs::create_dir_all(&dir).ok();
    dir
}

pub fn get_log_dir(app_handle: &AppHandle) -> PathBuf {
    let dir = get_install_dir(app_handle).join("logs");
    fs::create_dir_all(&dir).ok();
    dir
}

pub fn get_backup_dir(app_handle: &AppHandle) -> PathBuf {
    let dir = get_install_dir(app_handle).join("backups");
    fs::create_dir_all(&dir).ok();
    dir
}

pub fn get_db_backup_dir(app_handle: &AppHandle) -> PathBuf {
    let dir = get_install_dir(app_handle).join("database").join("backups");
    fs::create_dir_all(&dir).ok();
    dir
}
