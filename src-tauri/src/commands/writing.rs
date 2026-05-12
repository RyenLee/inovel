use crate::db::{get_db_path, init_db};
use crate::logging::operation::record_simple_operation;
use crate::models::{FocusSession, FocusStats, WritingGoal, WritingRecord};
use rusqlite::{Connection, params};
use std::fs;
use tauri::AppHandle;

/// 获取写作目标
///
/// 获取指定项目的每日写作目标设置。从项目的 project.json 文件中读取。
///
/// # 参数
/// - `app_handle`: Tauri 应用句柄
/// - `project_id`: 项目 ID
///
/// # 返回值
/// 成功返回写作目标（可能为 None），失败返回错误信息
#[tauri::command]
pub async fn get_writing_goal(
    app_handle: AppHandle,
    project_id: i64,
) -> Result<Option<WritingGoal>, String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;

    // 首先从数据库获取项目路径
    let project_path: Option<String> = conn
        .query_row(
            "SELECT path FROM projects WHERE id = ?1",
            [project_id],
            |row| row.get(0),
        )
        .ok();

    if let Some(path) = project_path {
        let project_json_path = std::path::Path::new(&path).join("project.json");
        if project_json_path.exists() {
            let content = fs::read_to_string(&project_json_path)
                .map_err(|e| format!("读取 project.json 失败: {}", e))?;
            let project_json: serde_json::Value = serde_json::from_str(&content)
                .map_err(|e| format!("解析 project.json 失败: {}", e))?;

            if let Some(writing_goal) = project_json.get("writing_goal").and_then(|v| v.as_i64()) {
                return Ok(Some(WritingGoal {
                    id: 0,
                    project_id,
                    daily_goal: writing_goal as i32,
                    updated_at: chrono::Utc::now().to_rfc3339(),
                }));
            }
        }
    }

    // 如果 project.json 中没有写作目标，尝试从数据库读取（兼容旧数据）
    let result: Option<WritingGoal> = conn
        .query_row(
            "SELECT id, project_id, daily_goal, updated_at FROM writing_goals WHERE project_id = ?1",
            [project_id],
            |row| Ok(WritingGoal { id: row.get(0)?, project_id: row.get(1)?, daily_goal: row.get(2)?, updated_at: row.get(3)? }),
        ).ok();

    Ok(result)
}

/// 保存写作目标
///
/// 创建或更新项目的每日写作目标。将目标保存到项目的 project.json 文件中。
///
/// # 参数
/// - `app_handle`: Tauri 应用句柄
/// - `project_id`: 项目 ID
/// - `daily_goal`: 每日目标字数
///
/// # 返回值
/// 成功返回保存的写作目标，失败返回错误信息
#[tauri::command]
pub async fn save_writing_goal(
    app_handle: AppHandle,
    project_id: i64,
    daily_goal: i32,
) -> Result<WritingGoal, String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;
    init_db(&conn).map_err(|e| format!("数据库初始化失败: {}", e))?;

    // 首先从数据库获取项目路径
    let project_path: Option<String> = conn
        .query_row(
            "SELECT path FROM projects WHERE id = ?1",
            [project_id],
            |row| row.get(0),
        )
        .ok();

    let now = chrono::Utc::now().to_rfc3339();

    if let Some(path) = project_path {
        let project_json_path = std::path::Path::new(&path).join("project.json");
        if project_json_path.exists() {
            // 读取现有 project.json
            let content = fs::read_to_string(&project_json_path)
                .map_err(|e| format!("读取 project.json 失败: {}", e))?;
            let mut project_json: serde_json::Value = serde_json::from_str(&content)
                .map_err(|e| format!("解析 project.json 失败: {}", e))?;

            // 更新写作目标
            project_json["writing_goal"] = serde_json::json!(daily_goal);

            // 写回 project.json
            fs::write(
                &project_json_path,
                serde_json::to_string_pretty(&project_json)
                    .map_err(|e| format!("序列化 project.json 失败: {}", e))?,
            )
            .map_err(|e| format!("写入 project.json 失败: {}", e))?;

            tracing::info!(project_id = %project_id, daily_goal = %daily_goal, path = %path, "写作目标已保存到 project.json");
        } else {
            tracing::warn!(project_id = %project_id, path = %path, "project.json 不存在");
        }
    } else {
        tracing::warn!(project_id = %project_id, "无法找到项目路径");
    }

    // 同时更新 writing_goals 数据库表
    conn.execute(
        "INSERT INTO writing_goals (project_id, daily_goal, updated_at) VALUES (?1, ?2, ?3)
         ON CONFLICT(project_id) DO UPDATE SET daily_goal = ?2, updated_at = ?3",
        (project_id, daily_goal, &now),
    )
    .map_err(|e| format!("保存写作目标失败: {}", e))?;
    let result = conn.query_row(
        "SELECT id, project_id, daily_goal, updated_at FROM writing_goals WHERE project_id = ?1",
        [project_id],
        |row| Ok(WritingGoal { id: row.get(0)?, project_id: row.get(1)?, daily_goal: row.get(2)?, updated_at: row.get(3)? }),
    ).map_err(|e| format!("查询写作目标失败: {}", e))?;
    tracing::info!(project_id = %project_id, daily_goal = %daily_goal, "写作目标已保存到数据库");
    Ok(result)
}

