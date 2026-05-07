use crate::db::{get_db_path, init_db};
use crate::models::{Chapter, ChapterStatusCount, Volume, VolumeWithChapters};
use base64::{Engine as _, engine::general_purpose};
use rusqlite::Connection;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::AppHandle;

/// 保存图片到项目 media 目录
///
/// 将前端传来的 Base64 编码的图片数据解码并保存到项目的 media 目录下。
/// 文件名会添加时间戳和随机数后缀以避免冲突。
///
/// # 参数
/// - `app_handle`: Tauri 应用句柄
/// - `project_id`: 项目 ID
/// - `file_name`: 原始文件名
/// - `file_data`: Base64 编码的文件数据
///
/// # 返回值
/// 成功返回相对路径（如 "media/1234567890_12345.png"），失败返回错误信息
#[tauri::command]
pub async fn save_image(
    app_handle: AppHandle,
    project_id: i64,
    file_name: String,
    file_data: String, // base64 编码的文件数据
) -> Result<String, String> {
    // 解码 base64 数据
    let data = general_purpose::STANDARD
        .decode(&file_data)
        .map_err(|e| format!("解码文件数据失败: {}", e))?;

    // 获取项目路径
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;
    let (storage_path, project_name): (String, String) = conn
        .query_row(
            "SELECT path, name FROM projects WHERE id = ?1",
            [project_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|e| format!("项目不存在: {}", e))?;

    // 创建 media 目录
    let media_dir = Path::new(&storage_path).join(&project_name).join("media");
    fs::create_dir_all(&media_dir).map_err(|e| format!("创建 media 目录失败: {}", e))?;

    // 生成唯一的文件名
    let timestamp = chrono::Utc::now().timestamp();
    let extension = file_name.split('.').last().unwrap_or("png");
    let random_num: u32 = rand::random();
    let new_file_name = format!("{}_{}.{}", timestamp, random_num, extension);
    let new_path = media_dir.join(&new_file_name);

    // 写入文件
    fs::write(&new_path, data).map_err(|e| format!("写入文件失败: {}", e))?;

    // 返回相对路径
    let relative_path = format!("media/{}", new_file_name);
    Ok(relative_path)
}

/// 创建新卷
///
/// 在指定项目中创建一个新的卷（Volume），用于组织章节。
/// 卷会按 sort_order 字段排序，新卷默认添加到末尾。
///
/// # 参数
/// - `app_handle`: Tauri 应用句柄
/// - `project_id`: 项目 ID
/// - `name`: 卷名称
///
/// # 返回值
/// 成功返回创建的卷信息，失败返回错误信息
#[tauri::command]
pub async fn create_volume(
    app_handle: AppHandle,
    project_id: i64,
    name: String,
) -> Result<Volume, String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;
    init_db(&conn).map_err(|e| format!("数据库初始化失败: {}", e))?;

    let max_order: Option<i32> = conn
        .query_row(
            "SELECT MAX(sort_order) FROM volumes WHERE project_id = ?1",
            [project_id],
            |row| row.get(0),
        )
        .ok();
    let sort_order = max_order.unwrap_or(-1) + 1;

    conn.execute(
        "INSERT INTO volumes (project_id, name, sort_order) VALUES (?1, ?2, ?3)",
        (project_id, &name, sort_order),
    )
    .map_err(|e| format!("创建卷失败: {}", e))?;
    let id = conn.last_insert_rowid();
    Ok(Volume {
        id,
        project_id,
        name,
        sort_order,
    })
}

