use crate::commands::git_snapshot::init_git_repo;
use crate::db::{check_project_exists, get_db_path, init_db};
use crate::logging::operation::record_simple_operation;
use crate::models::{
    CreateProjectParams, MigrateResult, MigrationDetail, PaginatedProjects, ProjectMeta,
    RollbackParams, UpdateProjectParams,
};
use rand::RngExt;
use rusqlite::Connection;
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

/// 生成字母数字组合的项目ID
///
/// 格式：字母P开头 + 5位字母数字混合（如 "P7K3M9"）
/// 排除了容易混淆的字符：O, I, 0, 1
fn generate_project_id() -> String {
    const CHARSET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
    let mut rng = rand::rng();
    let id: String = (0..5)
        .map(|_| {
            let idx = rng.random_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();
    format!("P{}", id)
}

/// 检查项目ID是否已存在于数据库中
///
/// # 参数
/// - `conn`: SQLite 数据库连接
/// - `project_id`: 要检查的项目ID
///
/// # 返回值
/// 如果存在返回 `true`，否则返回 `false`
fn check_project_id_exists(conn: &Connection, project_id: &str) -> bool {
    conn.query_row(
        "SELECT COUNT(*) FROM projects WHERE project_id = ?1",
        [project_id],
        |row| row.get::<_, i32>(0),
    )
    .map(|count| count > 0)
    .unwrap_or(false)
}

/// 生成唯一的项目ID
///
/// 循环生成项目ID直到找到一个不存在于数据库中的ID，最多尝试100次。
///
/// # 参数
/// - `conn`: SQLite 数据库连接
///
/// # 返回值
/// 成功返回唯一的项目ID `Ok(String)`，失败返回错误信息 `Err(String)`
fn generate_unique_project_id(conn: &Connection) -> Result<String, String> {
    let mut max_attempts = 100;
    while max_attempts > 0 {
        let id = generate_project_id();
        if !check_project_id_exists(conn, &id) {
            return Ok(id);
        }
        max_attempts -= 1;
    }
    Err("无法生成唯一项目ID".to_string())
}

/// 项目配置文件结构
///
/// 统一从 project.json 读取的配置项
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct ProjectConfig {
    pub name: String,
    pub author: String,
    pub description: String,
    pub project_id: String,
    pub cover_path: Option<String>,
    pub encrypted: bool,
    pub writing_goal: i32,
}

/// 从 project.json 读取项目配置
///
/// # 参数
/// - `project_path`: 项目根目录路径
///
/// # 返回值
/// 成功返回项目配置 `Ok(ProjectConfig)`，失败返回 `Err(String)`
fn read_project_config(project_path: &std::path::Path) -> Result<ProjectConfig, String> {
    let project_json_path = project_path.join("project.json");
    let content = fs::read_to_string(&project_json_path)
        .map_err(|e| format!("读取项目配置文件失败: {}", e))?;
    let json: serde_json::Value =
        serde_json::from_str(&content).map_err(|e| format!("解析项目配置文件失败: {}", e))?;

    let name = json
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let author = json
        .get("author")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let description = json
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let project_id = json
        .get("project_id")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let cover_path = json.get("cover_path").and_then(|v| v.as_str()).map(|s| {
        let full_path = project_path.join(s);
        full_path.to_string_lossy().to_string().replace('\\', "/")
    });
    let encrypted = json
        .get("encrypted")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let writing_goal = json
        .get("writing_goal")
        .and_then(|v| v.as_i64())
        .unwrap_or(3000) as i32;

    Ok(ProjectConfig {
        name,
        author,
        description,
        project_id,
        cover_path,
        encrypted,
        writing_goal,
    })
}

/// 创建新项目
///
/// 创建一个新的小说项目，包括：
/// 1. 验证路径有效性
/// 2. 生成唯一项目ID
/// 3. 创建项目目录结构
/// 4. 初始化 Git 仓库
/// 5. 创建项目配置文件
/// 6. 记录到数据库
///
/// # 参数
/// - `app_handle`: Tauri 应用句柄
/// - `params`: 创建项目的参数（名称、作者、描述、路径）
///
/// # 返回值
/// 创建成功返回项目元数据 `Ok(ProjectMeta)`，失败返回错误信息 `Err(String)`
#[tauri::command]
pub async fn create_project(
    app_handle: AppHandle,
    params: CreateProjectParams,
) -> Result<ProjectMeta, String> {
    let base_path = PathBuf::from(&params.path);
    if !base_path.exists() {
        return Err("指定的路径不存在".to_string());
    }
    if !base_path.is_dir() {
        return Err("指定的路径不是有效的目录".to_string());
    }

    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;
    init_db(&conn).map_err(|e| format!("数据库初始化失败: {}", e))?;

    // 生成唯一的项目ID（字母+数字组合）
    let project_id = generate_unique_project_id(&conn)?;

    // 使用 project_id 作为文件夹名称
    let project_folder = base_path.join(&project_id);
    if project_folder.exists() {
        return Err("项目文件夹已存在".to_string());
    }

    // 创建项目目录和章节目录
    fs::create_dir_all(&project_folder).map_err(|e| format!("创建项目目录失败: {}", e))?;
    let chapters_dir = project_folder.join("chapters");
    fs::create_dir_all(&chapters_dir).map_err(|e| format!("创建章节目录失败: {}", e))?;

    // 初始化 Git 仓库（失败时静默处理）
    let _ = init_git_repo(&project_folder);

    // 创建项目配置文件
    let now = chrono::Utc::now().to_rfc3339();
    let project_json = serde_json::json!({
        "name": params.name,
        "author": params.author,
        "description": params.description,
        "created_at": now,
        "project_id": project_id.clone(),
        "writing_goal": 3000,
        "encrypted": false
    });
    let project_json_path = project_folder.join("project.json");
    fs::write(
        &project_json_path,
        serde_json::to_string_pretty(&project_json)
            .map_err(|e| format!("序列化项目配置失败: {}", e))?,
    )
    .map_err(|e| format!("写入项目配置文件失败: {}", e))?;

    // 插入数据库记录
    let full_project_path = project_folder.to_string_lossy().to_string();
    conn.execute(
        "INSERT INTO projects (project_id, name, author, description, path, created_at, last_opened_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        (&project_id, &params.name, &params.author, &params.description, full_project_path.as_str(), &now, &now),
    ).map_err(|e| format!("数据库插入失败: {}", e))?;

    let id = conn.last_insert_rowid();

    let _ = record_simple_operation(
        &app_handle,
        "project",
        "create",
        "project",
        Some(id),
        Some(&format!("创建项目: {}", params.name)),
        Some(id),
    );

    Ok(ProjectMeta {
        id,
        project_id: project_id.clone(),
        name: params.name,
        author: params.author,
        description: params.description,
        path: project_folder.to_string_lossy().to_string(),
        created_at: now.clone(),
        last_opened_at: Some(now),
        is_valid: true,
        cover_path: None,
        encrypted: false,
    })
}

/// 获取最近打开的项目列表（分页）
///
/// 查询最近打开的项目（按最后打开时间倒序），支持分页。
///
/// # 参数
/// - `app_handle`: Tauri 应用句柄
/// - `page`: 页码（从1开始，默认1）
/// - `page_size`: 每页项目数（默认5）
///
/// # 返回值
/// 分页项目列表 `Ok(PaginatedProjects)`，失败返回错误信息 `Err(String)`
#[tauri::command]
pub async fn get_recent_projects(
    app_handle: AppHandle,
    page: Option<i32>,
    page_size: Option<i32>,
) -> Result<PaginatedProjects, String> {
    let page = page.unwrap_or(1).max(1);
    let page_size = page_size.unwrap_or(5).max(1).min(100);
    let offset = (page - 1) * page_size;

    let db_path = get_db_path(&app_handle);
    if !db_path.exists() {
        return Ok(PaginatedProjects {
            items: vec![],
            total: 0,
            page,
            page_size,
            total_pages: 0,
        });
    }
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;
    let _ = init_db(&conn);

    let total: i64 = conn
        .query_row("SELECT COUNT(*) FROM projects", [], |row| row.get(0))
        .map_err(|e| format!("查询总数失败: {}", e))?;

    let total_pages = if total == 0 {
        0
    } else {
        ((total as f64) / (page_size as f64)).ceil() as i32
    };

    let mut stmt = conn
        .prepare("SELECT id, project_id, name, author, description, path, created_at, last_opened_at FROM projects ORDER BY COALESCE(last_opened_at, created_at) DESC LIMIT ?1 OFFSET ?2")
        .map_err(|e| format!("查询准备失败: {}", e))?;

    let projects: Vec<ProjectMeta> = stmt
        .query_map([page_size, offset], |row| {
            let path: String = row.get(5)?;
            let project_id: String = row.get::<_, Option<String>>(1)?.unwrap_or_default();

            let config = read_project_config(std::path::Path::new(&path)).ok();

            Ok(ProjectMeta {
                id: row.get(0)?,
                project_id,
                name: row.get(2)?,
                author: row.get(3)?,
                description: row.get(4)?,
                path: path.clone(),
                created_at: row.get(6)?,
                last_opened_at: row.get(7)?,
                is_valid: check_project_exists(&path),
                cover_path: config.as_ref().and_then(|c| c.cover_path.clone()),
                encrypted: config.as_ref().map(|c| c.encrypted).unwrap_or(false),
            })
        })
        .map_err(|e| format!("查询执行失败: {}", e))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(PaginatedProjects {
        items: projects,
        total,
        page,
        page_size,
        total_pages,
    })
}

/// 打开项目
///
/// 打开指定项目并更新最后打开时间。同时读取封面路径和加密状态。
///
/// # 参数
/// - `app_handle`: Tauri 应用句柄
/// - `id`: 项目数据库 ID
///
/// # 返回值
/// 项目元数据 `Ok(ProjectMeta)`，失败返回错误信息 `Err(String)`
#[tauri::command]
pub async fn open_project(app_handle: AppHandle, id: i64) -> Result<ProjectMeta, String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;
    let now = chrono::Utc::now().to_rfc3339();

    // 更新最后打开时间
    conn.execute(
        "UPDATE projects SET last_opened_at = ?1 WHERE id = ?2",
        (&now, id),
    )
    .map_err(|e| format!("更新失败: {}", e))?;

    // 查询项目信息
    let mut stmt = conn
        .prepare("SELECT id, project_id, name, author, description, path, created_at, last_opened_at FROM projects WHERE id = ?1")
        .map_err(|e| format!("查询准备失败: {}", e))?;
    let project = stmt
        .query_row([id], |row| {
            let path: String = row.get(5)?;
            let project_id: String = row.get::<_, Option<String>>(1)?.unwrap_or_default();
            Ok(ProjectMeta {
                id: row.get(0)?,
                project_id,
                name: row.get(2)?,
                author: row.get(3)?,
                description: row.get(4)?,
                path: path.clone(),
                created_at: row.get(6)?,
                last_opened_at: row.get(7)?,
                is_valid: check_project_exists(&path),
                cover_path: None,
                encrypted: false,
            })
        })
        .map_err(|e| format!("项目不存在: {}", e))?;

    let config = read_project_config(std::path::Path::new(&project.path)).ok();

    Ok(ProjectMeta {
        cover_path: config.as_ref().and_then(|c| c.cover_path.clone()),
        encrypted: config.as_ref().map(|c| c.encrypted).unwrap_or(false),
        ..project
    })
}

/// 从列表中移除项目（可选删除本地文件）
///
/// # 参数
/// - `app_handle`: Tauri 应用句柄
/// - `id`: 项目数据库 ID
/// - `keep_files`: 是否保留本地文件
///   - `true` (默认): 仅从数据库删除记录，保留本地文件夹
///   - `false`: 同时删除数据库记录和本地文件夹
///
/// # 返回值
/// 成功返回 `Ok(())`，失败返回错误信息 `Err(String)`
#[tauri::command]
pub async fn remove_project_from_list(
    app_handle: AppHandle,
    id: i64,
    keep_files: Option<bool>,
) -> Result<(), String> {
    let keep_files = keep_files.unwrap_or(true);
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;

    // 获取项目路径（用于可能删除文件夹）
    let project_path: Option<String> = if !keep_files {
        conn.query_row("SELECT path FROM projects WHERE id = ?1", [id], |row| {
            row.get(0)
        })
        .ok()
    } else {
        None
    };

    // 从数据库删除项目记录（级联删除会处理子表）
    conn.execute("DELETE FROM projects WHERE id = ?1", [id])
        .map_err(|e| format!("从数据库删除失败: {}", e))?;

    // 如果不保留文件，删除本地文件夹
    if !keep_files {
        if let Some(path) = project_path {
            let project_dir = PathBuf::from(&path);
            if project_dir.exists() && project_dir.is_dir() {
                fs::remove_dir_all(&project_dir)
                    .map_err(|e| format!("删除项目文件夹失败: {}", e))?;
            }
        }
    }

    let _ = record_simple_operation(
        &app_handle,
        "project",
        "remove",
        "project",
        Some(id),
        Some(&format!(
            "移除项目{}",
            if keep_files {
                "(保留文件)"
            } else {
                "(删除文件)"
            }
        )),
        Some(id),
    );

    Ok(())
}

/// 更新项目信息
///
/// 更新项目的名称、作者和描述信息，同时更新项目配置文件。
///
/// # 参数
/// - `app_handle`: Tauri 应用句柄
/// - `id`: 项目数据库 ID
/// - `params`: 更新项目的参数（名称、作者、描述）
///
/// # 返回值
/// 更新后的项目元数据 `Ok(ProjectMeta)`，失败返回错误信息 `Err(String)`
#[tauri::command]
pub async fn update_project(
    app_handle: AppHandle,
    id: i64,
    params: UpdateProjectParams,
) -> Result<ProjectMeta, String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;

    // 更新数据库记录
    conn.execute(
        "UPDATE projects SET name = ?1, author = ?2, description = ?3 WHERE id = ?4",
        (&params.name, &params.author, &params.description, id),
    )
    .map_err(|e| format!("更新项目失败: {}", e))?;

    // 获取项目路径并更新 project.json
    let path: String = conn
        .query_row("SELECT path FROM projects WHERE id = ?1", [id], |row| {
            row.get(0)
        })
        .map_err(|e| format!("项目不存在: {}", e))?;
    let project_json_path = std::path::Path::new(&path).join("project.json");

    // 更新 project.json（保留现有字段如 cover_path、encrypted）
    if project_json_path.exists() {
        if let Ok(content) = fs::read_to_string(&project_json_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                let mut updated = json.clone();
                updated["name"] = serde_json::Value::String(params.name.clone());
                updated["author"] = serde_json::Value::String(params.author.clone());
                updated["description"] = serde_json::Value::String(params.description.clone());
                let _ = fs::write(
                    &project_json_path,
                    serde_json::to_string_pretty(&updated).unwrap_or_default(),
                );
            }
        }
    };

    // 使用统一方式读取配置获取 cover_path 和 encrypted
    let final_config = read_project_config(std::path::Path::new(&path)).ok();

    // 查询更新后的项目信息
    let mut stmt = conn
        .prepare("SELECT id, project_id, name, author, description, path, created_at, last_opened_at FROM projects WHERE id = ?1")
        .map_err(|e| format!("查询准备失败: {}", e))?;
    let project = stmt
        .query_row([id], |row| {
            let path: String = row.get(5)?;
            let project_id: String = row.get::<_, Option<String>>(1)?.unwrap_or_default();
            Ok(ProjectMeta {
                id: row.get(0)?,
                project_id,
                name: row.get(2)?,
                author: row.get(3)?,
                description: row.get(4)?,
                path: path.clone(),
                created_at: row.get(6)?,
                last_opened_at: row.get(7)?,
                is_valid: check_project_exists(&path),
                cover_path: None,
                encrypted: false,
            })
        })
        .map_err(|e| format!("查询项目失败: {}", e))?;

    Ok(ProjectMeta {
        cover_path: final_config.as_ref().and_then(|c| c.cover_path.clone()),
        encrypted: final_config.as_ref().map(|c| c.encrypted).unwrap_or(false),
        ..project
    })
}

