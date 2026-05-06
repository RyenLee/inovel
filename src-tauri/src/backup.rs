use crate::config::get_log_dir;
use crate::db::{get_db_path, init_db};
use crate::git_snapshot::{get_project_folder_path, open_or_init_repo};
use git2::{DiffFormat, Repository};
use rusqlite::Connection;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Once;
use tauri::AppHandle;
use tracing::{error, info, warn};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use walkdir::WalkDir;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

// ---------------------------------------------------------------------------
// Logging
// ---------------------------------------------------------------------------

/// 获取备份日志目录（从配置读取）
fn get_backup_log_dir(app_handle: &AppHandle) -> PathBuf {
    get_log_dir(app_handle).unwrap_or_else(|_| PathBuf::from("."))
}

static INIT_LOGGING: Once = Once::new();

fn init_backup_logging(app_handle: &AppHandle, _project_id: i64) {
    INIT_LOGGING.call_once(|| {
        let log_dir = get_backup_log_dir(app_handle);
        let _ = fs::create_dir_all(&log_dir);

        let file_appender = RollingFileAppender::new(Rotation::DAILY, &log_dir, "backup.log");
        let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
        // 泄漏 guard，使文件句柄在程序生命周期内保持有效
        std::mem::forget(guard);

        let env_filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("info"));

        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt::layer().with_writer(non_blocking).with_ansi(false))
            .with(fmt::layer().with_writer(std::io::stderr))
            .init();
    });
}

fn log_operation(
    conn: &Connection,
    project_id: i64,
    backup_id: Option<i64>,
    operation: &str,
    message: &str,
    level: &str,
) {
    let created_at = chrono::Local::now().to_rfc3339();
    let _ = conn.execute(
        "INSERT INTO backup_logs (project_id, backup_id, operation, message, level, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![project_id, backup_id, operation, message, level, created_at],
    );
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn escape_filename(name: &str) -> String {
    name.replace('/', "_")
        .replace('\\', "_")
        .replace(':', "_")
        .replace('*', "_")
        .replace('?', "_")
}

fn walkdir_filter(entry: &walkdir::DirEntry, exclude_exports: bool) -> bool {
    let path_str = entry.path().to_string_lossy();
    if path_str.contains(".git") {
        return false;
    }
    if exclude_exports && path_str.contains("exports") {
        return false;
    }
    true
}

/// 通用 ZIP 打包函数（同时返回文件大小）
fn zip_directory(
    dir: &Path,
    exclude_exports: bool,
) -> Result<(Vec<u8>, u64), String> {
    let mut buffer = Vec::new();
    {
        let mut zip = ZipWriter::new(std::io::Cursor::new(&mut buffer));
        let options = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .compression_level(Some(6));

        for entry in WalkDir::new(dir)
            .into_iter()
            .filter_entry(|e| walkdir_filter(e, exclude_exports))
            .filter_map(|e| e.ok())
        {
            let entry_path = entry.path();
            if entry_path == dir {
                continue;
            }
            let relative = entry_path
                .strip_prefix(dir)
                .unwrap_or(entry_path);
            let zip_path_str = relative.to_string_lossy().replace('\\', "/");

            if entry.file_type().is_file() {
                zip.start_file(&zip_path_str, options)
                    .map_err(|e| format!("添加文件失败: {}", e))?;
                let mut f =
                    File::open(entry_path).map_err(|e| format!("打开文件失败: {}", e))?;
                let mut content = Vec::new();
                f.read_to_end(&mut content)
                    .map_err(|e| format!("读取文件失败: {}", e))?;
                zip.write_all(&content)
                    .map_err(|e| format!("写入 zip 失败: {}", e))?;
            } else if entry.file_type().is_dir() && !zip_path_str.is_empty() {
                let dir_path = format!("{}/", zip_path_str);
                zip.add_directory(&dir_path, options)
                    .map_err(|e| format!("添加目录失败: {}", e))?;
            }
        }

        zip.finish()
            .map_err(|e| format!("完成压缩失败: {}", e))?;
    }
    let size = buffer.len() as u64;
    Ok((buffer, size))
}

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

/// 通用备份记录器：创建 backup 记录并写入 zip 文件
fn save_backup_record(
    conn: &Connection,
    project_id: i64,
    backup_type: &str,
    zip_bytes: &[u8],
    destination_path: &Path,
    git_commit: Option<&str>,
    description: &str,
) -> Result<i64, String> {
    let created_at = chrono::Local::now().to_rfc3339();

    // 确保目录存在
    if let Some(parent) = destination_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("创建目标目录失败: {}", e))?;
    }

    // 写入 zip 文件
    let mut file =
        File::create(destination_path).map_err(|e| format!("创建备份文件失败: {}", e))?;
    file.write_all(zip_bytes)
        .map_err(|e| format!("写入备份文件失败: {}", e))?;
    let file_size = zip_bytes.len() as i64;

    conn.execute(
        "INSERT INTO backups (project_id, backup_type, file_path, file_size, git_commit, description, created_at, status) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'completed')",
        rusqlite::params![
            project_id,
            backup_type,
            destination_path.to_string_lossy().to_string(),
            file_size,
            git_commit,
            description,
            created_at
        ],
    )
    .map_err(|e| format!("保存备份记录失败: {}", e))?;

    Ok(conn.last_insert_rowid())
}

