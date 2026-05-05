use crate::db::{get_db_path, init_db};
use crate::models::{Chapter, Volume, VolumeWithChapters};
use rusqlite::Connection;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::AppHandle;

#[tauri::command]
pub async fn create_volume(
    app_handle: AppHandle, project_id: i64, name: String,
) -> Result<Volume, String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;
    init_db(&conn).map_err(|e| format!("数据库初始化失败: {}", e))?;

    let max_order: Option<i32> = conn
        .query_row("SELECT MAX(sort_order) FROM volumes WHERE project_id = ?1", [project_id], |row| row.get(0)).ok();
    let sort_order = max_order.unwrap_or(-1) + 1;

    conn.execute("INSERT INTO volumes (project_id, name, sort_order) VALUES (?1, ?2, ?3)",
        (project_id, &name, sort_order)).map_err(|e| format!("创建卷失败: {}", e))?;
    let id = conn.last_insert_rowid();
    Ok(Volume { id, project_id, name, sort_order })
}

#[tauri::command]
pub async fn create_chapter(
    app_handle: AppHandle, project_id: i64, volume_id: i64, title: String,
) -> Result<Chapter, String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;
    init_db(&conn).map_err(|e| format!("数据库初始化失败: {}", e))?;

    let max_order: Option<i32> = conn
        .query_row("SELECT MAX(sort_order) FROM chapters WHERE volume_id = ?1", [volume_id], |row| row.get(0)).ok();
    let sort_order = max_order.unwrap_or(-1) + 1;
    let now = chrono::Utc::now().to_rfc3339();

    let (storage_path, project_name): (String, String) = conn
        .query_row("SELECT path, name FROM projects WHERE id = ?1", [project_id], |row| Ok((row.get(0)?, row.get(1)?)))
        .map_err(|e| format!("项目不存在: {}", e))?;

    conn.execute(
        "INSERT INTO chapters (volume_id, title, file_path, sort_order, summary, word_count_cache, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        (&volume_id, &title, "", sort_order, "", 0, &now, &now),
    ).map_err(|e| format!("创建章节失败: {}", e))?;
    let id = conn.last_insert_rowid();

    let actual_file_path = Path::new(&storage_path).join(&project_name)
        .join("chapters").join(format!("v{}_c{}.md", volume_id, id));
    fs::create_dir_all(actual_file_path.parent().unwrap()).map_err(|e| format!("创建目录失败: {}", e))?;
    fs::write(&actual_file_path, "").ok();

    conn.execute("UPDATE chapters SET file_path = ?1 WHERE id = ?2",
        (actual_file_path.to_string_lossy().to_string(), id)).map_err(|e| format!("更新文件路径失败: {}", e))?;

    Ok(Chapter {
        id, volume_id, title, file_path: actual_file_path.to_string_lossy().to_string(),
        sort_order, summary: String::new(), word_count_cache: 0, created_at: now.clone(), updated_at: now,
    })
}

#[tauri::command]
pub async fn update_volume_name(app_handle: AppHandle, volume_id: i64, new_name: String) -> Result<(), String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;
    conn.execute("UPDATE volumes SET name = ?1 WHERE id = ?2", (&new_name, volume_id))
        .map_err(|e| format!("更新卷名失败: {}", e))?;
    Ok(())
}

#[tauri::command]
pub async fn update_chapter_title(app_handle: AppHandle, chapter_id: i64, new_title: String) -> Result<(), String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute("UPDATE chapters SET title = ?1, updated_at = ?2 WHERE id = ?3", (&new_title, &now, chapter_id))
        .map_err(|e| format!("更新章节标题失败: {}", e))?;
    Ok(())
}

#[tauri::command]
pub async fn update_chapter_word_count(app_handle: AppHandle, chapter_id: i64, word_count: i32) -> Result<(), String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;
    conn.execute("UPDATE chapters SET word_count_cache = ?1 WHERE id = ?2", (word_count, chapter_id))
        .map_err(|e| format!("更新章节字数失败: {}", e))?;
    Ok(())
}

#[tauri::command]
pub async fn update_chapter_summary(app_handle: AppHandle, chapter_id: i64, new_summary: String) -> Result<(), String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;
    conn.execute("UPDATE chapters SET summary = ?1 WHERE id = ?2", (&new_summary, chapter_id))
        .map_err(|e| format!("更新章节摘要失败: {}", e))?;
    Ok(())
}