/// 获取写作统计数据
///
/// 获取指定天数内的写作记录统计。当 project_id 为 0 时返回所有项目的汇总数据。
///
/// # 参数
/// - `app_handle`: Tauri 应用句柄
/// - `project_id`: 项目 ID（0 表示所有项目）
/// - `days`: 查询天数
///
/// # 返回值
/// 成功返回写作记录列表，失败返回错误信息
#[tauri::command]
pub async fn get_writing_stats(
    app_handle: AppHandle,
    project_id: i64,
    days: i32,
) -> Result<Vec<WritingRecord>, String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;

    let records: Vec<WritingRecord> = if project_id == 0 {
        let mut stmt = conn
            .prepare(
                "SELECT record_date, SUM(total_words) as total_words, SUM(duration) as duration
             FROM writing_records GROUP BY record_date ORDER BY record_date DESC LIMIT ?1",
            )
            .map_err(|e| format!("查询失败: {}", e))?;
        stmt.query_map(params![days as i64], |row| {
            Ok(WritingRecord {
                date: row.get(0)?,
                total_words: row.get::<_, i64>(1)? as i32,
                duration: row.get::<_, i64>(2)? as i32,
            })
        })
        .map_err(|e| format!("查询执行失败: {}", e))?
        .filter_map(|r| r.ok())
        .collect()
    } else {
        let mut stmt = conn.prepare(
            "SELECT record_date, total_words, duration FROM writing_records WHERE project_id = ?1 ORDER BY record_date DESC LIMIT ?2"
        ).map_err(|e| format!("查询失败: {}", e))?;
        stmt.query_map(params![project_id, days as i64], |row| {
            Ok(WritingRecord {
                date: row.get(0)?,
                total_words: row.get(1)?,
                duration: row.get(2)?,
            })
        })
        .map_err(|e| format!("查询执行失败: {}", e))?
        .filter_map(|r| r.ok())
        .collect()
    };
    Ok(records)
}

