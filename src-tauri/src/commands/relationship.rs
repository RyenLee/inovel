use crate::db::{get_db_path, init_db};
use crate::models::{CreateRelationshipParams, Relationship, UpdateRelationshipParams};
use rusqlite::{params, Connection};
use tauri::AppHandle;

/// 创建角色关系
///
/// 在数据库中创建一个新的角色关系记录。
///
/// # 参数
/// - `app_handle`: Tauri 应用句柄
/// - `params`: 创建关系参数（包含 project_id, source_id, target_id, relation_type）
///
/// # 返回值
/// 成功返回创建的关系记录，失败返回错误信息
#[tauri::command]
pub async fn create_relationship(app_handle: AppHandle, params: CreateRelationshipParams) -> Result<Relationship, String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;
    init_db(&conn).map_err(|e| format!("数据库初始化失败: {}", e))?;
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO relationships (project_id, source_id, target_id, relation_type, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![params.project_id, params.source_id, params.target_id, params.relation_type, now],
    ).map_err(|e| format!("创建关系失败: {}", e))?;
    let id = conn.last_insert_rowid();
    Ok(Relationship { id, project_id: params.project_id, source_id: params.source_id, target_id: params.target_id, relation_type: params.relation_type, created_at: now })
}

/// 更新角色关系类型
///
/// # 参数
/// - `app_handle`: Tauri 应用句柄
/// - `relationship_id`: 关系 ID
/// - `params`: 更新参数（包含新的 relation_type）
///
/// # 返回值
/// 成功返回更新后的关系记录，失败返回错误信息
#[tauri::command]
pub async fn update_relationship(app_handle: AppHandle, relationship_id: i64, params: UpdateRelationshipParams) -> Result<Relationship, String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;
    conn.execute("UPDATE relationships SET relation_type = ?1 WHERE id = ?2", params![params.relation_type, relationship_id])
        .map_err(|e| format!("更新关系失败: {}", e))?;
    let r = conn.query_row(
        "SELECT id, project_id, source_id, target_id, relation_type, created_at FROM relationships WHERE id = ?1", [relationship_id],
        |row| Ok(Relationship { id: row.get(0)?, project_id: row.get(1)?, source_id: row.get(2)?, target_id: row.get(3)?, relation_type: row.get(4)?, created_at: row.get(5)? }),
    ).map_err(|e| format!("查询关系失败: {}", e))?;
    Ok(r)
}

/// 删除角色关系
///
/// # 参数
/// - `app_handle`: Tauri 应用句柄
/// - `relationship_id`: 关系 ID
///
/// # 返回值
/// 成功返回 Ok(())，失败返回错误信息
#[tauri::command]
pub async fn delete_relationship(app_handle: AppHandle, relationship_id: i64) -> Result<(), String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;
    conn.execute("DELETE FROM relationships WHERE id = ?1", [relationship_id]).map_err(|e| format!("删除关系失败: {}", e))?;
    Ok(())
}

/// 获取项目的所有角色关系
///
/// # 参数
/// - `app_handle`: Tauri 应用句柄
/// - `project_id`: 项目 ID
///
/// # 返回值
/// 成功返回关系列表，失败返回错误信息
#[tauri::command]
pub async fn get_relationships(app_handle: AppHandle, project_id: i64) -> Result<Vec<Relationship>, String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;
    let mut stmt = conn.prepare("SELECT id, project_id, source_id, target_id, relation_type, created_at FROM relationships WHERE project_id = ?1")
        .map_err(|e| format!("查询失败: {}", e))?;
    let rs: Vec<Relationship> = stmt.query_map([project_id], |row| {
        Ok(Relationship { id: row.get(0)?, project_id: row.get(1)?, source_id: row.get(2)?, target_id: row.get(3)?, relation_type: row.get(4)?, created_at: row.get(5)? })
    }).map_err(|e| format!("查询失败: {}", e))?.filter_map(|r| r.ok()).collect();
    Ok(rs)
}
