use crate::db::{get_db_path, init_db};
use crate::models::{CreateEventParams, Event, UpdateEventParams};
use rusqlite::{Connection, params};
use tauri::AppHandle;

/// 创建时间线事件
///
/// 在项目中创建一个新的时间线事件，用于记录故事中的重要事件。
///
/// # 参数
/// - `app_handle`: Tauri 应用句柄
/// - `params`: 事件参数（包含 project_id, title, story_time, description, chapter_id）
///
/// # 返回值
/// 成功返回创建的事件记录，失败返回错误信息
#[tauri::command(rename_all = "snake_case")]
pub async fn create_event(
    app_handle: AppHandle,
    params: CreateEventParams,
) -> Result<Event, String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;
    init_db(&conn).map_err(|e| format!("数据库初始化失败: {}", e))?;
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO events (project_id, title, story_time, description, chapter_id, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![params.project_id, params.title, params.story_time, params.description, params.chapter_id, now, now],
    ).map_err(|e| format!("创建事件失败: {}", e))?;
    let id = conn.last_insert_rowid();
    Ok(Event {
        id,
        project_id: params.project_id,
        title: params.title,
        story_time: params.story_time,
        description: params.description,
        chapter_id: params.chapter_id,
        created_at: now.clone(),
        updated_at: now,
    })
}

/// 更新时间线事件
///
/// 更新指定事件的标题、故事时间、描述或关联章节。
///
/// # 参数
/// - `app_handle`: Tauri 应用句柄
/// - `event_id`: 事件 ID
/// - `params`: 更新参数
///
/// # 返回值
/// 成功返回更新后的事件记录，失败返回错误信息
#[tauri::command(rename_all = "snake_case")]
pub async fn update_event(
    app_handle: AppHandle,
    event_id: i64,
    params: UpdateEventParams,
) -> Result<Event, String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE events SET title=?1, story_time=?2, description=?3, chapter_id=?4, updated_at=?5 WHERE id=?6",
        params![params.title, params.story_time, params.description, params.chapter_id, now, event_id],
    ).map_err(|e| format!("更新事件失败: {}", e))?;
    let ev = conn.query_row(
        "SELECT id, project_id, title, story_time, description, chapter_id, created_at, updated_at FROM events WHERE id = ?1", [event_id],
        |row| Ok(Event {
            id: row.get(0)?, project_id: row.get(1)?, title: row.get(2)?, story_time: row.get(3)?,
            description: row.get(4)?, chapter_id: row.get(5)?, created_at: row.get(6)?, updated_at: row.get(7)?,
        }),
    ).map_err(|e| format!("查询事件失败: {}", e))?;
    Ok(ev)
}

/// 删除时间线事件
///
/// 从数据库中删除指定的时间线事件。
///
/// # 参数
/// - `app_handle`: Tauri 应用句柄
/// - `event_id`: 事件 ID
///
/// # 返回值
/// 成功返回 Ok(())，失败返回错误信息
#[tauri::command(rename_all = "snake_case")]
pub async fn delete_event(app_handle: AppHandle, event_id: i64) -> Result<(), String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;
    conn.execute("DELETE FROM events WHERE id = ?1", [event_id])
        .map_err(|e| format!("删除事件失败: {}", e))?;
    Ok(())
}

/// 列出项目的所有时间线事件
///
/// 返回指定项目下所有时间线事件，按故事时间倒序排列。
///
/// # 参数
/// - `app_handle`: Tauri 应用句柄
/// - `project_id`: 项目 ID
///
/// # 返回值
/// 成功返回事件列表，失败返回错误信息
#[tauri::command(rename_all = "snake_case")]
pub async fn list_events(app_handle: AppHandle, project_id: i64) -> Result<Vec<Event>, String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;
    let mut stmt = conn.prepare("SELECT id, project_id, title, story_time, description, chapter_id, created_at, updated_at FROM events WHERE project_id = ?1 ORDER BY story_time DESC")
        .map_err(|e| format!("查询失败: {}", e))?;
    let evs: Vec<Event> = stmt
        .query_map([project_id], |row| {
            Ok(Event {
                id: row.get(0)?,
                project_id: row.get(1)?,
                title: row.get(2)?,
                story_time: row.get(3)?,
                description: row.get(4)?,
                chapter_id: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })
        .map_err(|e| format!("查询失败: {}", e))?
        .filter_map(|r| r.ok())
        .collect();
    Ok(evs)
}
