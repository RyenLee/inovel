use crate::db::{get_db_path, init_db};
use crate::models::{CreateInspirationItemParams, InspirationItem, UpdateInspirationItemParams};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::AppHandle;

#[derive(Debug, Serialize, Deserialize)]
pub struct ColumnInfo {
    pub column_key: String,
    pub column_name: String,
    pub items: Vec<InspirationItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BoardData {
    pub columns: Vec<ColumnInfo>,
}

fn get_column_translations() -> HashMap<&'static str, HashMap<&'static str, &'static str>> {
    let mut map = HashMap::new();

    let mut zh_cn = HashMap::new();
    zh_cn.insert("inspiration", "灵感");
    zh_cn.insert("dialogue", "对白");
    zh_cn.insert("scene", "场景");
    map.insert("zh-CN", zh_cn);

    let mut en_us = HashMap::new();
    en_us.insert("inspiration", "Inspiration");
    en_us.insert("dialogue", "Dialogue");
    en_us.insert("scene", "Scene");
    map.insert("en-US", en_us);

    map
}

fn translate_column_name(column_key: &str, locale: &str) -> String {
    get_column_translations()
        .get(locale)
        .and_then(|m| m.get(column_key))
        .copied()
        .unwrap_or(column_key)
        .to_string()
}

fn read_item_from_row(row: &rusqlite::Row) -> rusqlite::Result<InspirationItem> {
    Ok(InspirationItem {
        id: row.get(0)?,
        project_id: row.get(1)?,
        column_key: row.get(2)?,
        column_name: row.get(3)?,
        content: row.get(4)?,
        sort_order: row.get(5)?,
        created_at: row.get(6)?,
        updated_at: row.get(7)?,
    })
}

#[tauri::command]
pub async fn create_inspiration_item(
    app_handle: AppHandle,
    params: CreateInspirationItemParams,
) -> Result<InspirationItem, String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;
    init_db(&conn).map_err(|e| format!("数据库初始化失败: {}", e))?;

    let max_order: i32 = conn
        .query_row(
            "SELECT COALESCE(MAX(sort_order), 0) FROM inspiration_items WHERE project_id = ?1 AND column_key = ?2",
            params![params.project_id, params.column_key],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let now = chrono::Utc::now().to_rfc3339();
    let sort_order = max_order + 1;

    conn.execute(
        "INSERT INTO inspiration_items (project_id, column_key, column_name, content, sort_order, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![params.project_id, params.column_key, "", params.content, sort_order, now, now],
    )
    .map_err(|e| format!("创建灵感条目失败: {}", e))?;

    let id = conn.last_insert_rowid();
    Ok(InspirationItem {
        id,
        project_id: params.project_id,
        column_key: params.column_key,
        column_name: String::new(),
        content: params.content,
        sort_order,
        created_at: now.clone(),
        updated_at: now,
    })
}

#[tauri::command]
pub async fn update_inspiration_item(
    app_handle: AppHandle,
    item_id: i64,
    params: UpdateInspirationItemParams,
) -> Result<InspirationItem, String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;
    let now = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "UPDATE inspiration_items SET content = ?1, updated_at = ?2 WHERE id = ?3",
        params![params.content, now, item_id],
    )
    .map_err(|e| format!("更新灵感条目失败: {}", e))?;

    let item = conn
        .query_row(
            "SELECT id, project_id, column_key, column_name, content, sort_order, created_at, updated_at FROM inspiration_items WHERE id = ?1",
            [item_id],
            read_item_from_row,
        )
        .map_err(|e| format!("查询灵感条目失败: {}", e))?;

    Ok(item)
}

#[tauri::command]
pub async fn delete_inspiration_item(
    app_handle: AppHandle,
    item_id: i64,
) -> Result<(), String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;
    conn.execute("DELETE FROM inspiration_items WHERE id = ?1", [item_id])
        .map_err(|e| format!("删除灵感条目失败: {}", e))?;
    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct ReorderItem {
    pub id: i64,
    pub column_key: String,
    pub sort_order: i32,
}

#[tauri::command]
pub async fn reorder_inspiration_items(
    app_handle: AppHandle,
    _project_id: i64,
    updates: Vec<ReorderItem>,
) -> Result<(), String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;
    let now = chrono::Utc::now().to_rfc3339();

    conn.execute("BEGIN TRANSACTION", [])
        .map_err(|e| format!("开启事务失败: {}", e))?;

    for update in updates {
        conn.execute(
            "UPDATE inspiration_items SET column_key = ?1, sort_order = ?2, updated_at = ?3 WHERE id = ?4",
            params![update.column_key, update.sort_order, now, update.id],
        )
        .map_err(|e| format!("更新排序失败: {}", e))?;
    }

    conn.execute("COMMIT", [])
        .map_err(|e| format!("提交事务失败: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn get_inspiration_board(
    app_handle: AppHandle,
    project_id: i64,
    locale: String,
) -> Result<BoardData, String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;

    let default_keys = vec!["inspiration", "dialogue", "scene"];

    let mut stmt = conn
        .prepare("SELECT DISTINCT column_key FROM inspiration_items WHERE project_id = ?1")
        .map_err(|e| format!("查询列key失败: {}", e))?;

    let db_column_keys: Vec<String> = stmt
        .query_map([project_id], |row| row.get(0))
        .map_err(|e| format!("查询失败: {}", e))?
        .filter_map(|r| r.ok())
        .collect();

    let all_keys: Vec<String> = default_keys
        .iter()
        .map(|s| s.to_string())
        .chain(db_column_keys.into_iter().filter(|k| !default_keys.contains(&k.as_str())))
        .collect();

    let mut columns: Vec<ColumnInfo> = Vec::new();

    for col_key in all_keys {
        let display_name = translate_column_name(&col_key, &locale);

        let mut stmt = conn
            .prepare(
                "SELECT id, project_id, column_key, column_name, content, sort_order, created_at, updated_at 
                 FROM inspiration_items 
                 WHERE project_id = ?1 AND column_key = ?2 
                 ORDER BY sort_order ASC",
            )
            .map_err(|e| format!("查询列数据失败: {}", e))?;

        let items: Vec<InspirationItem> = stmt
            .query_map(params![project_id, col_key], read_item_from_row)
            .map_err(|e| format!("查询失败: {}", e))?
            .filter_map(|r| r.ok())
            .collect();

        columns.push(ColumnInfo {
            column_key: col_key,
            column_name: display_name,
            items,
        });
    }

    Ok(BoardData { columns })
}

#[tauri::command]
pub async fn get_inspiration_items(
    app_handle: AppHandle,
    project_id: i64,
    column_key: String,
) -> Result<Vec<InspirationItem>, String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;

    let mut stmt = conn
        .prepare(
            "SELECT id, project_id, column_key, column_name, content, sort_order, created_at, updated_at 
             FROM inspiration_items 
             WHERE project_id = ?1 AND column_key = ?2 
             ORDER BY sort_order ASC",
        )
        .map_err(|e| format!("查询失败: {}", e))?;

    let items: Vec<InspirationItem> = stmt
        .query_map(params![project_id, column_key], read_item_from_row)
        .map_err(|e| format!("查询失败: {}", e))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(items)
}