/// 创建新章节
///
/// 在指定卷中创建一个新的章节，并创建对应的 Markdown 文件。
/// 章节文件保存在 `项目名/chapters/v{volume_id}_c{chapter_id}.md`。
///
/// # 参数
/// - `app_handle`: Tauri 应用句柄
/// - `project_id`: 项目 ID
/// - `volume_id`: 所属卷的 ID
/// - `title`: 章节标题
/// - `initial_content`: 初始内容（可选）
///
/// # 返回值
/// 成功返回创建的章节信息，失败返回错误信息
#[tauri::command]
pub async fn create_chapter(
    app_handle: AppHandle,
    project_id: i64,
    volume_id: i64,
    title: String,
    initial_content: Option<String>,
) -> Result<Chapter, String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;
    init_db(&conn).map_err(|e| format!("数据库初始化失败: {}", e))?;

    let max_order: Option<i32> = conn
        .query_row(
            "SELECT MAX(sort_order) FROM chapters WHERE volume_id = ?1",
            [volume_id],
            |row| row.get(0),
        )
        .ok();
    let sort_order = max_order.unwrap_or(-1) + 1;
    let now = chrono::Utc::now().to_rfc3339();

    let (storage_path, project_name): (String, String) = conn
        .query_row(
            "SELECT path, name FROM projects WHERE id = ?1",
            [project_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|e| format!("项目不存在: {}", e))?;

    conn.execute(
        "INSERT INTO chapters (volume_id, title, file_path, sort_order, summary, word_count_cache, status, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        (&volume_id, &title, "", sort_order, "", 0, "draft", &now, &now),
    ).map_err(|e| format!("创建章节失败: {}", e))?;
    let id = conn.last_insert_rowid();

    let actual_file_path = Path::new(&storage_path)
        .join(&project_name)
        .join("chapters")
        .join(format!("v{}_c{}.md", volume_id, id));
    fs::create_dir_all(actual_file_path.parent().unwrap())
        .map_err(|e| format!("创建目录失败: {}", e))?;

    // 如果有初始内容，写入文件；否则写入空字符串
    let content = initial_content.unwrap_or_default();
    fs::write(&actual_file_path, content).map_err(|e| format!("写入章节文件失败: {}", e))?;

    conn.execute(
        "UPDATE chapters SET file_path = ?1 WHERE id = ?2",
        (actual_file_path.to_string_lossy().to_string(), id),
    )
    .map_err(|e| format!("更新文件路径失败: {}", e))?;

    Ok(Chapter {
        id,
        volume_id,
        title,
        file_path: actual_file_path.to_string_lossy().to_string(),
        sort_order,
        summary: String::new(),
        word_count_cache: 0,
        status: "draft".to_string(),
        created_at: now.clone(),
        updated_at: now,
    })
}

/// 更新卷名称
///
/// # 参数
/// - `app_handle`: Tauri 应用句柄
/// - `volume_id`: 卷 ID
/// - `new_name`: 新的卷名称
///
/// # 返回值
/// 成功返回 Ok(()), 失败返回错误信息
#[tauri::command]
pub async fn update_volume_name(
    app_handle: AppHandle,
    volume_id: i64,
    new_name: String,
) -> Result<(), String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;
    conn.execute(
        "UPDATE volumes SET name = ?1 WHERE id = ?2",
        (&new_name, volume_id),
    )
    .map_err(|e| format!("更新卷名失败: {}", e))?;
    Ok(())
}

/// 更新章节标题
///
/// # 参数
/// - `app_handle`: Tauri 应用句柄
/// - `chapter_id`: 章节 ID
/// - `new_title`: 新的标题
///
/// # 返回值
/// 成功返回 Ok(()), 失败返回错误信息
#[tauri::command]
pub async fn update_chapter_title(
    app_handle: AppHandle,
    chapter_id: i64,
    new_title: String,
) -> Result<(), String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE chapters SET title = ?1, updated_at = ?2 WHERE id = ?3",
        (&new_title, &now, chapter_id),
    )
    .map_err(|e| format!("更新章节标题失败: {}", e))?;
    Ok(())
}

/// 更新章节字数缓存
///
/// 将计算后的章节字数保存到数据库，避免每次都重新计算。
///
/// # 参数
/// - `app_handle`: Tauri 应用句柄
/// - `chapter_id`: 章节 ID
/// - `word_count`: 字数
///
/// # 返回值
/// 成功返回 Ok(()), 失败返回错误信息
#[tauri::command]
pub async fn update_chapter_word_count(
    app_handle: AppHandle,
    chapter_id: i64,
    word_count: i32,
) -> Result<(), String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;
    conn.execute(
        "UPDATE chapters SET word_count_cache = ?1 WHERE id = ?2",
        (word_count, chapter_id),
    )
    .map_err(|e| format!("更新章节字数失败: {}", e))?;
    Ok(())
}