/// 创建全量备份
#[tauri::command]
pub fn backup_project(
    app_handle: AppHandle,
    project_id: i64,
    destination_path: String,
    exclude_exports: bool,
    description: Option<String>,
) -> Result<String, String> {
    init_backup_logging(&app_handle, project_id);
    info!(project_id, "开始全量备份");

    let project_path = get_project_folder_path(&app_handle, project_id)?;
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;
    init_db(&conn).map_err(|e| format!("数据库初始化失败: {}", e))?;

    // 获取书名
    let project_name: String = conn
        .query_row(
            "SELECT name FROM projects WHERE id = ?1",
            [project_id],
            |row| row.get(0),
        )
        .map_err(|e| format!("查询项目信息失败: {}", e))?;

    info!(project_id, name = %project_name, "项目信息加载完成");

    // 获取当前 Git HEAD commit（作为增量基准）
    let git_commit = match open_or_init_repo(&project_path) {
        Ok(repo) => repo.head().ok().and_then(|h| h.target()).map(|o| o.to_string()),
        Err(_) => None,
    };

    // 生成 zip 文件名
    let date = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
    let safe_name = escape_filename(&project_name);
    let zip_name = if description.is_some() {
        format!(
            "{}_full_backup_{}.zip",
            safe_name,
            chrono::Local::now().format("%Y%m%d")
        )
    } else {
        format!("{}_full_backup_{}.zip", safe_name, date)
    };

    let dest_path = Path::new(&destination_path).join(&zip_name);
    let desc = description.unwrap_or_else(|| "全量备份".to_string());

    info!(project_id, path = ?dest_path, exclude_exports, "开始打包目录");

    // 打包
    let (zip_bytes, _size) = zip_directory(&project_path, exclude_exports)
        .map_err(|e| {
            error!(project_id, "全量备份失败: {}", e);
            e
        })?;

    info!(project_id, size_kb = zip_bytes.len() / 1024, "ZIP 打包完成");

    // 保存记录
    let backup_id = save_backup_record(
        &conn,
        project_id,
        "full",
        &zip_bytes,
        &dest_path,
        git_commit.as_deref(),
        &desc,
    )
    .map_err(|e| {
        error!(project_id, "保存备份记录失败: {}", e);
        e
    })?;

    info!(
        project_id,
        backup_id,
        path = ?dest_path,
        "全量备份完成"
    );

    log_operation(
        &conn,
        project_id,
        Some(backup_id),
        "backup",
        &format!("全量备份成功: {}", dest_path.to_string_lossy()),
        "info",
    );

    Ok(dest_path.to_string_lossy().to_string())
}

