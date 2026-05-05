use crate::db::{get_db_path, init_db};
use crate::models::{SensitiveWord, SensitiveWordMatch};
use aho_corasick::AhoCorasick;
use rusqlite::{params, Connection};
use std::fs;
use tauri::AppHandle;

/// 添加敏感词
#[tauri::command]
pub async fn add_sensitive_word(
    app_handle: AppHandle,
    project_id: i64,
    word: String,
) -> Result<SensitiveWord, String> {
    let word = word.trim().to_string();
    if word.is_empty() {
        return Err("敏感词不能为空".into());
    }

    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;
    init_db(&conn).map_err(|e| format!("数据库初始化失败: {}", e))?;

    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO sensitive_words (project_id, word, created_at) VALUES (?1, ?2, ?3)
         ON CONFLICT(project_id, word) DO NOTHING",
        params![project_id, word, now],
    )
    .map_err(|e| format!("添加敏感词失败: {}", e))?;

    let result = conn
        .query_row(
            "SELECT id, project_id, word, created_at FROM sensitive_words WHERE project_id = ?1 AND word = ?2",
            params![project_id, word],
            |row| {
                Ok(SensitiveWord {
                    id: row.get(0)?,
                    project_id: row.get(1)?,
                    word: row.get(2)?,
                    created_at: row.get(3)?,
                })
            },
        )
        .map_err(|e| format!("查询敏感词失败: {}", e))?;
    Ok(result)
}

/// 删除敏感词
#[tauri::command]
pub async fn remove_sensitive_word(
    app_handle: AppHandle,
    project_id: i64,
    word: String,
) -> Result<(), String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;
    init_db(&conn).map_err(|e| format!("数据库初始化失败: {}", e))?;

    conn.execute(
        "DELETE FROM sensitive_words WHERE project_id = ?1 AND word = ?2",
        params![project_id, word],
    )
    .map_err(|e| format!("删除敏感词失败: {}", e))?;
    Ok(())
}

/// 列出项目的所有敏感词
#[tauri::command]
pub async fn list_sensitive_words(
    app_handle: AppHandle,
    project_id: i64,
) -> Result<Vec<SensitiveWord>, String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;
    init_db(&conn).map_err(|e| format!("数据库初始化失败: {}", e))?;

    let mut stmt = conn
        .prepare("SELECT id, project_id, word, created_at FROM sensitive_words WHERE project_id = ?1 ORDER BY id")
        .map_err(|e| format!("查询失败: {}", e))?;

    let words = stmt
        .query_map(params![project_id], |row| {
            Ok(SensitiveWord {
                id: row.get(0)?,
                project_id: row.get(1)?,
                word: row.get(2)?,
                created_at: row.get(3)?,
            })
        })
        .map_err(|e| format!("查询执行失败: {}", e))?
        .filter_map(|r| r.ok())
        .collect();
    Ok(words)
}

/// 从文件导入敏感词（每行一个）
#[tauri::command]
pub async fn import_sensitive_words(
    app_handle: AppHandle,
    project_id: i64,
    file_path: String,
) -> Result<Vec<SensitiveWord>, String> {
    let content =
        fs::read_to_string(&file_path).map_err(|e| format!("读取文件失败: {}", e))?;

    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;
    init_db(&conn).map_err(|e| format!("数据库初始化失败: {}", e))?;

    let now = chrono::Utc::now().to_rfc3339();
    let mut imported = Vec::new();

    for line in content.lines() {
        let word = line.trim().to_string();
        if word.is_empty() {
            continue;
        }
        conn.execute(
            "INSERT OR IGNORE INTO sensitive_words (project_id, word, created_at) VALUES (?1, ?2, ?3)",
            params![project_id, word, now],
        )
        .map_err(|e| format!("导入敏感词失败: {}", e))?;

        if let Ok(word_row) = conn.query_row(
            "SELECT id, project_id, word, created_at FROM sensitive_words WHERE project_id = ?1 AND word = ?2",
            params![project_id, word],
            |row| {
                Ok(SensitiveWord {
                    id: row.get(0)?,
                    project_id: row.get(1)?,
                    word: row.get(2)?,
                    created_at: row.get(3)?,
                })
            },
        ) {
            imported.push(word_row);
        }
    }
    Ok(imported)
}

/// 扫描内容中的敏感词，返回匹配位置
#[tauri::command]
pub async fn scan_sensitive_words(
    app_handle: AppHandle,
    project_id: i64,
    content: String,
) -> Result<Vec<SensitiveWordMatch>, String> {
    if content.is_empty() {
        return Ok(Vec::new());
    }

    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;
    init_db(&conn).map_err(|e| format!("数据库初始化失败: {}", e))?;

    let mut stmt = conn
        .prepare("SELECT word FROM sensitive_words WHERE project_id = ?1")
        .map_err(|e| format!("查询失败: {}", e))?;

    let patterns: Vec<String> = stmt
        .query_map(params![project_id], |row| row.get::<_, String>(0))
        .map_err(|e| format!("查询执行失败: {}", e))?
        .filter_map(|r| r.ok())
        .collect();

    if patterns.is_empty() {
        return Ok(Vec::new());
    }

    let ac = AhoCorasick::new(&patterns)
        .map_err(|e| format!("构建搜索器失败: {}", e))?;

    let matches: Vec<SensitiveWordMatch> = ac
        .find_iter(&content)
        .map(|m| SensitiveWordMatch {
            word: patterns[m.pattern().as_usize()].clone(),
            start: m.start(),
            end: m.end(),
        })
        .collect();

    Ok(matches)
}