/// 清理文件名中的非法字符
///
/// 将文件名中的非法字符替换为下划线，确保文件名符合文件系统规范。
/// 非法字符包括：/, \, :, *, ?, ", <, >, |
///
/// # 参数
/// - `name`: 原始文件名
///
/// # 返回值
/// 清理后的文件名
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => c,
        })
        .collect::<String>()
        .trim()
        .to_string()
}

/// 设置项目封面
///
/// 将指定图片设置为项目封面，封面将保存到项目目录下的 `covers/{书名}_cover.{扩展名}`。
/// 路径存储为相对于项目根目录的路径，便于项目迁移。
///
/// # 参数
/// - `app_handle`: Tauri 应用句柄
/// - `id`: 项目数据库 ID
/// - `image_path`: 源图片文件路径
///
/// # 返回值
/// 封面文件的完整路径 `Ok(String)`，失败返回错误信息 `Err(String)`
#[tauri::command]
pub async fn set_cover(
    app_handle: AppHandle,
    id: i64,
    image_path: String,
) -> Result<String, String> {
    // 获取项目路径和书名
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;

    let (project_path, book_name): (String, String) = conn
        .query_row(
            "SELECT path, name FROM projects WHERE id = ?1",
            [id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|e| format!("项目不存在: {}", e))?;

    let project_dir = PathBuf::from(&project_path);

    // 验证源图片存在
    let source_path = PathBuf::from(&image_path);
    if !source_path.exists() {
        return Err("图片文件不存在".to_string());
    }

    // 获取原图扩展名
    let extension = source_path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("jpg")
        .to_lowercase();

    // 清理书名作为文件名的一部分
    let sanitized_name = sanitize_filename(&book_name);
    if sanitized_name.is_empty() {
        return Err("书名不能为空".to_string());
    }

    // 创建 covers 目录（如果不存在）
    let covers_dir = project_dir.join("covers");
    if !covers_dir.exists() {
        fs::create_dir_all(&covers_dir).map_err(|e| format!("创建封面目录失败: {}", e))?;
    }

    // 生成封面文件名：cover.{扩展名}
    let cover_filename = format!("cover.{}", extension);
    let new_cover_path = covers_dir.join(&cover_filename);

    // 读取项目配置获取旧的封面路径（用于清理旧文件）
    let project_json_path = project_dir.join("project.json");
    let old_cover_path: Option<String> = if project_json_path.exists() {
        fs::read_to_string(&project_json_path)
            .ok()
            .and_then(|c| serde_json::from_str::<serde_json::Value>(&c).ok())
            .and_then(|json| json.get("cover_path")?.as_str().map(|s| s.to_string()))
    } else {
        None
    };

    // 删除旧的封面文件（如果存在且与新封面路径不同）
    if let Some(old_path) = old_cover_path {
        let old_full_path = if PathBuf::from(&old_path).is_absolute() {
            PathBuf::from(&old_path)
        } else {
            project_dir.join(&old_path)
        };
        if old_full_path.exists() && old_full_path != new_cover_path {
            let _ = fs::remove_file(&old_full_path);
        }
    }

    // 处理文件名冲突：如果文件已存在，先删除
    if new_cover_path.exists() {
        fs::remove_file(&new_cover_path).map_err(|e| format!("删除旧封面失败: {}", e))?;
    }

    // 复制图片到目标位置
    fs::copy(&image_path, &new_cover_path).map_err(|e| format!("复制图片失败: {}", e))?;

    // 使用相对路径存储（相对于项目根目录）
    let relative_cover_path = format!("covers/{}", cover_filename);

    // 更新 project.json
    if project_json_path.exists() {
        let content = fs::read_to_string(&project_json_path)
            .map_err(|e| format!("读取项目配置失败: {}", e))?;

        let mut json: serde_json::Value =
            serde_json::from_str(&content).map_err(|e| format!("解析项目配置失败: {}", e))?;

        json["cover_path"] = serde_json::Value::String(relative_cover_path.clone());

        fs::write(
            &project_json_path,
            serde_json::to_string_pretty(&json)
                .map_err(|e| format!("序列化项目配置失败: {}", e))?,
        )
        .map_err(|e| format!("写入项目配置失败: {}", e))?;
    }

    // 返回完整路径供前端使用（确保使用正斜杠）
    let full_cover_path = new_cover_path
        .to_string_lossy()
        .to_string()
        .replace('\\', "/");
    Ok(full_cover_path)
}

/// 检查是否需要迁移
///
/// 查询数据库中没有 project_id 的项目数量，用于判断是否需要执行数据迁移。
///
/// # 参数
/// - `app_handle`: Tauri 应用句柄
///
/// # 返回值
/// 待迁移项目数量 `Ok(i32)`，失败返回错误信息 `Err(String)`
#[tauri::command]
pub async fn check_migration_needed(app_handle: AppHandle) -> Result<i32, String> {
    let db_path = get_db_path(&app_handle);
    if !db_path.exists() {
        return Ok(0);
    }
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;
    init_db(&conn).map_err(|e| format!("数据库初始化失败: {}", e))?;

    let count: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM projects WHERE project_id IS NULL OR project_id = ''",
            [],
            |row| row.get(0),
        )
        .map_err(|e| format!("查询失败: {}", e))?;
    Ok(count)
}