/// 创建增量备份：仅打包自上次备份以来变更的文件
#[tauri::command]
pub fn create_incremental_backup(
    app_handle: AppHandle,
    project_id: i64,
    destination_path: String,
    exclude_exports: bool,
    description: Option<String>,
) -> Result<String, String> {
    init_backup_logging(&app_handle, project_id);
    info!(project_id, "开始增量备份");

    let project_path = get_project_folder_path(&app_handle, project_id)?;
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;
    init_db(&conn).map_err(|e| format!("数据库初始化失败: {}", e))?;

    // 获取书名
    let project_name: String = conn
        .query_row(
            "SELECT name FROM projects WHERE id = ?1",
            [project_id],
            |row| row.get(0),
        )
        .map_err(|e| format!("查询项目信息失败: {}", e))?;

    // 查找最近一次备份对应的 Git commit
    let last_commit: Option<String> = conn
        .query_row(
            "SELECT git_commit FROM backups WHERE project_id = ?1 AND git_commit IS NOT NULL ORDER BY created_at DESC LIMIT 1",
            [project_id],
            |row| row.get(0),
        )
        .ok();

    let current_commit = match open_or_init_repo(&project_path) {
        Ok(repo) => repo.head().ok().and_then(|h| h.target()).map(|o| o.to_string()),
        Err(_) => None,
    };

    // 如果没有历史备份或无法获取 commit，退化为全量备份
    let changed_files: Vec<PathBuf> = if let (Some(base), Some(head)) = (&last_commit, &current_commit) {
        if base != head {
            match Repository::open(&project_path) {
                Ok(repo) => {
                    let base_oid = base.parse().map_err(|e: git2::Error| format!("解析 base commit: {}", e)).ok();
                    let head_oid = head.parse().map_err(|e: git2::Error| format!("解析 head commit: {}", e)).ok();

                    let mut changed = Vec::new();
                    if let (Some(b_oid), Some(h_oid)) = (base_oid, head_oid) {
                        if let (Ok(base_tree), Ok(head_tree)) = (
                            repo.find_commit(b_oid).and_then(|c| c.tree()),
                            repo.find_commit(h_oid).and_then(|c| c.tree()),
                        ) {
                            if let Ok(diff) =
                                repo.diff_tree_to_tree(Some(&base_tree), Some(&head_tree), None)
                            {
                                let _ = diff.print(DiffFormat::Patch, |delta, _, _| {
                                    let path = delta.new_file().path().or_else(|| delta.old_file().path());
                                    if let Some(p) = path {
                                        let full = project_path.join(p);
                                        if full.exists() && full.is_file() {
                                            // 应用过滤
                                            let path_str = full.to_string_lossy();
                                            let passes = if exclude_exports {
                                                !path_str.contains("exports") && !path_str.contains(".git")
                                            } else {
                                                !path_str.contains(".git")
                                            };
                                            if passes {
                                                changed.push(full);
                                            }
                                        }
                                    }
                                    true
                                });
                            }
                        }
                    }
                    changed
                }
                Err(e) => {
                    warn!(project_id, "无法打开 Git 仓库: {}", e);
                    Vec::new()
                }
            }
        } else {
            info!(project_id, "无变更，跳过增量备份");
            log_operation(
                &conn,
                project_id,
                None,
                "incremental",
                "无变更，无需备份",
                "warn",
            );
            return Err("自上次备份以来没有文件变更".to_string());
        }
    } else {
        info!(project_id, "没有历史备份，退化为全量备份");
        // 退化：打包所有文件（无 git 差异可比较）
        WalkDir::new(&project_path)
            .into_iter()
            .filter_entry(|e| walkdir_filter(e, exclude_exports))
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .map(|e| e.path().to_path_buf())
            .collect()
    };

    info!(project_id, changed_files = changed_files.len(), "变更文件数量");

    if changed_files.is_empty() {
        warn!(project_id, "没有变更文件");
        return Err("没有可备份的变更文件".to_string());
    }

    // 生成 zip（仅包含变更文件，保留目录结构）
    let mut buffer = Vec::new();
    {
        let mut zip = ZipWriter::new(std::io::Cursor::new(&mut buffer));
        let options = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .compression_level(Some(6));

        for file_path in &changed_files {
            let relative = file_path
                .strip_prefix(&project_path)
                .unwrap_or(file_path);
            let zip_path_str = relative.to_string_lossy().replace('\\', "/");

            zip.start_file(&zip_path_str, options)
                .map_err(|e| format!("添加文件失败: {}", e))?;
            let mut f =
                File::open(file_path).map_err(|e| format!("打开文件失败: {}", e))?;
            let mut content = Vec::new();
            f.read_to_end(&mut content)
                .map_err(|e| format!("读取文件失败: {}", e))?;
            zip.write_all(&content)
                .map_err(|e| format!("写入 zip 失败: {}", e))?;
        }

        // 写入增量备份清单（manifest）
        let manifest: String = changed_files
            .iter()
            .map(|p| {
                p.strip_prefix(&project_path)
                    .unwrap_or(p)
                    .to_string_lossy()
                    .to_string()
            })
            .collect::<Vec<_>>()
            .join("\n");

        zip.start_file("__MANIFEST.txt", options)
            .map_err(|e| format!("添加清单文件失败: {}", e))?;
        zip.write_all(manifest.as_bytes())
            .map_err(|e| format!("写入清单失败: {}", e))?;

        zip.finish()
            .map_err(|e| format!("完成压缩失败: {}", e))?;
    }

    let zip_bytes = buffer;
    let date = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
    let safe_name = escape_filename(&project_name);
    let zip_name = format!("{}_incr_backup_{}.zip", safe_name, date);
    let dest_path = Path::new(&destination_path).join(&zip_name);
    let desc = description.unwrap_or_else(|| "增量备份".to_string());

    let backup_id = save_backup_record(
        &conn,
        project_id,
        "incremental",
        &zip_bytes,
        &dest_path,
        current_commit.as_deref(),
        &desc,
    )
    .map_err(|e| {
        error!(project_id, "保存增量备份记录失败: {}", e);
        e
    })?;

    info!(
        project_id,
        backup_id,
        path = ?dest_path,
        files = changed_files.len(),
        "增量备份完成"
    );

    log_operation(
        &conn,
        project_id,
        Some(backup_id),
        "incremental",
        &format!(
            "增量备份成功: {} ({} 个文件)",
            dest_path.to_string_lossy(),
            changed_files.len()
        ),
        "info",
    );

    Ok(dest_path.to_string_lossy().to_string())
}