/// 更新章节摘要
///
/// # 参数
/// - `app_handle`: Tauri 应用句柄
/// - `chapter_id`: 章节 ID
/// - `new_summary`: 新的摘要
///
/// # 返回值
/// 成功返回 Ok(()), 失败返回错误信息
#[tauri::command]
pub async fn update_chapter_summary(
    app_handle: AppHandle,
    chapter_id: i64,
    new_summary: String,
) -> Result<(), String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;
    conn.execute(
        "UPDATE chapters SET summary = ?1 WHERE id = ?2",
        (&new_summary, chapter_id),
    )
    .map_err(|e| format!("更新章节摘要失败: {}", e))?;
    Ok(())
}

/// 删除卷及其所有章节
///
/// 删除卷时会同时删除该卷下的所有章节记录，但不会删除章节文件。
///
/// # 参数
/// - `app_handle`: Tauri 应用句柄
/// - `volume_id`: 卷 ID
///
/// # 返回值
/// 成功返回 Ok(()), 失败返回错误信息
#[tauri::command]
pub async fn delete_volume(app_handle: AppHandle, volume_id: i64) -> Result<(), String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;
    conn.execute("DELETE FROM chapters WHERE volume_id = ?1", [volume_id])
        .ok();
    conn.execute("DELETE FROM volumes WHERE id = ?1", [volume_id])
        .map_err(|e| format!("删除卷失败: {}", e))?;
    Ok(())
}

/// 删除章节
///
/// # 参数
/// - `chapter_id`: 章节数据库 ID
/// - `keep_file`: 是否保留本地文件（默认 false，删除文件）
#[tauri::command]
pub async fn delete_chapter(
    app_handle: AppHandle,
    chapter_id: i64,
    keep_file: Option<bool>,
) -> Result<(), String> {
    let keep_file = keep_file.unwrap_or(false);
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;

    // 获取文件路径（仅在不保留文件时需要）
    if !keep_file {
        let file_path: Option<String> = conn
            .query_row(
                "SELECT file_path FROM chapters WHERE id = ?1",
                [chapter_id],
                |row| row.get(0),
            )
            .ok();
        if let Some(p) = file_path {
            let path = PathBuf::from(&p);
            if path.exists() {
                fs::remove_file(&path).map_err(|e| format!("删除章节文件失败: {}", e))?;
            }
        }
    }

    conn.execute("DELETE FROM chapters WHERE id = ?1", [chapter_id])
        .map_err(|e| format!("删除章节记录失败: {}", e))?;
    Ok(())
}

/// 批量更新卷的排序顺序
///
/// 根据提供的 ID 列表顺序更新各卷的 sort_order 字段。
///
/// # 参数
/// - `app_handle`: Tauri 应用句柄
/// - `project_id`: 项目 ID（用于验证卷属于该项目）
/// - `ordered_ids`: 卷 ID 列表，按新顺序排列
///
/// # 返回值
/// 成功返回 Ok(()), 失败返回错误信息
#[tauri::command]
pub async fn reorder_volumes(
    app_handle: AppHandle,
    project_id: i64,
    ordered_ids: Vec<i64>,
) -> Result<(), String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;
    for (i, vid) in ordered_ids.iter().enumerate() {
        conn.execute(
            "UPDATE volumes SET sort_order = ?1 WHERE id = ?2 AND project_id = ?3",
            (i as i32, vid, project_id),
        )
        .map_err(|e| format!("更新排序失败: {}", e))?;
    }
    Ok(())
}

/// 批量更新章节的排序顺序
///
/// 根据提供的 ID 列表顺序更新各章节的 sort_order 字段。
///
/// # 参数
/// - `app_handle`: Tauri 应用句柄
/// - `volume_id`: 卷 ID（用于验证章节属于该卷）
/// - `ordered_ids`: 章节 ID 列表，按新顺序排列
///
/// # 返回值
/// 成功返回 Ok(()), 失败返回错误信息
#[tauri::command]
pub async fn reorder_chapters(
    app_handle: AppHandle,
    volume_id: i64,
    ordered_ids: Vec<i64>,
) -> Result<(), String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;
    for (i, cid) in ordered_ids.iter().enumerate() {
        conn.execute(
            "UPDATE chapters SET sort_order = ?1 WHERE id = ?2 AND volume_id = ?3",
            (i as i32, cid, volume_id),
        )
        .map_err(|e| format!("更新排序失败: {}", e))?;
    }
    Ok(())
}