/// 更新或插入写作记录
///
/// 使用 INSERT OR UPDATE 语义，为当前日期创建或更新写作记录。
///
/// # 参数
/// - `app_handle`: Tauri 应用句柄
/// - `project_id`: 项目 ID
/// - `total_words`: 写作字数
/// - `duration`: 写作时长（分钟）
///
/// # 返回值
/// 成功返回写作记录，失败返回错误信息
#[tauri::command]
pub async fn upsert_writing_record(
    app_handle: AppHandle,
    project_id: i64,
    total_words: i32,
    duration: i32,
) -> Result<WritingRecord, String> {
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

/// 获取今日写作记录
///
/// 获取指定项目今日的写作字数和时长。
///
/// # 参数
/// - `app_handle`: Tauri 应用句柄
/// - `project_id`: 项目 ID
///
/// # 返回值
/// 成功返回今日写作记录（可能为 None），失败返回错误信息
#[tauri::command]
pub async fn get_today_words(
    app_handle: AppHandle,
    project_id: i64,
) -> Result<Option<WritingRecord>, String> {
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

/// 记录专注会话
///
/// 创建一条新的番茄钟专注会话记录。
///
/// # 参数
/// - `app_handle`: Tauri 应用句柄
/// - `project_id`: 项目 ID
/// - `session_type`: 会话类型（work/short_break/long_break）
/// - `duration_minutes`: 会话时长（分钟）
/// - `started_at`: 开始时间
/// - `completed`: 是否完成
///
/// # 返回值
/// 成功返回专注会话记录，失败返回错误信息
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

    let _ = record_simple_operation(
        &app_handle,
        "focus",
        "record_session",
        "focus_session",
        Some(id),
        Some(&format!(
            "记录专注会话: {}分钟 ({})",
            duration_minutes, session_type
        )),
        Some(project_id),
    );

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

/// 获取专注会话列表
///
/// 获取指定天数内的所有专注会话记录。当 project_id 为 0 时返回所有项目的会话记录。
///
/// # 参数
/// - `app_handle`: Tauri 应用句柄
/// - `project_id`: 项目 ID（0 表示所有项目）
/// - `days`: 查询天数
///
/// # 返回值
/// 成功返回专注会话列表，失败返回错误信息
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

    let sessions: Vec<FocusSession> = if project_id == 0 {
        let mut stmt = conn.prepare(
            "SELECT id, project_id, session_type, duration_minutes, started_at, completed, created_at
             FROM focus_sessions WHERE started_at >= ?1 ORDER BY started_at DESC"
        ).map_err(|e| format!("查询失败: {}", e))?;
        stmt.query_map(params![cutoff_str], |row| {
            Ok(FocusSession {
                id: row.get(0)?,
                project_id: row.get(1)?,
                session_type: row.get(2)?,
                duration_minutes: row.get(3)?,
                started_at: row.get(4)?,
                completed: row.get::<_, i32>(5)? == 1,
                created_at: row.get(6)?,
            })
        })
        .map_err(|e| format!("查询执行失败: {}", e))?
        .filter_map(|r| r.ok())
        .collect()
    } else {
        let mut stmt = conn.prepare(
            "SELECT id, project_id, session_type, duration_minutes, started_at, completed, created_at
             FROM focus_sessions WHERE project_id = ?1 AND started_at >= ?2 ORDER BY started_at DESC"
        ).map_err(|e| format!("查询失败: {}", e))?;
        stmt.query_map(params![project_id, cutoff_str], |row| {
            Ok(FocusSession {
                id: row.get(0)?,
                project_id: row.get(1)?,
                session_type: row.get(2)?,
                duration_minutes: row.get(3)?,
                started_at: row.get(4)?,
                completed: row.get::<_, i32>(5)? == 1,
                created_at: row.get(6)?,
            })
        })
        .map_err(|e| format!("查询执行失败: {}", e))?
        .filter_map(|r| r.ok())
        .collect()
    };

    Ok(sessions)
}

/// 获取专注统计数据
///
/// 获取指定天数内的专注会话统计汇总。当 project_id 为 0 时返回所有项目的统计数据。
///
/// # 参数
/// - `app_handle`: Tauri 应用句柄
/// - `project_id`: 项目 ID（0 表示所有项目）
/// - `days`: 查询天数
///
/// # 返回值
/// 成功返回统计数据（包含总会话数、总时长、完成数等），失败返回错误信息
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

    let stats = if project_id == 0 {
        conn.query_row(
            "SELECT 
                COUNT(*) as total_sessions,
                COALESCE(SUM(duration_minutes), 0) as total_minutes,
                COALESCE(SUM(CASE WHEN completed = 1 THEN 1 ELSE 0 END), 0) as completed_sessions,
                COALESCE(SUM(CASE WHEN session_type = 'work' THEN 1 ELSE 0 END), 0) as work_sessions,
                COALESCE(SUM(CASE WHEN session_type = 'short_break' THEN 1 ELSE 0 END), 0) as short_break_sessions,
                COALESCE(SUM(CASE WHEN session_type = 'long_break' THEN 1 ELSE 0 END), 0) as long_break_sessions,
                COALESCE(SUM(CASE WHEN session_type = 'work' AND completed = 1 THEN 1 ELSE 0 END), 0) as completed_work_sessions,
                COALESCE(SUM(CASE WHEN session_type = 'work' THEN duration_minutes ELSE 0 END), 0) as work_duration_minutes
             FROM focus_sessions WHERE started_at >= ?1",
            params![cutoff_str],
            |row| Ok(FocusStats {
                total_sessions: row.get(0)?,
                total_minutes: row.get(1)?,
                completed_sessions: row.get(2)?,
                work_sessions: row.get(3)?,
                short_break_sessions: row.get(4)?,
                long_break_sessions: row.get(5)?,
                completed_work_sessions: row.get(6)?,
                work_duration_minutes: row.get(7)?,
            }),
        ).map_err(|e| format!("查询统计数据失败: {}", e))?
    } else {
        conn.query_row(
            "SELECT 
                COUNT(*) as total_sessions,
                COALESCE(SUM(duration_minutes), 0) as total_minutes,
                COALESCE(SUM(CASE WHEN completed = 1 THEN 1 ELSE 0 END), 0) as completed_sessions,
                COALESCE(SUM(CASE WHEN session_type = 'work' THEN 1 ELSE 0 END), 0) as work_sessions,
                COALESCE(SUM(CASE WHEN session_type = 'short_break' THEN 1 ELSE 0 END), 0) as short_break_sessions,
                COALESCE(SUM(CASE WHEN session_type = 'long_break' THEN 1 ELSE 0 END), 0) as long_break_sessions,
                COALESCE(SUM(CASE WHEN session_type = 'work' AND completed = 1 THEN 1 ELSE 0 END), 0) as completed_work_sessions,
                COALESCE(SUM(CASE WHEN session_type = 'work' THEN duration_minutes ELSE 0 END), 0) as work_duration_minutes
             FROM focus_sessions WHERE project_id = ?1 AND started_at >= ?2",
            params![project_id, cutoff_str],
            |row| Ok(FocusStats {
                total_sessions: row.get(0)?,
                total_minutes: row.get(1)?,
                completed_sessions: row.get(2)?,
                work_sessions: row.get(3)?,
                short_break_sessions: row.get(4)?,
                long_break_sessions: row.get(5)?,
                completed_work_sessions: row.get(6)?,
                work_duration_minutes: row.get(7)?,
            }),
        ).map_err(|e| format!("查询统计数据失败: {}", e))?
    };

    Ok(stats)
}
