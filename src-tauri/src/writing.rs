use crate::db::{get_db_path, init_db};
use crate::models::{FocusSession, FocusStats, WritingGoal, WritingRecord};
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

// ==================== 番茄钟专注会话相关命令 ====================

#[tauri::command]
pub async fn record_focus_session(
    app_handle: AppHandle,
    project_id: i64,
    session_type: String,
    duration_minutes: i32,
    started_at: String,
    completed: bool,
) -> Result<FocusSession, String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;
    init_db(&conn).map_err(|e| format!("数据库初始化失败: {}", e))?;
    let now = chrono::Utc::now().to_rfc3339();
    let completed_int = if completed { 1 } else { 0 };
    
    conn.execute(
        "INSERT INTO focus_sessions (project_id, session_type, duration_minutes, started_at, completed, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![project_id, session_type, duration_minutes, started_at, completed_int, now],
    ).map_err(|e| format!("记录专注会话失败: {}", e))?;
    
    let id = conn.last_insert_rowid();
    Ok(FocusSession {
        id,
        project_id,
        session_type,
        duration_minutes,
        started_at,
        completed,
        created_at: now,
    })
}

#[tauri::command]
pub async fn get_focus_sessions(
    app_handle: AppHandle,
    project_id: i64,
    days: i32,
) -> Result<Vec<FocusSession>, String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;
    
    let cutoff_date = chrono::Local::now() - chrono::Duration::days(days as i64);
    let cutoff_str = cutoff_date.format("%Y-%m-%d").to_string();
    
    let mut stmt = conn.prepare(
        "SELECT id, project_id, session_type, duration_minutes, started_at, completed, created_at
         FROM focus_sessions WHERE project_id = ?1 AND started_at >= ?2 ORDER BY started_at DESC"
    ).map_err(|e| format!("查询失败: {}", e))?;
    
    let sessions = stmt.query_map(params![project_id, cutoff_str], |row| {
        Ok(FocusSession {
            id: row.get(0)?,
            project_id: row.get(1)?,
            session_type: row.get(2)?,
            duration_minutes: row.get(3)?,
            started_at: row.get(4)?,
            completed: row.get::<_, i32>(5)? == 1,
            created_at: row.get(6)?,
        })
    }).map_err(|e| format!("查询执行失败: {}", e))?
    .filter_map(|r| r.ok())
    .collect();
    
    Ok(sessions)
}

#[tauri::command]
pub async fn get_focus_stats(
    app_handle: AppHandle,
    project_id: i64,
    days: i32,
) -> Result<FocusStats, String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;
    
    let cutoff_date = chrono::Local::now() - chrono::Duration::days(days as i64);
    let cutoff_str = cutoff_date.format("%Y-%m-%d").to_string();
    
    let stats = conn.query_row(
        "SELECT 
            COUNT(*) as total_sessions,
            COALESCE(SUM(duration_minutes), 0) as total_minutes,
            COALESCE(SUM(CASE WHEN completed = 1 THEN 1 ELSE 0 END), 0) as completed_sessions,
            COALESCE(SUM(CASE WHEN session_type = 'work' THEN 1 ELSE 0 END), 0) as work_sessions,
            COALESCE(SUM(CASE WHEN session_type = 'short_break' THEN 1 ELSE 0 END), 0) as short_break_sessions,
            COALESCE(SUM(CASE WHEN session_type = 'long_break' THEN 1 ELSE 0 END), 0) as long_break_sessions
         FROM focus_sessions WHERE project_id = ?1 AND started_at >= ?2",
        params![project_id, cutoff_str],
        |row| Ok(FocusStats {
            total_sessions: row.get(0)?,
            total_minutes: row.get(1)?,
            completed_sessions: row.get(2)?,
            work_sessions: row.get(3)?,
            short_break_sessions: row.get(4)?,
            long_break_sessions: row.get(5)?,
        }),
    ).map_err(|e| format!("查询统计数据失败: {}", e))?;
    
    Ok(stats)
}