#[tauri::command]
pub async fn delete_volume(app_handle: AppHandle, volume_id: i64) -> Result<(), String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;
    conn.execute("DELETE FROM chapters WHERE volume_id = ?1", [volume_id]).ok();
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
            .query_row("SELECT file_path FROM chapters WHERE id = ?1", [chapter_id], |row| row.get(0))
            .ok();
        if let Some(p) = file_path {
            let path = PathBuf::from(&p);
            if path.exists() {
                fs::remove_file(&path)
                    .map_err(|e| format!("删除章节文件失败: {}", e))?;
            }
        }
    }

    conn.execute("DELETE FROM chapters WHERE id = ?1", [chapter_id])
        .map_err(|e| format!("删除章节记录失败: {}", e))?;
    Ok(())
}

#[tauri::command]
pub async fn reorder_volumes(app_handle: AppHandle, project_id: i64, ordered_ids: Vec<i64>) -> Result<(), String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;
    for (i, vid) in ordered_ids.iter().enumerate() {
        conn.execute("UPDATE volumes SET sort_order = ?1 WHERE id = ?2 AND project_id = ?3",
            (i as i32, vid, project_id)).map_err(|e| format!("更新排序失败: {}", e))?;
    }
    Ok(())
}

#[tauri::command]
pub async fn reorder_chapters(app_handle: AppHandle, volume_id: i64, ordered_ids: Vec<i64>) -> Result<(), String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;
    for (i, cid) in ordered_ids.iter().enumerate() {
        conn.execute("UPDATE chapters SET sort_order = ?1 WHERE id = ?2 AND volume_id = ?3",
            (i as i32, cid, volume_id)).map_err(|e| format!("更新排序失败: {}", e))?;
    }
    Ok(())
}

#[tauri::command]
pub async fn get_chapter_tree(app_handle: AppHandle, project_id: i64) -> Result<Vec<VolumeWithChapters>, String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;
    init_db(&conn).map_err(|e| format!("数据库初始化失败: {}", e))?;

    let mut stmt = conn.prepare("SELECT id, project_id, name, sort_order FROM volumes WHERE project_id = ?1 ORDER BY sort_order")
        .map_err(|e| format!("查询失败: {}", e))?;
    let volumes: Vec<Volume> = stmt.query_map([project_id], |row| {
        Ok(Volume { id: row.get(0)?, project_id: row.get(1)?, name: row.get(2)?, sort_order: row.get(3)? })
    }).map_err(|e| format!("查询失败: {}", e))?.filter_map(|r| r.ok()).collect();

    let mut result = Vec::new();
    for v in volumes {
        let mut cs = conn.prepare("SELECT id, volume_id, title, file_path, sort_order, summary, word_count_cache, created_at, updated_at FROM chapters WHERE volume_id = ?1 ORDER BY sort_order")
            .map_err(|e| format!("查询失败: {}", e))?;
        let chapters: Vec<Chapter> = cs.query_map([v.id], |row| {
            Ok(Chapter {
                id: row.get(0)?, volume_id: row.get(1)?, title: row.get(2)?,
                file_path: row.get(3)?, sort_order: row.get(4)?, summary: row.get(5)?,
                word_count_cache: row.get(6)?, created_at: row.get(7)?, updated_at: row.get(8)?,
            })
        }).map_err(|e| format!("查询失败: {}", e))?.filter_map(|r| r.ok()).collect();
        result.push(VolumeWithChapters { id: v.id, project_id: v.project_id, name: v.name, sort_order: v.sort_order, chapters });
    }
    Ok(result)
}

#[tauri::command]
pub async fn get_chapter_content(app_handle: AppHandle, project_id: String, chapter_id: String) -> Result<String, String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;
    let (storage_path, project_name): (String, String) = conn
        .query_row("SELECT path, name FROM projects WHERE id = ?1", [project_id], |row| Ok((row.get(0)?, row.get(1)?)))
        .map_err(|e| format!("项目不存在: {}", e))?;
    let full_path = Path::new(&storage_path).join(&project_name).join("chapters").join(format!("{}.md", chapter_id));
    if !full_path.exists() { return Ok(String::new()); }
    fs::read_to_string(&full_path).map_err(|e| format!("读取章节失败: {}", e))
}

#[tauri::command]
pub async fn save_chapter_content(app_handle: AppHandle, project_id: String, chapter_id: String, content: String) -> Result<(), String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;
    let (storage_path, project_name): (String, String) = conn
        .query_row("SELECT path, name FROM projects WHERE id = ?1", [project_id], |row| Ok((row.get(0)?, row.get(1)?)))
        .map_err(|e| format!("项目不存在: {}", e))?;
    let chapters_dir = Path::new(&storage_path).join(&project_name).join("chapters");
    fs::create_dir_all(&chapters_dir).map_err(|e| format!("创建章节目录失败: {}", e))?;
    let chapter_path = chapters_dir.join(format!("{}.md", chapter_id));
    fs::write(&chapter_path, content).map_err(|e| format!("保存章节失败: {}", e))?;
    Ok(())
}
