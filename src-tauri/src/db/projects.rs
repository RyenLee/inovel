use chrono::Utc;
use rusqlite::{Connection, OptionalExtension, Result as SqliteResult};
use std::path::PathBuf;

use crate::models::ProjectMeta;

/// 创建新项目记录
/// 
/// 在数据库中插入一条新的项目记录，并返回完整的项目元数据。
/// 
/// # 参数
/// - `conn`: SQLite 数据库连接
/// - `project_id`: 项目唯一标识符（如 "P7K3M9"）
/// - `name`: 项目名称（书名）
/// - `author`: 作者名
/// - `description`: 项目描述
/// - `path`: 项目文件夹路径
/// 
/// # 返回值
/// 创建成功返回 `Ok(ProjectMeta)`，失败返回 `rusqlite::Error`
pub fn create_project(
    conn: &Connection,
    project_id: &str,
    name: &str,
    author: &str,
    description: &str,
    path: &str,
) -> SqliteResult<ProjectMeta> {
    let now = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO projects (project_id, name, author, description, path, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        (project_id, name, author, description, path, &now),
    )?;

    Ok(ProjectMeta {
        id: conn.last_insert_rowid(),
        project_id: project_id.to_string(),
        name: name.to_string(),
        author: author.to_string(),
        description: description.to_string(),
        path: path.to_string(),
        created_at: now,
        last_opened_at: None,
        is_valid: true,
        cover_path: None,
        encrypted: false,
    })
}

/// 根据数据库 ID 获取项目
/// 
/// 通过数据库主键 ID 查询项目记录。
/// 
/// # 参数
/// - `conn`: SQLite 数据库连接
/// - `project_id`: 数据库主键 ID
/// 
/// # 返回值
/// 找到返回 `Ok(Some(ProjectMeta))`，未找到返回 `Ok(None)`，出错返回 `rusqlite::Error`
pub fn get_project_by_id(conn: &Connection, project_id: i64) -> SqliteResult<Option<ProjectMeta>> {
    conn.query_row(
        "SELECT id, project_id, name, author, description, path, created_at, last_opened_at
         FROM projects WHERE id = ?1",
        [project_id],
        |row| {
            Ok(ProjectMeta {
                id: row.get(0)?,
                project_id: row.get(1)?,
                name: row.get(2)?,
                author: row.get(3)?,
                description: row.get(4)?,
                path: row.get(5)?,
                created_at: row.get(6)?,
                last_opened_at: row.get(7)?,
                is_valid: true,
                cover_path: None,
                encrypted: false,
            })
        },
    )
    .optional()
}

/// 根据项目唯一标识符获取项目
/// 
/// 通过项目唯一标识符（如 "P7K3M9"）查询项目记录。
/// 
/// # 参数
/// - `conn`: SQLite 数据库连接
/// - `project_id`: 项目唯一标识符
/// 
/// # 返回值
/// 找到返回 `Ok(Some(ProjectMeta))`，未找到返回 `Ok(None)`，出错返回 `rusqlite::Error`
pub fn get_project_by_project_id(
    conn: &Connection,
    project_id: &str,
) -> SqliteResult<Option<ProjectMeta>> {
    conn.query_row(
        "SELECT id, project_id, name, author, description, path, created_at, last_opened_at
         FROM projects WHERE project_id = ?1",
        [project_id],
        |row| {
            Ok(ProjectMeta {
                id: row.get(0)?,
                project_id: row.get(1)?,
                name: row.get(2)?,
                author: row.get(3)?,
                description: row.get(4)?,
                path: row.get(5)?,
                created_at: row.get(6)?,
                last_opened_at: row.get(7)?,
                is_valid: true,
                cover_path: None,
                encrypted: false,
            })
        },
    )
    .optional()
}

/// 获取所有项目列表
/// 
/// 查询数据库中所有项目，按创建时间倒序排列。
/// 
/// # 参数
/// - `conn`: SQLite 数据库连接
/// 
/// # 返回值
/// 项目列表 `Ok(Vec<ProjectMeta>)`，出错返回 `rusqlite::Error`
pub fn get_all_projects(conn: &Connection) -> SqliteResult<Vec<ProjectMeta>> {
    let mut stmt = conn.prepare("SELECT id, project_id, name, author, description, path, created_at, last_opened_at FROM projects ORDER BY created_at DESC")?;
    let projects = stmt
        .query_map([], |row| {
            Ok(ProjectMeta {
                id: row.get(0)?,
                project_id: row.get(1)?,
                name: row.get(2)?,
                author: row.get(3)?,
                description: row.get(4)?,
                path: row.get(5)?,
                created_at: row.get(6)?,
                last_opened_at: row.get(7)?,
                is_valid: true,
                cover_path: None,
                encrypted: false,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();
    Ok(projects)
}

/// 更新项目信息
/// 
/// 更新项目的名称、作者和描述字段。
/// 
/// # 参数
/// - `conn`: SQLite 数据库连接
/// - `id`: 项目数据库主键 ID
/// - `name`: 新的项目名称
/// - `author`: 新的作者名
/// - `description`: 新的项目描述
/// 
/// # 返回值
/// 更新成功返回 `Ok(())`，失败返回 `rusqlite::Error`
pub fn update_project(
    conn: &Connection,
    id: i64,
    name: &str,
    author: &str,
    description: &str,
) -> SqliteResult<()> {
    conn.execute(
        "UPDATE projects SET name = ?1, author = ?2, description = ?3 WHERE id = ?4",
        (name, author, description, id),
    )?;
    Ok(())
}

/// 更新项目最后打开时间
/// 
/// 记录项目的最后访问时间，用于最近打开列表排序。
/// 
/// # 参数
/// - `conn`: SQLite 数据库连接
/// - `id`: 项目数据库主键 ID
/// 
/// # 返回值
/// 更新成功返回 `Ok(())`，失败返回 `rusqlite::Error`
pub fn update_project_last_opened(conn: &Connection, id: i64) -> SqliteResult<()> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE projects SET last_opened_at = ?1 WHERE id = ?2",
        (&now, id),
    )?;
    Ok(())
}

/// 删除项目记录
/// 
/// 从数据库中删除指定项目记录。注意：此操作会触发外键级联删除。
/// 
/// # 参数
/// - `conn`: SQLite 数据库连接
/// - `id`: 项目数据库主键 ID
/// 
/// # 返回值
/// 返回删除的行数 `Ok(usize)`，失败返回 `rusqlite::Error`
pub fn delete_project(conn: &Connection, id: i64) -> SqliteResult<usize> {
    conn.execute("DELETE FROM projects WHERE id = ?1", [id])
}

/// 检查项目文件夹是否存在
/// 
/// 判断指定路径是否存在且为目录，用于验证项目的有效性。
/// 
/// # 参数
/// - `path`: 项目文件夹路径
/// 
/// # 返回值
/// 存在且为目录返回 `true`，否则返回 `false`
pub fn check_project_exists(path: &str) -> bool {
    let project_path = PathBuf::from(path);
    project_path.exists() && project_path.is_dir()
}