/// 执行数据库备份
///
/// 使用 SQLite 的 VACUUM INTO 命令创建数据库快照，返回备份文件路径。
/// 备份文件存储在数据库目录下的 backups 子目录中。
///
/// # 参数
/// - `db_path`: 数据库文件路径
///
/// # 返回值
/// 备份文件路径 `Ok(String)`，失败返回错误信息 `Err(String)`
fn backup_database(db_path: &PathBuf) -> Result<String, String> {
    let backup_dir = db_path
        .parent()
        .ok_or("无法获取数据库目录")?
        .join("backups");
    fs::create_dir_all(&backup_dir).map_err(|e| format!("创建备份目录失败: {}", e))?;

    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let backup_filename = format!("inovel_before_migration_{}.db", timestamp);
    let backup_path = backup_dir.join(&backup_filename);

    // 使用 VACUUM INTO 创建数据库快照（SQLite 3.27.0+）
    let conn = Connection::open(db_path).map_err(|e| format!("打开数据库失败: {}", e))?;
    conn.execute_batch(&format!(
        "VACUUM INTO '{}'",
        backup_path.to_string_lossy().replace('\'', "''")
    ))
    .map_err(|e| format!("数据库备份失败: {}", e))?;

    Ok(backup_path.to_string_lossy().to_string())
}

