use crate::db::{get_db_path, init_db};
use crate::models::{CreateInspirationItemParams, InspirationItem, UpdateInspirationItemParams};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use tauri::AppHandle;

#[derive(Debug, Serialize, Deserialize)]
pub struct ColumnInfo {
    pub name: String,
    pub items: Vec<InspirationItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BoardData {
    pub columns: Vec<ColumnInfo>,
}

#[tauri::command]
pub async fn create_inspiration_item(
    app_handle: AppHandle,
    params: CreateInspirationItemParams,
) -> Result<InspirationItem, String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;
    init_db(&conn).map_err(|e| format!("数据库初始化失败: {}", e))?;

    // 获取该列的最大排序值
    let max_order: i32 = conn
        .query_row(
            "SELECT COALESCE(MAX(sort_order), 0) FROM inspiration_items WHERE project_id = ?1 AND column_name = ?2",
            params![params.project_id, params.column_name],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let now = chrono::Utc::now().to_rfc3339();
    let sort_order = max_order + 1;

    conn.execute(
        "INSERT INTO inspiration_items (project_id, column_name, content, sort_order, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![params.project_id, params.column_name, params.content, sort_order, now, now],
    )
    .map_err(|e| format!("创建灵感条目失败: {}", e))?;

    let id = conn.last_insert_rowid();
    Ok(InspirationItem {
        id,
        project_id: params.project_id,
        column_name: params.column_name,
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
            "SELECT id, project_id, column_name, content, sort_order, created_at, updated_at FROM inspiration_items WHERE id = ?1",
            [item_id],
            |row| {
                Ok(InspirationItem {
                    id: row.get(0)?,
                    project_id: row.get(1)?,
                    column_name: row.get(2)?,
                    content: row.get(3)?,
                    sort_order: row.get(4)?,
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                })
            },
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

#[tauri::command]
pub async fn reorder_inspiration_items(
    app_handle: AppHandle,
    _project_id: i64,
    updates: Vec<ReorderItem>,
) -> Result<(), String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;
    let now = chrono::Utc::now().to_rfc3339();

    // 开启事务
    conn.execute("BEGIN TRANSACTION", [])
        .map_err(|e| format!("开启事务失败: {}", e))?;

    for update in updates {
        conn.execute(
            "UPDATE inspiration_items SET column_name = ?1, sort_order = ?2, updated_at = ?3 WHERE id = ?4",
            params![update.column_name, update.sort_order, now, update.id],
        )
        .map_err(|e| format!("更新排序失败: {}", e))?;
    }

    conn.execute("COMMIT", [])
        .map_err(|e| format!("提交事务失败: {}", e))?;

    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct ReorderItem {
    pub id: i64,
    pub column_name: String,
    pub sort_order: i32,
}

#[tauri::command]
pub async fn get_inspiration_board(
    app_handle: AppHandle,
    project_id: i64,
) -> Result<BoardData, String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;

    // 获取所有唯一的列名
    let mut stmt = conn
        .prepare("SELECT DISTINCT column_name FROM inspiration_items WHERE project_id = ?1")
        .map_err(|e| format!("查询列名失败: {}", e))?;

    let column_names: Vec<String> = stmt
        .query_map([project_id], |row| row.get(0))
        .map_err(|e| format!("查询失败: {}", e))?
        .filter_map(|r| r.ok())
        .collect();

    // 默认列
    let default_columns = vec![
        "灵感".to_string(),
        "对白".to_string(),
        "场景".to_string(),
    ];

    // 合并默认列和已有列（克隆 default_columns 以避免借用问题）
    let all_columns: Vec<String> = default_columns
        .clone()
        .into_iter()
        .chain(column_names.into_iter().filter(|c| !default_columns.contains(c)))
        .collect();

    let mut columns: Vec<ColumnInfo> = Vec::new();

    for col_name in all_columns {
        let mut stmt = conn
            .prepare(
                "SELECT id, project_id, column_name, content, sort_order, created_at, updated_at 
                 FROM inspiration_items 
                 WHERE project_id = ?1 AND column_name = ?2 
                 ORDER BY sort_order ASC",
            )
            .map_err(|e| format!("查询列数据失败: {}", e))?;

        let items: Vec<InspirationItem> = stmt
            .query_map(params![project_id, col_name], |row| {
                Ok(InspirationItem {
                    id: row.get(0)?,
                    project_id: row.get(1)?,
                    column_name: row.get(2)?,
                    content: row.get(3)?,
                    sort_order: row.get(4)?,
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                })
            })
            .map_err(|e| format!("查询失败: {}", e))?
            .filter_map(|r| r.ok())
            .collect();

        columns.push(ColumnInfo {
            name: col_name,
            items,
        });
    }

    Ok(BoardData { columns })
}

#[tauri::command]
pub async fn get_inspiration_items(
    app_handle: AppHandle,
    project_id: i64,
    column_name: String,
) -> Result<Vec<InspirationItem>, String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;

    let mut stmt = conn
        .prepare(
            "SELECT id, project_id, column_name, content, sort_order, created_at, updated_at 
             FROM inspiration_items 
             WHERE project_id = ?1 AND column_name = ?2 
             ORDER BY sort_order ASC",
        )
        .map_err(|e| format!("查询失败: {}", e))?;

    let items: Vec<InspirationItem> = stmt
        .query_map(params![project_id, column_name], |row| {
            Ok(InspirationItem {
                id: row.get(0)?,
                project_id: row.get(1)?,
                column_name: row.get(2)?,
                content: row.get(3)?,
                sort_order: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })
        .map_err(|e| format!("查询失败: {}", e))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(items)
}