/// 列出项目的所有备份记录
#[tauri::command]
pub fn list_backups(app_handle: AppHandle, project_id: i64) -> Result<Vec<BackupRecord>, String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;
    init_db(&conn).map_err(|e| format!("数据库初始化失败: {}", e))?;

    let mut stmt = conn
        .prepare(
            "SELECT id, project_id, backup_type, file_path, file_size, git_commit, description, created_at, status FROM backups WHERE project_id = ?1 ORDER BY created_at DESC",
        )
        .map_err(|e| format!("查询备份列表失败: {}", e))?;

    let backups = stmt
        .query_map([project_id], |row| {
            Ok(BackupRecord {
                id: row.get(0)?,
                project_id: row.get(1)?,
                backup_type: row.get(2)?,
                file_path: row.get(3)?,
                file_size: row.get(4)?,
                git_commit: row.get(5)?,
                description: row.get(6)?,
                created_at: row.get(7)?,
                status: row.get(8)?,
            })
        })
        .map_err(|e| format!("查询失败: {}", e))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(backups)
}

/// 获取备份操作日志
#[tauri::command]
pub fn get_backup_logs(
    app_handle: AppHandle,
    project_id: i64,
    limit: Option<i64>,
) -> Result<Vec<BackupLogEntry>, String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;
    init_db(&conn).map_err(|e| format!("数据库初始化失败: {}", e))?;

    let limit = limit.unwrap_or(50);
    let mut stmt = conn
        .prepare(
            "SELECT id, project_id, backup_id, operation, message, level, created_at FROM backup_logs WHERE project_id = ?1 ORDER BY created_at DESC LIMIT ?2",
        )
        .map_err(|e| format!("查询日志失败: {}", e))?;

    let logs = stmt
        .query_map([project_id, limit], |row| {
            Ok(BackupLogEntry {
                id: row.get(0)?,
                project_id: row.get(1)?,
                backup_id: row.get(2)?,
                operation: row.get(3)?,
                message: row.get(4)?,
                level: row.get(5)?,
                created_at: row.get(6)?,
            })
        })
        .map_err(|e| format!("查询失败: {}", e))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(logs)
}