/// 记录迁移日志到 migration_logs 表
///
/// 记录每次迁移或回滚操作的详细信息，便于追踪和审计。
///
/// # 参数
/// - `conn`: SQLite 数据库连接
/// - `operation`: 操作类型（"migrate" 或 "rollback"）
/// - `project_db_id`: 项目数据库 ID
/// - `old_project_id`: 旧的项目 ID（迁移前）
/// - `new_project_id`: 新的项目 ID（迁移后）
/// - `old_path`: 旧的项目路径
/// - `new_path`: 新的项目路径
/// - `status`: 操作状态（"success", "failed", "warning"）
/// - `error_message`: 错误信息（如果失败）
fn log_migration(
    conn: &Connection,
    operation: &str,
    project_db_id: i64,
    old_project_id: &Option<String>,
    new_project_id: &str,
    old_path: &str,
    new_path: &str,
    status: &str,
    error_message: &str,
) {
    let now = chrono::Utc::now().to_rfc3339();
    let _ = conn.execute(
        "INSERT INTO migration_logs (operation, project_db_id, old_project_id, new_project_id, old_path, new_path, status, error_message, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        rusqlite::params![operation, project_db_id, old_project_id, new_project_id, old_path, new_path, status, error_message, now],
    );
}

/// 验证 Git 仓库在重命名后是否可访问
///
/// 检查重命名后的项目目录中的 Git 仓库是否仍然可用。
/// 如果没有 Git 仓库，视为验证通过。
///
/// # 参数
/// - `repo_path`: Git 仓库路径
///
/// # 返回值
/// 验证通过返回 `true`，失败返回 `false`
fn verify_git_repo(repo_path: &PathBuf) -> bool {
    let git_dir = repo_path.join(".git");
    if !git_dir.exists() {
        return true; // 没有 Git 仓库，视为正常
    }
    match git2::Repository::open(repo_path) {
        Ok(_) => true,
        Err(_) => false,
    }
}