/// 获取项目的章节树结构
///
/// 查询指定项目下所有卷及其包含的章节，返回树形结构。
/// 卷和章节分别按 sort_order 字段排序。
///
/// # 参数
/// - `app_handle`: Tauri 应用句柄
/// - `project_id`: 项目 ID
///
/// # 返回值
/// 成功返回卷及其章节的树形结构列表，失败返回错误信息
#[tauri::command]
pub async fn get_chapter_tree(
    app_handle: AppHandle,
    project_id: i64,
) -> Result<Vec<VolumeWithChapters>, String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;
    init_db(&conn).map_err(|e| format!("数据库初始化失败: {}", e))?;

    let mut stmt = conn.prepare("SELECT id, project_id, name, sort_order FROM volumes WHERE project_id = ?1 ORDER BY sort_order")
        .map_err(|e| format!("查询失败: {}", e))?;
    let volumes: Vec<Volume> = stmt
        .query_map([project_id], |row| {
            Ok(Volume {
                id: row.get(0)?,
                project_id: row.get(1)?,
                name: row.get(2)?,
                sort_order: row.get(3)?,
            })
        })
        .map_err(|e| format!("查询失败: {}", e))?
        .filter_map(|r| r.ok())
        .collect();

    let mut result = Vec::new();
    for v in volumes {
        let mut cs = conn.prepare("SELECT id, volume_id, title, file_path, sort_order, summary, word_count_cache, status, created_at, updated_at FROM chapters WHERE volume_id = ?1 ORDER BY sort_order")
            .map_err(|e| format!("查询失败: {}", e))?;
        let chapters: Vec<Chapter> = cs
            .query_map([v.id], |row| {
                Ok(Chapter {
                    id: row.get(0)?,
                    volume_id: row.get(1)?,
                    title: row.get(2)?,
                    file_path: row.get(3)?,
                    sort_order: row.get(4)?,
                    summary: row.get(5)?,
                    word_count_cache: row.get(6)?,
                    status: row.get(7)?,
                    created_at: row.get(8)?,
                    updated_at: row.get(9)?,
                })
            })
            .map_err(|e| format!("查询失败: {}", e))?
            .filter_map(|r| r.ok())
            .collect();
        result.push(VolumeWithChapters {
            id: v.id,
            project_id: v.project_id,
            name: v.name,
            sort_order: v.sort_order,
            chapters,
        });
    }
    Ok(result)
}

/// 获取章节内容
///
/// 从文件系统读取章节的 Markdown 文件内容。
/// 支持字符串或数字类型的 chapter_id。
///
/// # 参数
/// - `app_handle`: Tauri 应用句柄
/// - `_project_id`: 项目 ID（当前未使用）
/// - `chapter_id`: 章节 ID（支持字符串或数字）
///
/// # 返回值
/// 成功返回章节内容，失败返回错误信息
#[tauri::command]
pub async fn get_chapter_content(
    app_handle: AppHandle,
    _project_id: String,
    chapter_id: serde_json::Value,
) -> Result<String, String> {
    // 支持字符串或数字类型的 chapter_id
    let chapter_id: i64 = match chapter_id {
        serde_json::Value::String(s) => s
            .parse()
            .map_err(|_| "chapter_id 必须是有效的整数".to_string())?,
        serde_json::Value::Number(n) => n.as_i64().ok_or("chapter_id 超出有效范围")?,
        _ => return Err("chapter_id 必须是字符串或数字".to_string()),
    };

    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;

    // 从数据库获取章节的文件路径
    let file_path: Option<String> = conn
        .query_row(
            "SELECT file_path FROM chapters WHERE id = ?1",
            [chapter_id],
            |row| row.get(0),
        )
        .ok();

    if let Some(path) = file_path {
        let full_path = PathBuf::from(&path);
        if full_path.exists() {
            return fs::read_to_string(&full_path).map_err(|e| format!("读取章节失败: {}", e));
        }
    }

    Ok(String::new())
}