/// 恢复备份：从 ZIP 文件恢复到项目目录
#[tauri::command]
pub fn restore_backup(
    app_handle: AppHandle,
    project_id: i64,
    backup_id: i64,
) -> Result<(), String> {
    init_backup_logging(&app_handle, project_id);
    info!(project_id, backup_id, "开始恢复备份");

    let project_path = get_project_folder_path(&app_handle, project_id)?;
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;
    init_db(&conn).map_err(|e| format!("数据库初始化失败: {}", e))?;

    // 读取备份记录
    let (zip_path, backup_type): (String, String) = conn
        .query_row(
            "SELECT file_path, backup_type FROM backups WHERE id = ?1 AND project_id = ?2",
            [backup_id, project_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|e| format!("查找备份记录失败: {}", e))?;

    let zip_path = PathBuf::from(&zip_path);
    if !zip_path.exists() {
        error!(project_id, backup_id, path = ?zip_path, "备份文件不存在");
        return Err(format!("备份文件不存在: {}", zip_path.display()));
    }

    info!(project_id, path = ?zip_path, backup_type, "读取备份文件");

    // 增量备份需要先提取 manifest 确定恢复范围
    if backup_type == "incremental" {
        // 增量备份：只恢复 ZIP 中包含的文件
        let file = File::open(&zip_path)
            .map_err(|e| format!("打开备份文件失败: {}", e))?;
        let mut archive =
            zip::ZipArchive::new(file).map_err(|e| format!("解析 ZIP 失败: {}", e))?;

        let mut manifest_files: Vec<String> = Vec::new();
        for i in 0..archive.len() {
            let mut file = archive.by_index(i).map_err(|e| format!("读取文件失败: {}", e))?;
            let name = file.name().to_string();
            if name == "__MANIFEST.txt" {
                let mut content = String::new();
                file.read_to_string(&mut content)
                    .map_err(|e| format!("读取清单失败: {}", e))?;
                manifest_files = content.lines().map(|s| s.to_string()).collect();
            }
        }

        let manifest_set: std::collections::HashSet<_> =
            manifest_files.iter().collect();
        let total = manifest_set.len();

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).map_err(|e| format!("读取文件失败: {}", e))?;
            let name = file.name().to_string();

            if name == "__MANIFEST.txt" {
                continue;
            }

            // 检查是否在 manifest 中（增量恢复）
            let relative_name = name.replace('\\', "/");
            if !manifest_set.contains(&relative_name) {
                continue;
            }

            let out_path = project_path.join(&name);
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("创建目录失败: {}", e))?;
            }

            let mut out_file = File::create(&out_path)
                .map_err(|e| format!("创建文件失败: {}", e))?;
            std::io::copy(&mut file, &mut out_file)
                .map_err(|e| format!("写入文件失败: {}", e))?;
        }

        info!(project_id, backup_id, total, "增量恢复完成");
        log_operation(
            &conn,
            project_id,
            Some(backup_id),
            "restore",
            &format!("增量恢复成功，共 {} 个文件", total),
            "info",
        );
    } else {
        // 全量备份：清空项目目录后完整恢复
        // 先将当前项目移动到临时备份（防止数据丢失）
        let temp_backup = project_path.parent().map(|p| p.join(".restore_temp")).ok_or("无法确定临时目录")?;
        fs::create_dir_all(&temp_backup).map_err(|e| format!("创建临时目录失败: {}", e))?;
        let temp_project = temp_backup.join("project");
        if temp_project.exists() {
            let _ = fs::remove_dir_all(&temp_project);
        }

        // 如果原目录存在，移入临时备份（作为兜底）
        if project_path.exists() {
            fs::rename(&project_path, &temp_project)
                .map_err(|e| format!("移动原项目目录失败: {}", e))?;
        }
        fs::create_dir_all(&project_path)
            .map_err(|e| format!("创建项目目录失败: {}", e))?;

        let file = File::open(&zip_path)
            .map_err(|e| format!("打开备份文件失败: {}", e))?;
        let mut archive =
            zip::ZipArchive::new(file).map_err(|e| format!("解析 ZIP 失败: {}", e))?;

        let total = archive.len();
        for i in 0..archive.len() {
            let mut file = archive.by_index(i).map_err(|e| format!("读取文件失败: {}", e))?;
            let name = file.name().to_string().replace('\\', "/");
            let out_path = project_path.join(&name);

            if file.is_dir() {
                fs::create_dir_all(&out_path)
                    .map_err(|e| format!("创建目录失败: {}", e))?;
            } else {
                if let Some(parent) = out_path.parent() {
                    fs::create_dir_all(parent)
                        .map_err(|e| format!("创建目录失败: {}", e))?;
                }
                let mut out_file = File::create(&out_path)
                    .map_err(|e| format!("创建文件失败: {}", e))?;
                std::io::copy(&mut file, &mut out_file)
                    .map_err(|e| format!("写入文件失败: {}", e))?;
            }
        }

        // 清理临时备份
        let _ = fs::remove_dir_all(&temp_project);

        info!(project_id, backup_id, total, "全量恢复完成");
        log_operation(
            &conn,
            project_id,
            Some(backup_id),
            "restore",
            &format!("全量恢复成功，共 {} 个文件", total),
            "info",
        );
    }

    info!(project_id, backup_id, "恢复操作完成");
    Ok(())
}