/// 迁移现有项目：为没有 project_id 的项目生成字母数字ID
/// 并将文件夹名称从书名改为项目ID
///
/// # 参数
/// - `dry_run`: 预览模式。若为 true，只统计和预览，不做实际变更
///
/// # 返回值
/// 返回 MigrateResult，包含迁移详情和备份路径
#[tauri::command]
pub async fn migrate_existing_projects(
    app_handle: AppHandle,
    dry_run: Option<bool>,
) -> Result<MigrateResult, String> {
    let is_dry_run = dry_run.unwrap_or(false);
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;
    init_db(&conn).map_err(|e| format!("数据库初始化失败: {}", e))?;

    // 查询所有没有 project_id 的项目
    let mut stmt = conn
        .prepare("SELECT id, name, path FROM projects WHERE project_id IS NULL OR project_id = ''")
        .map_err(|e| format!("查询准备失败: {}", e))?;

    let projects_to_migrate: Vec<(i64, String, String)> = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
        .map_err(|e| format!("查询执行失败: {}", e))?
        .filter_map(|r| r.ok())
        .collect();

    let total = projects_to_migrate.len() as i32;
    if total == 0 {
        return Ok(MigrateResult {
            total: 0,
            success: 0,
            failed: 0,
            skipped: 0,
            backup_path: String::new(),
            details: vec![],
        });
    }

    // 如果是预览模式，只返回预览信息
    if is_dry_run {
        let details: Vec<MigrationDetail> = projects_to_migrate
            .iter()
            .map(|(id, name, old_path)| {
                let old_path_buf = PathBuf::from(old_path);
                let parent_dir = old_path_buf
                    .parent()
                    .map(|p| p.to_string_lossy().to_string());
                MigrationDetail {
                    project_db_id: *id,
                    old_name: name.clone(),
                    old_path: old_path.clone(),
                    new_path: parent_dir.unwrap_or_default(),
                    project_id: String::new(),
                    status: "pending".to_string(),
                    error: None,
                }
            })
            .collect();

        return Ok(MigrateResult {
            total,
            success: 0,
            failed: 0,
            skipped: 0,
            backup_path: "PREVIEW_MODE".to_string(),
            details,
        });
    }

    // --- 实际操作模式 ---

    // 1. 创建数据库备份
    let backup_path = backup_database(&db_path)?;

    // 2. 逐项目迁移
    let mut success_count = 0;
    let mut failed_count = 0;
    let mut details: Vec<MigrationDetail> = Vec::new();

    for (project_db_id, old_name, old_path_str) in &projects_to_migrate {
        let old_path_buf = PathBuf::from(old_path_str);
        let parent_dir = match old_path_buf.parent() {
            Some(p) => p.to_path_buf(),
            None => {
                failed_count += 1;
                details.push(MigrationDetail {
                    project_db_id: *project_db_id,
                    old_name: old_name.clone(),
                    old_path: old_path_str.clone(),
                    new_path: String::new(),
                    project_id: String::new(),
                    status: "failed".to_string(),
                    error: Some("无法获取父目录路径".to_string()),
                });
                continue;
            }
        };

        // 检查旧路径是否存在
        if !old_path_buf.exists() {
            failed_count += 1;
            details.push(MigrationDetail {
                project_db_id: *project_db_id,
                old_name: old_name.clone(),
                old_path: old_path_str.clone(),
                new_path: String::new(),
                project_id: String::new(),
                status: "failed".to_string(),
                error: Some("项目文件夹已不存在".to_string()),
            });
            continue;
        }

        // 生成唯一项目 ID
        let project_id = match generate_unique_project_id(&conn) {
            Ok(id) => id,
            Err(e) => {
                failed_count += 1;
                details.push(MigrationDetail {
                    project_db_id: *project_db_id,
                    old_name: old_name.clone(),
                    old_path: old_path_str.clone(),
                    new_path: String::new(),
                    project_id: String::new(),
                    status: "failed".to_string(),
                    error: Some(format!("生成项目ID失败: {}", e)),
                });
                continue;
            }
        };

        let new_path = parent_dir.join(&project_id);

        // 检查目标路径是否已存在，若冲突则重新生成ID（最多3次）
        let mut final_project_id = project_id;
        let mut final_new_path = new_path.clone();
        let mut retry_count = 0;
        while final_new_path.exists() && retry_count < 3 {
            retry_count += 1;
            match generate_unique_project_id(&conn) {
                Ok(new_id) => {
                    final_project_id = new_id;
                    final_new_path = parent_dir.join(&final_project_id);
                }
                Err(e) => {
                    failed_count += 1;
                    details.push(MigrationDetail {
                        project_db_id: *project_db_id,
                        old_name: old_name.clone(),
                        old_path: old_path_str.clone(),
                        new_path: String::new(),
                        project_id: String::new(),
                        status: "failed".to_string(),
                        error: Some(format!("重试生成ID失败: {}", e)),
                    });
                    break;
                }
            }
        }
        if final_new_path.exists() {
            failed_count += 1;
            details.push(MigrationDetail {
                project_db_id: *project_db_id,
                old_name: old_name.clone(),
                old_path: old_path_str.clone(),
                new_path: String::new(),
                project_id: String::new(),
                status: "failed".to_string(),
                error: Some("目标路径已存在，无法迁移".to_string()),
            });
            continue;
        }

        // 开始事务
        let tx_result = (|| -> Result<(), String> {
            conn.execute("BEGIN", [])
                .map_err(|e| format!("开始事务失败: {}", e))?;

            // 重命名文件夹
            fs::rename(&old_path_buf, &final_new_path)
                .map_err(|e| format!("重命名文件夹失败: {}", e))?;

            // 验证 Git 仓库完整性
            let git_ok = verify_git_repo(&final_new_path);
            if !git_ok {
                // Git 失败仅记录警告，不阻断迁移
                let _ = log_migration(
                    &conn,
                    "migrate",
                    *project_db_id,
                    &None,
                    &final_project_id,
                    old_path_str,
                    &final_new_path.to_string_lossy().to_string(),
                    "warning",
                    "Git仓库重命名后验证失败，但迁移继续",
                );
            }

            // 更新数据库
            let new_path_str = final_new_path.to_string_lossy().to_string();
            let rows_affected = conn
                .execute(
                    "UPDATE projects SET project_id = ?1, path = ?2 WHERE id = ?3",
                    rusqlite::params![&final_project_id, &new_path_str, project_db_id],
                )
                .map_err(|e| format!("更新数据库失败: {}", e))?;

            if rows_affected == 0 {
                return Err("更新数据库时未找到对应记录".to_string());
            }

            // 更新 project.json
            let project_json_path = final_new_path.join("project.json");
            if project_json_path.exists() {
                let content = fs::read_to_string(&project_json_path)
                    .map_err(|e| format!("读取项目配置失败: {}", e))?;
                let mut json: serde_json::Value = serde_json::from_str(&content)
                    .map_err(|e| format!("解析项目配置失败: {}", e))?;
                json["project_id"] = serde_json::Value::String(final_project_id.clone());
                let json_string = serde_json::to_string_pretty(&json)
                    .map_err(|e| format!("序列化项目配置失败: {}", e))?;
                fs::write(&project_json_path, json_string)
                    .map_err(|e| format!("写入项目配置失败: {}", e))?;
            }

            // 记录迁移日志
            log_migration(
                &conn,
                "migrate",
                *project_db_id,
                &None,
                &final_project_id,
                old_path_str,
                &new_path_str,
                "success",
                "",
            );

            conn.execute("COMMIT", [])
                .map_err(|e| format!("提交事务失败: {}", e))?;

            Ok(())
        })();

        match tx_result {
            Ok(()) => {
                success_count += 1;
                details.push(MigrationDetail {
                    project_db_id: *project_db_id,
                    old_name: old_name.clone(),
                    old_path: old_path_str.clone(),
                    new_path: final_new_path.to_string_lossy().to_string(),
                    project_id: final_project_id,
                    status: "success".to_string(),
                    error: None,
                });
            }
            Err(e) => {
                // 回滚事务
                let _ = conn.execute("ROLLBACK", []);
                failed_count += 1;
                details.push(MigrationDetail {
                    project_db_id: *project_db_id,
                    old_name: old_name.clone(),
                    old_path: old_path_str.clone(),
                    new_path: String::new(),
                    project_id: String::new(),
                    status: "failed".to_string(),
                    error: Some(e),
                });
            }
        }
    }

    Ok(MigrateResult {
        total,
        success: success_count,
        failed: failed_count,
        skipped: 0,
        backup_path,
        details,
    })
}

