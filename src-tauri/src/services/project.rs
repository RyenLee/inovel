use chrono::Utc;
use rusqlite::Connection;
use tauri::AppHandle;
use tracing::info;

use crate::config;
use crate::error::{AppError, Result};
use crate::models::{CreateProjectParams, ProjectMeta};

/// 项目服务层
/// 
/// 提供项目相关的业务逻辑，包括创建、查询、删除等操作。
/// 该服务封装了数据库访问和业务逻辑，向上层提供简洁的 API。
pub struct ProjectService;

impl ProjectService {
    /// 创建新项目
    /// 
    /// 创建一个新的小说项目，包括在数据库中记录项目元数据。
    /// 
    /// # 参数
    /// - `app_handle`: Tauri 应用句柄
    /// - `params`: 创建项目的参数（包含名称、作者、描述、路径）
    /// 
    /// # 返回值
    /// 创建成功返回 `Ok(ProjectMeta)`，失败返回 `AppError`
    pub fn create_project(
        app_handle: &AppHandle,
        params: &CreateProjectParams,
    ) -> Result<ProjectMeta> {
        let db_path = config::get_db_path(app_handle);
        let conn = Connection::open(&db_path)?;

        // 使用当前时间戳作为数据库主键 ID
        let id = Utc::now().timestamp_millis();
        // 生成唯一的项目标识符（6位字母数字组合）
        let project_id = generate_project_id();
        let now = Utc::now().to_rfc3339();

        conn.execute(
            "INSERT INTO projects (id, name, author, description, path, created_at) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            (
                id,
                &params.name,
                &params.author,
                &params.description,
                &params.path,
                &now,
            ),
        )?;

        info!(project_id = %id, name = %params.name, "项目创建成功");

        Ok(ProjectMeta {
            id,
            project_id: project_id.clone(),
            name: params.name.clone(),
            author: params.author.clone(),
            description: params.description.clone(),
            path: params.path.clone(),
            created_at: now,
            last_opened_at: None,
            is_valid: true,
            cover_path: None,
            encrypted: false,
        })
    }

    /// 获取最近创建的项目列表
/// 
/// 查询最近创建的项目，按创建时间倒序排列，可限制返回数量。
/// 
/// # 参数
/// - `app_handle`: Tauri 应用句柄
/// - `limit`: 返回的最大项目数量
/// 
/// # 返回值
/// 项目列表 `Ok(Vec<ProjectMeta>)`，失败返回 `AppError`
    pub fn get_recent_projects(app_handle: &AppHandle, limit: u32) -> Result<Vec<ProjectMeta>> {
    let db_path = config::get_db_path(app_handle);
    let conn = Connection::open(&db_path)?;

    let mut stmt = conn.prepare(
        "SELECT id, name, author, description, path, created_at 
             FROM projects ORDER BY created_at DESC LIMIT ?1",
    )?;

    let projects = stmt
        .query_map([limit], |row| {
            Ok(ProjectMeta {
                id: row.get(0)?,
                project_id: String::new(),
                name: row.get(1)?,
                author: row.get(2)?,
                description: row.get(3)?,
                path: row.get(4)?,
                created_at: row.get(5)?,
                last_opened_at: None,
                is_valid: true,
                cover_path: None,
                encrypted: false,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(projects)
}

/// 删除项目
/// 
/// 从数据库中删除指定项目记录。如果项目不存在，返回 NotFound 错误。
/// 
/// # 参数
/// - `app_handle`: Tauri 应用句柄
/// - `project_id`: 项目数据库主键 ID
/// 
/// # 返回值
/// 删除成功返回 `Ok(())`，失败返回 `AppError`
pub fn delete_project(app_handle: &AppHandle, project_id: i64) -> Result<()> {
    let db_path = config::get_db_path(app_handle);
    let conn = Connection::open(&db_path)?;

    let rows_affected = conn.execute("DELETE FROM projects WHERE id = ?1", [project_id])?;

    if rows_affected == 0 {
        return Err(AppError::not_found(format!("项目 {} 不存在", project_id)));
    }

    info!(project_id = %project_id, "项目删除成功");
    Ok(())
}
}

/// 生成项目唯一标识符
/// 
/// 生成一个 6 位的字母数字组合作为项目唯一标识符。
/// 排除了容易混淆的字符（O, I, 0, 1），提高可读性。
/// 
/// # 返回值
/// 6 位字母数字组合的字符串（如 "P7K3M9"）
fn generate_project_id() -> String {
    // 排除易混淆字符：O, I, 0, 1
    const CHARSET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
    (0..6)
        .map(|_| {
            let idx = rand::random_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}