/// 删除备份记录（不删除文件）
#[tauri::command]
pub fn delete_backup_record(
    app_handle: AppHandle,
    project_id: i64,
    backup_id: i64,
) -> Result<(), String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;
    init_db(&conn).map_err(|e| format!("数据库初始化失败: {}", e))?;

    conn.execute(
        "DELETE FROM backups WHERE id = ?1 AND project_id = ?2",
        [backup_id, project_id],
    )
    .map_err(|e| format!("删除备份记录失败: {}", e))?;

    log_operation(
        &conn,
        project_id,
        Some(backup_id),
        "delete",
        "删除了备份记录",
        "warn",
    );

    Ok(())
}

/// 获取最近的备份统计摘要
#[tauri::command]
pub fn get_backup_stats(
    app_handle: AppHandle,
    project_id: i64,
) -> Result<BackupStats, String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;
    init_db(&conn).map_err(|e| format!("数据库初始化失败: {}", e))?;

    let total: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM backups WHERE project_id = ?1",
            [project_id],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let full_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM backups WHERE project_id = ?1 AND backup_type = 'full'",
            [project_id],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let incr_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM backups WHERE project_id = ?1 AND backup_type = 'incremental'",
            [project_id],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let total_size: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(file_size), 0) FROM backups WHERE project_id = ?1",
            [project_id],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let last_backup: Option<String> = conn
        .query_row(
            "SELECT created_at FROM backups WHERE project_id = ?1 ORDER BY created_at DESC LIMIT 1",
            [project_id],
            |row| row.get(0),
        )
        .ok();

    Ok(BackupStats {
        total,
        full_count,
        incr_count,
        total_size,
        last_backup,
    })
}

// ---------------------------------------------------------------------------
// Data models
// ---------------------------------------------------------------------------

#[derive(Debug, serde::Serialize)]
pub struct BackupRecord {
    pub id: i64,
    pub project_id: i64,
    pub backup_type: String,
    pub file_path: String,
    pub file_size: i64,
    pub git_commit: Option<String>,
    pub description: String,
    pub created_at: String,
    pub status: String,
}

#[derive(Debug, serde::Serialize)]
pub struct BackupLogEntry {
    pub id: i64,
    pub project_id: i64,
    pub backup_id: Option<i64>,
    pub operation: String,
    pub message: String,
    pub level: String,
    pub created_at: String,
}

#[derive(Debug, serde::Serialize)]
pub struct BackupStats {
    pub total: i64,
    pub full_count: i64,
    pub incr_count: i64,
    pub total_size: i64,
    pub last_backup: Option<String>,
}