#[tauri::command]
pub async fn save_window_size(
    app_handle: AppHandle,
    project_id: i64,
    width: f64,
    height: f64,
) -> Result<(), String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;

    let project_path: String = conn
        .query_row(
            "SELECT path FROM projects WHERE id = ?1",
            [project_id],
            |row| row.get(0),
        )
        .map_err(|e| format!("查询项目路径失败: {}", e))?;

    let project_json_path = PathBuf::from(&project_path).join("project.json");

    if project_json_path.exists() {
        let content = fs::read_to_string(&project_json_path)
            .map_err(|e| format!("读取项目配置失败: {}", e))?;
        let mut json: serde_json::Value =
            serde_json::from_str(&content).map_err(|e| format!("解析项目配置失败: {}", e))?;

        json["window_width"] = serde_json::Value::Number(
            serde_json::Number::from_f64(width).unwrap_or(serde_json::Number::from(1200)),
        );
        json["window_height"] = serde_json::Value::Number(
            serde_json::Number::from_f64(height).unwrap_or(serde_json::Number::from(800)),
        );

        fs::write(
            &project_json_path,
            serde_json::to_string_pretty(&json)
                .map_err(|e| format!("序列化项目配置失败: {}", e))?,
        )
        .map_err(|e| format!("写入项目配置失败: {}", e))?;
    } else {
        return Err("项目配置文件不存在".to_string());
    }

    Ok(())
}

