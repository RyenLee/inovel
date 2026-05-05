use crate::db::{get_db_path, init_db};
use crate::models::{WritingGoal, WritingRecord};
use rusqlite::{params, Connection};
use tauri::AppHandle;

#[tauri::command]
pub async fn get_writing_goal(app_handle: AppHandle, project_id: i64) -> Result<Option<WritingGoal>, String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;
    let result: Option<WritingGoal> = conn
        .query_row(
            "SELECT id, project_id, daily_goal, updated_at FROM writing_goals WHERE project_id = ?1",
            [project_id],
            |row| Ok(WritingGoal { id: row.get(0)?, project_id: row.get(1)?, daily_goal: row.get(2)?, updated_at: row.get(3)? }),
        ).ok();
    Ok(result)
}

#[tauri::command]
pub async fn save_writing_goal(app_handle: AppHandle, project_id: i64, daily_goal: i32) -> Result<WritingGoal, String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;
    init_db(&conn).map_err(|e| format!("数据库初始化失败: {}", e))?;
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO writing_goals (project_id, daily_goal, updated_at) VALUES (?1, ?2, ?3)
         ON CONFLICT(project_id) DO UPDATE SET daily_goal = ?2, updated_at = ?3",
        (project_id, daily_goal, &now),
    ).map_err(|e| format!("保存写作目标失败: {}", e))?;
    let result = conn.query_row(
        "SELECT id, project_id, daily_goal, updated_at FROM writing_goals WHERE project_id = ?1",
        [project_id],
        |row| Ok(WritingGoal { id: row.get(0)?, project_id: row.get(1)?, daily_goal: row.get(2)?, updated_at: row.get(3)? }),
    ).map_err(|e| format!("查询写作目标失败: {}", e))?;
    Ok(result)
}

#[tauri::command]
pub async fn get_writing_stats(app_handle: AppHandle, project_id: i64, days: i32) -> Result<Vec<WritingRecord>, String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;

    let records: Vec<WritingRecord> = if project_id == 0 {
        let mut stmt = conn.prepare(
            "SELECT record_date, SUM(total_words) as total_words, SUM(duration) as duration
             FROM writing_records GROUP BY record_date ORDER BY record_date DESC LIMIT ?1"
        ).map_err(|e| format!("查询失败: {}", e))?;
        stmt.query_map(params![days as i64], |row| {
            Ok(WritingRecord { date: row.get(0)?, total_words: row.get::<_, i64>(1)? as i32, duration: row.get::<_, i64>(2)? as i32 })
        }).map_err(|e| format!("查询执行失败: {}", e))?.filter_map(|r| r.ok()).collect()
    } else {
        let mut stmt = conn.prepare(
            "SELECT record_date, total_words, duration FROM writing_records WHERE project_id = ?1 ORDER BY record_date DESC LIMIT ?2"
        ).map_err(|e| format!("查询失败: {}", e))?;
        stmt.query_map(params![project_id, days as i64], |row| {
            Ok(WritingRecord { date: row.get(0)?, total_words: row.get(1)?, duration: row.get(2)? })
        }).map_err(|e| format!("查询执行失败: {}", e))?.filter_map(|r| r.ok()).collect()
    };
    Ok(records)
}

#[tauri::command]
pub async fn upsert_writing_record(app_handle: AppHandle, project_id: i64, total_words: i32, duration: i32) -> Result<WritingRecord, String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;
    init_db(&conn).map_err(|e| format!("数据库初始化失败: {}", e))?;
    let date_str = chrono::Local::now().format("%Y-%m-%d").to_string();
    conn.execute(
        "INSERT INTO writing_records (project_id, record_date, total_words, duration) VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(project_id, record_date) DO UPDATE SET total_words = ?3, duration = ?4",
        (project_id, &date_str, total_words, duration),
    ).map_err(|e| format!("保存写作记录失败: {}", e))?;
    let result = conn.query_row(
        "SELECT record_date, total_words, duration FROM writing_records WHERE project_id = ?1 AND record_date = ?2",
        params![project_id, &date_str],
        |row| Ok(WritingRecord { date: row.get(0)?, total_words: row.get(1)?, duration: row.get(2)? }),
    ).map_err(|e| format!("查询写作记录失败: {}", e))?;
    Ok(result)
}

#[tauri::command]
pub async fn get_today_words(app_handle: AppHandle, project_id: i64) -> Result<Option<WritingRecord>, String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;
    let date_str = chrono::Local::now().format("%Y-%m-%d").to_string();
    let result: Option<WritingRecord> = conn
        .query_row(
            "SELECT record_date, total_words, duration FROM writing_records WHERE project_id = ?1 AND record_date = ?2",
            params![project_id, &date_str],
            |row| Ok(WritingRecord { date: row.get(0)?, total_words: row.get(1)?, duration: row.get(2)? }),
        ).ok();
    Ok(result)
}