/// 保存章节内容
///
/// 将章节内容写入文件系统中的 Markdown 文件。
/// 支持字符串或数字类型的 chapter_id。
///
/// # 参数
/// - `_app_handle`: Tauri 应用句柄（当前未使用）
/// - `_project_id`: 项目 ID（当前未使用）
/// - `chapter_id`: 章节 ID（支持字符串或数字）
/// - `content`: 章节内容
///
/// # 返回值
/// 成功返回 Ok(()), 失败返回错误信息
#[tauri::command]
pub async fn save_chapter_content(
    _app_handle: AppHandle,
    _project_id: String,
    chapter_id: serde_json::Value,
    content: String,
) -> Result<(), String> {
    // 支持字符串或数字类型的 chapter_id
    let chapter_id: i64 = match chapter_id {
        serde_json::Value::String(s) => s
            .parse()
            .map_err(|_| "chapter_id 必须是有效的整数".to_string())?,
        serde_json::Value::Number(n) => n.as_i64().ok_or("chapter_id 超出有效范围")?,
        _ => return Err("chapter_id 必须是字符串或数字".to_string()),
    };

    let db_path = get_db_path(&_app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;

    // 从数据库获取章节的文件路径
    let file_path: Option<String> = conn
        .query_row(
            "SELECT file_path FROM chapters WHERE id = ?1",
            [chapter_id],
            |row| row.get(0),
        )
        .ok();

    let chapter_path = if let Some(path) = file_path {
        PathBuf::from(path)
    } else {
        return Err(format!("无法找到章节 {} 的文件路径", chapter_id));
    };

    // 确保目录存在
    if let Some(parent) = chapter_path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {}", e))?;
    }

    fs::write(&chapter_path, content).map_err(|e| format!("保存章节失败: {}", e))?;
    Ok(())
}

/// 更新章节状态
///
/// 章节状态用于跟踪写作进度，有效值包括：
/// - `outline`: 大纲
/// - `draft`: 初稿
/// - `revised`: 修订稿
/// - `final`: 定稿
/// - `abandoned`: 已废弃
///
/// # 参数
/// - `app_handle`: Tauri 应用句柄
/// - `chapter_id`: 章节 ID
/// - `status`: 新状态
///
/// # 返回值
/// 成功返回 Ok(()), 失败返回错误信息
#[tauri::command]
pub async fn update_chapter_status(
    app_handle: AppHandle,
    chapter_id: i64,
    status: String,
) -> Result<(), String> {
    // 验证 status 值
    let valid_statuses = ["outline", "draft", "revised", "final", "abandoned"];
    if !valid_statuses.contains(&status.as_str()) {
        return Err(format!(
            "无效的章节状态: {}，有效值为: {:?}",
            status, valid_statuses
        ));
    }

    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE chapters SET status = ?1, updated_at = ?2 WHERE id = ?3",
        (&status, &now, chapter_id),
    )
    .map_err(|e| format!("更新章节状态失败: {}", e))?;
    Ok(())
}

/// 获取项目中各状态的章节数量统计
///
/// 统计指定项目下每个状态的章节数量，返回状态-数量的列表。
///
/// # 参数
/// - `app_handle`: Tauri 应用句柄
/// - `project_id`: 项目 ID
///
/// # 返回值
/// 成功返回状态统计列表，失败返回错误信息
#[tauri::command]
pub async fn get_chapter_status_counts(
    app_handle: AppHandle,
    project_id: i64,
) -> Result<Vec<ChapterStatusCount>, String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;
    init_db(&conn).map_err(|e| format!("数据库初始化失败: {}", e))?;

    // 查询各状态的章节数量
    let mut stmt = conn
        .prepare(
            "SELECT c.status, COUNT(*) as count
         FROM chapters c
         JOIN volumes v ON c.volume_id = v.id
         WHERE v.project_id = ?1
         GROUP BY c.status",
        )
        .map_err(|e| format!("查询失败: {}", e))?;

    let counts: Vec<ChapterStatusCount> = stmt
        .query_map([project_id], |row| {
            Ok(ChapterStatusCount {
                status: row.get(0)?,
                count: row.get(1)?,
            })
        })
        .map_err(|e| format!("查询失败: {}", e))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(counts)
}