#[tauri::command]
pub async fn set_window_size(
    app: tauri::AppHandle,
    project_id: i64,
    width: f64,
    height: f64,
) -> Result<(), String> {
    let db_path = get_db_path(&app);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;

    let project_path: String = conn
        .query_row(
            "SELECT path FROM projects WHERE id = ?1",
            [project_id],
            |row| row.get(0),
        )
        .map_err(|e| format!("查询项目路径失败: {}", e))?;

    let project_json_path = PathBuf::from(&project_path).join("project.json");

    if project_json_path.exists() {
        let content = fs::read_to_string(&project_json_path)
            .map_err(|e| format!("读取项目配置失败: {}", e))?;
        let mut json: serde_json::Value =
            serde_json::from_str(&content).map_err(|e| format!("解析项目配置失败: {}", e))?;

        json["window_width"] = serde_json::Value::Number(
            serde_json::Number::from_f64(width).unwrap_or(serde_json::Number::from(1200)),
        );
        json["window_height"] = serde_json::Value::Number(
            serde_json::Number::from_f64(height).unwrap_or(serde_json::Number::from(800)),
        );

        fs::write(
            &project_json_path,
            serde_json::to_string_pretty(&json)
                .map_err(|e| format!("序列化项目配置失败: {}", e))?,
        )
        .map_err(|e| format!("写入项目配置失败: {}", e))?;
    } else {
        return Err("项目配置文件不存在".to_string());
    }

    if let Some(window) = app.get_webview_window("main") {
        window
            .set_size(tauri::Size::Physical(tauri::PhysicalSize {
                width: width as u32,
                height: height as u32,
            }))
            .map_err(|e| format!("设置窗口大小失败: {}", e))?;
    }

    Ok(())
}

#[tauri::command]
pub async fn get_window_size(
    app_handle: AppHandle,
    project_id: i64,
) -> Result<Option<(f64, f64)>, String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;

    let project_path: String = conn
        .query_row(
            "SELECT path FROM projects WHERE id = ?1",
            [project_id],
            |row| row.get(0),
        )
        .map_err(|e| format!("查询项目路径失败: {}", e))?;

    let project_json_path = PathBuf::from(&project_path).join("project.json");

    if project_json_path.exists() {
        let content = fs::read_to_string(&project_json_path)
            .map_err(|e| format!("读取项目配置失败: {}", e))?;
        let json: serde_json::Value =
            serde_json::from_str(&content).map_err(|e| format!("解析项目配置失败: {}", e))?;

        if let (Some(width), Some(height)) = (
            json.get("window_width").and_then(|v| v.as_f64()),
            json.get("window_height").and_then(|v| v.as_f64()),
        ) {
            return Ok(Some((width, height)));
        }
    }

    Ok(None)
}

