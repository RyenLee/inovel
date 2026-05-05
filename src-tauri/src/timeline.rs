use crate::db::{get_db_path, init_db};
use crate::models::{CreateEventParams, Event, UpdateEventParams};
use rusqlite::{params, Connection};
use tauri::AppHandle;

#[tauri::command]
pub async fn create_event(app_handle: AppHandle, params: CreateEventParams) -> Result<Event, String> {
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
        id, project_id: params.project_id, title: params.title, story_time: params.story_time,
        description: params.description, chapter_id: params.chapter_id, created_at: now.clone(), updated_at: now,
    })
}

#[tauri::command]
pub async fn update_event(app_handle: AppHandle, event_id: i64, params: UpdateEventParams) -> Result<Event, String> {
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

#[tauri::command]
pub async fn delete_event(app_handle: AppHandle, event_id: i64) -> Result<(), String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;
    conn.execute("DELETE FROM events WHERE id = ?1", [event_id]).map_err(|e| format!("删除事件失败: {}", e))?;
    Ok(())
}

#[tauri::command]
pub async fn list_events(app_handle: AppHandle, project_id: i64) -> Result<Vec<Event>, String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;
    let mut stmt = conn.prepare("SELECT id, project_id, title, story_time, description, chapter_id, created_at, updated_at FROM events WHERE project_id = ?1 ORDER BY story_time ASC")
        .map_err(|e| format!("查询失败: {}", e))?;
    let evs: Vec<Event> = stmt.query_map([project_id], |row| {
        Ok(Event {
            id: row.get(0)?, project_id: row.get(1)?, title: row.get(2)?, story_time: row.get(3)?,
            description: row.get(4)?, chapter_id: row.get(5)?, created_at: row.get(6)?, updated_at: row.get(7)?,
        })
    }).map_err(|e| format!("查询失败: {}", e))?.filter_map(|r| r.ok()).collect();
    Ok(evs)
}
