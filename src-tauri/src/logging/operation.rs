use chrono::Utc;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::config::get_db_path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationLog {
    pub id: i64,
    pub timestamp: String,
    pub user_id: String,
    pub operation_type: String,
    pub operation_action: String,
    pub target_type: String,
    pub target_id: Option<String>,
    pub details: Option<String>,
    pub result: OperationResult,
    pub duration_ms: Option<i64>,
    pub ip_address: Option<String>,
    pub project_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OperationResult {
    Success,
    Failed,
    Partial,
}

impl From<OperationResult> for &str {
    fn from(result: OperationResult) -> Self {
        match result {
            OperationResult::Success => "success",
            OperationResult::Failed => "failed",
            OperationResult::Partial => "partial",
        }
    }
}

impl From<&str> for OperationResult {
    fn from(s: &str) -> Self {
        match s {
            "success" => OperationResult::Success,
            "failed" => OperationResult::Failed,
            "partial" => OperationResult::Partial,
            _ => OperationResult::Failed,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationType {
    pub category: String,
    pub action: String,
    pub description: String,
}

pub const OPERATION_CATEGORIES: &[&str] = &[
    "project",
    "chapter",
    "writing",
    "worldbuilding",
    "relationship",
    "timeline",
    "export",
    "backup",
    "encryption",
    "template",
    "system",
    "settings",
];

static DB_INIT: std::sync::Once = std::sync::Once::new();

fn ensure_operation_log_table(conn: &Connection) -> Result<(), String> {
    DB_INIT.call_once(|| {
        let sql = r#"
            CREATE TABLE IF NOT EXISTS operation_logs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL DEFAULT (datetime('now')),
                user_id TEXT NOT NULL DEFAULT 'system',
                operation_type TEXT NOT NULL,
                operation_action TEXT NOT NULL,
                target_type TEXT NOT NULL DEFAULT 'unknown',
                target_id TEXT,
                details TEXT,
                result TEXT NOT NULL DEFAULT 'success',
                duration_ms INTEGER,
                ip_address TEXT,
                project_id INTEGER,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE INDEX IF NOT EXISTS idx_operation_logs_timestamp ON operation_logs(timestamp);
            CREATE INDEX IF NOT EXISTS idx_operation_logs_type ON operation_logs(operation_type);
            CREATE INDEX IF NOT EXISTS idx_operation_logs_project ON operation_logs(project_id);
            CREATE INDEX IF NOT EXISTS idx_operation_logs_result ON operation_logs(result);

            CREATE TABLE IF NOT EXISTS operation_log_stats (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                date TEXT NOT NULL,
                operation_type TEXT NOT NULL,
                count INTEGER NOT NULL DEFAULT 0,
                success_count INTEGER NOT NULL DEFAULT 0,
                failed_count INTEGER NOT NULL DEFAULT 0,
                UNIQUE(date, operation_type)
            );
        "#;
        let _ = conn.execute_batch(sql);
    });
    Ok(())
}

pub fn init_operation_log_db(app_handle: &AppHandle) -> Result<(), String> {
    let db_path = get_db_path(app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("打开操作日志数据库失败: {}", e))?;

    ensure_operation_log_table(&conn)?;

    Ok(())
}

pub fn record_operation(
    app_handle: &AppHandle,
    params: RecordOperationParams,
) -> Result<OperationLog, String> {
    let db_path = get_db_path(app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;

    ensure_operation_log_table(&conn)?;

    let result_str: &str = params.result.clone().into();
    let op_type = params.operation_type.clone();

    conn.execute(
        "INSERT INTO operation_logs (user_id, operation_type, operation_action, target_type, target_id, details, result, duration_ms, ip_address, project_id)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        (
            &params.user_id,
            &params.operation_type,
            &params.operation_action,
            &params.target_type,
            &params.target_id,
            &params.details,
            result_str,
            params.duration_ms,
            &params.ip_address,
            params.project_id,
        ),
    )
    .map_err(|e| format!("记录操作日志失败: {}", e))?;

    let id = conn.last_insert_rowid();
    update_stats(&conn, &op_type, &params.result)?;

    Ok(OperationLog {
        id,
        timestamp: Utc::now().to_rfc3339(),
        user_id: params.user_id,
        operation_type: op_type,
        operation_action: params.operation_action,
        target_type: params.target_type,
        target_id: params.target_id,
        details: params.details,
        result: params.result,
        duration_ms: params.duration_ms,
        ip_address: params.ip_address,
        project_id: params.project_id,
    })
}

fn update_stats(conn: &Connection, op_type: &str, result: &OperationResult) -> Result<(), String> {
    let today = Utc::now().format("%Y-%m-%d").to_string();
    let is_success = *result == OperationResult::Success;

    let existing: Option<(i64, i64)> = conn
        .query_row(
            "SELECT count, success_count FROM operation_log_stats WHERE date = ?1 AND operation_type = ?2",
            [today.as_str(), op_type],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .ok();

    match existing {
        Some((_count, success_count)) => {
            let new_success = if is_success {
                success_count + 1
            } else {
                success_count
            };
            conn.execute(
                "UPDATE operation_log_stats SET count = count + 1, success_count = ?1 WHERE date = ?2 AND operation_type = ?3",
                (new_success, today.as_str(), op_type),
            )
            .map_err(|e| format!("更新统计失败: {}", e))?;
        }
        None => {
            conn.execute(
                "INSERT INTO operation_log_stats (date, operation_type, count, success_count, failed_count) VALUES (?1, ?2, 1, ?3, ?4)",
                (today.as_str(), op_type, if is_success { 1 } else { 0 }, if is_success { 0 } else { 1 }),
            )
            .map_err(|e| format!("插入统计失败: {}", e))?;
        }
    }

    Ok(())
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RecordOperationParams {
    #[serde(default = "default_user_id")]
    pub user_id: String,
    pub operation_type: String,
    pub operation_action: String,
    #[serde(default = "default_target_type")]
    pub target_type: String,
    pub target_id: Option<String>,
    pub details: Option<String>,
    #[serde(default = "default_result")]
    pub result: OperationResult,
    pub duration_ms: Option<i64>,
    pub ip_address: Option<String>,
    pub project_id: Option<i64>,
}

fn default_user_id() -> String {
    "system".to_string()
}

fn default_target_type() -> String {
    "unknown".to_string()
}

fn default_result() -> OperationResult {
    OperationResult::Success
}

#[derive(Clone, Serialize, Deserialize)]
pub struct OperationLogFilter {
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub operation_type: Option<String>,
    pub result: Option<OperationResult>,
    pub project_id: Option<i64>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

impl Default for OperationLogFilter {
    fn default() -> Self {
        Self {
            start_date: None,
            end_date: None,
            operation_type: None,
            result: None,
            project_id: None,
            limit: Some(100),
            offset: Some(0),
        }
    }
}

pub fn query_operation_logs(
    app_handle: &AppHandle,
    filter: OperationLogFilter,
) -> Result<Vec<OperationLog>, String> {
    let db_path = get_db_path(app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;

    ensure_operation_log_table(&conn)?;

    let mut conditions = Vec::new();
    let mut values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(ref start) = filter.start_date {
        conditions.push("timestamp >= ?");
        values.push(Box::new(start.clone()));
    }
    if let Some(ref end) = filter.end_date {
        conditions.push("timestamp <= ?");
        values.push(Box::new(end.clone()));
    }
    if let Some(ref op_type) = filter.operation_type {
        conditions.push("operation_type = ?");
        values.push(Box::new(op_type.clone()));
    }
    if let Some(ref result) = filter.result {
        let result_str: &str = result.clone().into();
        conditions.push("result = ?");
        values.push(Box::new(result_str.to_string()));
    }
    if let Some(pid) = filter.project_id {
        conditions.push("project_id = ?");
        values.push(Box::new(pid));
    }

    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };

    let limit = filter.limit.unwrap_or(100).min(1000);
    let offset = filter.offset.unwrap_or(0);

    let sql = format!(
        "SELECT id, timestamp, user_id, operation_type, operation_action, target_type, target_id, details, result, duration_ms, ip_address, project_id
         FROM operation_logs {}
         ORDER BY timestamp DESC
         LIMIT {} OFFSET {}",
        where_clause, limit, offset
    );

    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| format!("查询准备失败: {}", e))?;

    let logs: Vec<OperationLog> = stmt
        .query_map(rusqlite::params_from_iter(values.iter()), |row| {
            let result_str: String = row.get(8)?;
            Ok(OperationLog {
                id: row.get(0)?,
                timestamp: row.get(1)?,
                user_id: row.get(2)?,
                operation_type: row.get(3)?,
                operation_action: row.get(4)?,
                target_type: row.get(5)?,
                target_id: row.get(6)?,
                details: row.get(7)?,
                result: OperationResult::from(result_str.as_str()),
                duration_ms: row.get(9)?,
                ip_address: row.get(10)?,
                project_id: row.get(11)?,
            })
        })
        .map_err(|e| format!("查询操作日志失败: {}", e))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(logs)
}

pub fn get_operation_stats(
    app_handle: &AppHandle,
    days: u32,
) -> Result<Vec<OperationStat>, String> {
    let db_path = get_db_path(app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;

    ensure_operation_log_table(&conn)?;

    let since = Utc::now() - chrono::Duration::days(days as i64);
    let date_since = since.format("%Y-%m-%d").to_string();

    let mut stmt = conn
        .prepare(
            "SELECT date, operation_type, count, success_count, failed_count
             FROM operation_log_stats
             WHERE date >= ?1
             ORDER BY date DESC, operation_type",
        )
        .map_err(|e| format!("查询统计失败: {}", e))?;

    let stats: Vec<OperationStat> = stmt
        .query_map([&date_since], |row| {
            Ok(OperationStat {
                date: row.get(0)?,
                operation_type: row.get(1)?,
                total_count: row.get(2)?,
                success_count: row.get(3)?,
                failed_count: row.get(4)?,
            })
        })
        .map_err(|e| format!("解析统计数据失败: {}", e))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(stats)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationStat {
    pub date: String,
    pub operation_type: String,
    pub total_count: i64,
    pub success_count: i64,
    pub failed_count: i64,
}