/// 回滚迁移：将已迁移的项目恢复为旧路径
/// 如果 params.project_ids 为空（None），则回滚所有已迁移的项目
#[tauri::command]
pub async fn rollback_migration(
    app_handle: AppHandle,
    params: Option<RollbackParams>,
) -> Result<MigrateResult, String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;
    init_db(&conn).map_err(|e| format!("数据库初始化失败: {}", e))?;

    // 查询已迁移的记录
    let rollback_targets: Vec<(i64, String, String)> = if let Some(ref p) = params {
        if let Some(ref ids) = p.project_ids {
            // 回滚指定项目
            let placeholders: Vec<String> = ids.iter().map(|_| "?".to_string()).collect();
            let sql = format!(
                "SELECT DISTINCT project_db_id, old_path, new_path FROM migration_logs
                 WHERE operation = 'migrate' AND status = 'success'
                 AND project_db_id IN ({})
                 AND project_db_id NOT IN (
                     SELECT project_db_id FROM migration_logs
                     WHERE operation = 'rollback' AND status = 'success'
                 )",
                placeholders.join(",")
            );
            let mut stmt = conn
                .prepare(&sql)
                .map_err(|e| format!("查询准备失败: {}", e))?;
            let params_refs: Vec<&dyn rusqlite::types::ToSql> = ids
                .iter()
                .map(|id| id as &dyn rusqlite::types::ToSql)
                .collect();
            stmt.query_map(params_refs.as_slice(), |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?))
            })
            .map_err(|e| format!("查询执行失败: {}", e))?
            .filter_map(|r| r.ok())
            .collect()
        } else {
            vec![]
        }
    } else {
        // 回滚全部已迁移但未回滚的项目
        let mut stmt = conn
            .prepare(
                "SELECT DISTINCT ml.project_db_id, ml.old_path, ml.new_path FROM migration_logs ml
                 WHERE ml.operation = 'migrate' AND ml.status = 'success'
                 AND ml.project_db_id NOT IN (
                     SELECT project_db_id FROM migration_logs
                     WHERE operation = 'rollback' AND status = 'success'
                 )",
            )
            .map_err(|e| format!("查询准备失败: {}", e))?;
        stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
            .map_err(|e| format!("查询执行失败: {}", e))?
            .filter_map(|r| r.ok())
            .collect()
    };

    let total = rollback_targets.len() as i32;
    if total == 0 {
        return Ok(MigrateResult {
            total: 0,
            success: 0,
            failed: 0,
            skipped: 0,
            backup_path: String::new(),
            details: vec![],
        });
    }

    // 创建回滚前备份
    let backup_path = backup_database(&db_path)?;

    let mut success_count = 0;
    let mut failed_count = 0;
    let mut details: Vec<MigrationDetail> = Vec::new();

    for (project_db_id, old_path, new_path) in &rollback_targets {
        let old_path_buf = PathBuf::from(old_path);
        let new_path_buf = PathBuf::from(new_path);

        let tx_result = (|| -> Result<(), String> {
            conn.execute("BEGIN", [])
                .map_err(|e| format!("开始事务失败: {}", e))?;

            // 重命名回来
            if new_path_buf.exists() {
                fs::rename(&new_path_buf, &old_path_buf)
                    .map_err(|e| format!("回滚重命名失败: {}", e))?;
            } else {
                return Err("新路径已不存在，无法回滚".to_string());
            }

            // 更新数据库：清空 project_id，恢复 path
            let rows_affected = conn
                .execute(
                    "UPDATE projects SET project_id = NULL, path = ?1 WHERE id = ?2",
                    rusqlite::params![old_path, project_db_id],
                )
                .map_err(|e| format!("更新数据库失败: {}", e))?;

            if rows_affected == 0 {
                return Err("更新数据库时未找到对应记录".to_string());
            }

            // 恢复 project.json（移除 project_id 字段）
            let project_json_path = old_path_buf.join("project.json");
            if project_json_path.exists() {
                let content = fs::read_to_string(&project_json_path)
                    .map_err(|e| format!("读取项目配置失败: {}", e))?;
                let mut json: serde_json::Value = serde_json::from_str(&content)
                    .map_err(|e| format!("解析项目配置失败: {}", e))?;
                // 移除 project_id 字段
                if let Some(obj) = json.as_object_mut() {
                    obj.remove("project_id");
                }
                let json_string = serde_json::to_string_pretty(&json)
                    .map_err(|e| format!("序列化项目配置失败: {}", e))?;
                fs::write(&project_json_path, json_string)
                    .map_err(|e| format!("写入项目配置失败: {}", e))?;
            }

            // 记录回滚日志
            log_migration(
                &conn,
                "rollback",
                *project_db_id,
                &None,
                "",
                old_path,
                new_path,
                "success",
                "",
            );

            conn.execute("COMMIT", [])
                .map_err(|e| format!("提交事务失败: {}", e))?;

            Ok(())
        })();

        match tx_result {
            Ok(()) => {
                success_count += 1;
                details.push(MigrationDetail {
                    project_db_id: *project_db_id,
                    old_name: String::new(),
                    old_path: old_path.clone(),
                    new_path: new_path.clone(),
                    project_id: String::new(),
                    status: "success".to_string(),
                    error: None,
                });
            }
            Err(e) => {
                let _ = conn.execute("ROLLBACK", []);
                failed_count += 1;
                details.push(MigrationDetail {
                    project_db_id: *project_db_id,
                    old_name: String::new(),
                    old_path: old_path.clone(),
                    new_path: new_path.clone(),
                    project_id: String::new(),
                    status: "failed".to_string(),
                    error: Some(e),
                });
            }
        }
    }

    Ok(MigrateResult {
        total,
        success: success_count,
        failed: failed_count,
        skipped: 0,
        backup_path,
        details,
    })
}
