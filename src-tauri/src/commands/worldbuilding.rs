use crate::db::{get_db_path, init_db};
use crate::models::{
    Character, CreateCharacterParams, UpdateCharacterParams,
    Location, CreateLocationParams, UpdateLocationParams,
    Organization, CreateOrganizationParams, UpdateOrganizationParams,
};
use rusqlite::{params, Connection};
use tauri::AppHandle;

// ============== Character ==============

/// 创建角色
///
/// 在项目中创建一个新的角色记录，用于世界观构建。
///
/// # 参数
/// - `app_handle`: Tauri 应用句柄
/// - `params`: 创建角色参数（包含 name, gender, age, appearance, personality, background）
///
/// # 返回值
/// 成功返回创建的角色记录，失败返回错误信息
#[tauri::command]
pub async fn create_character(app_handle: AppHandle, params: CreateCharacterParams) -> Result<Character, String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;
    init_db(&conn).map_err(|e| format!("数据库初始化失败: {}", e))?;
    let now = chrono::Utc::now().to_rfc3339();
    let cf = if params.custom_fields.is_empty() { "{}".to_string() } else { params.custom_fields };
    conn.execute(
        "INSERT INTO characters (project_id, name, gender, age, appearance, personality, background, custom_fields, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![params.project_id, params.name, params.gender, params.age, params.appearance, params.personality, params.background, cf, now, now],
    ).map_err(|e| format!("创建角色失败: {}", e))?;
    let id = conn.last_insert_rowid();
    Ok(Character { id, project_id: params.project_id, name: params.name, gender: params.gender, age: params.age, appearance: params.appearance, personality: params.personality, background: params.background, custom_fields: cf, created_at: now.clone(), updated_at: now })
}

/// 更新角色信息
///
/// 更新指定角色的各项属性信息。
///
/// # 参数
/// - `app_handle`: Tauri 应用句柄
/// - `character_id`: 角色 ID
/// - `params`: 更新参数（包含 name, gender, age, appearance, personality, background）
///
/// # 返回值
/// 成功返回更新后的角色记录，失败返回错误信息
#[tauri::command]
pub async fn update_character(app_handle: AppHandle, character_id: i64, params: UpdateCharacterParams) -> Result<Character, String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;
    let now = chrono::Utc::now().to_rfc3339();
    let cf = if params.custom_fields.is_empty() { "{}".to_string() } else { params.custom_fields };
    conn.execute(
        "UPDATE characters SET name=?1,gender=?2,age=?3,appearance=?4,personality=?5,background=?6,custom_fields=?7,updated_at=?8 WHERE id=?9",
        params![params.name, params.gender, params.age, params.appearance, params.personality, params.background, cf, now, character_id],
    ).map_err(|e| format!("更新角色失败: {}", e))?;
    let c = conn.query_row(
        "SELECT id, project_id, name, gender, age, appearance, personality, background, custom_fields, created_at, updated_at FROM characters WHERE id = ?1", [character_id],
        |row| Ok(Character {
            id: row.get(0)?, project_id: row.get(1)?, name: row.get(2)?, gender: row.get(3)?, age: row.get(4)?,
            appearance: row.get(5)?, personality: row.get(6)?, background: row.get(7)?, custom_fields: row.get(8)?,
            created_at: row.get(9)?, updated_at: row.get(10)?,
        }),
    ).map_err(|e| format!("查询角色失败: {}", e))?;
    Ok(c)
}

/// 删除角色
///
/// 从数据库中删除指定的角色记录。
///
/// # 参数
/// - `app_handle`: Tauri 应用句柄
/// - `character_id`: 角色 ID
///
/// # 返回值
/// 成功返回 Ok(())，失败返回错误信息
#[tauri::command]
pub async fn delete_character(app_handle: AppHandle, character_id: i64) -> Result<(), String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;
    conn.execute("DELETE FROM characters WHERE id = ?1", [character_id]).map_err(|e| format!("删除角色失败: {}", e))?;
    Ok(())
}

/// 列出项目的所有角色
///
/// 返回指定项目下所有角色列表，按创建时间倒序排列。
///
/// # 参数
/// - `app_handle`: Tauri 应用句柄
/// - `project_id`: 项目 ID
///
/// # 返回值
/// 成功返回角色列表，失败返回错误信息
#[tauri::command]
pub async fn list_characters(app_handle: AppHandle, project_id: i64) -> Result<Vec<Character>, String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;
    let mut stmt = conn.prepare("SELECT id, project_id, name, gender, age, appearance, personality, background, custom_fields, created_at, updated_at FROM characters WHERE project_id = ?1 ORDER BY created_at DESC")
        .map_err(|e| format!("查询失败: {}", e))?;
    let cs: Vec<Character> = stmt.query_map([project_id], |row| {
        Ok(Character {
            id: row.get(0)?, project_id: row.get(1)?, name: row.get(2)?, gender: row.get(3)?, age: row.get(4)?,
            appearance: row.get(5)?, personality: row.get(6)?, background: row.get(7)?, custom_fields: row.get(8)?,
            created_at: row.get(9)?, updated_at: row.get(10)?,
        })
    }).map_err(|e| format!("查询失败: {}", e))?.filter_map(|r| r.ok()).collect();
    Ok(cs)
}

// ============== Location ==============

/// 创建地点
///
/// 在项目中创建一个新的地点记录，用于世界观构建。
///
/// # 参数
/// - `app_handle`: Tauri 应用句柄
/// - `params`: 创建地点参数（包含 name, location_type, description, climate, population, notable_features）
///
/// # 返回值
/// 成功返回创建的地点记录，失败返回错误信息
#[tauri::command]
pub async fn create_location(app_handle: AppHandle, params: CreateLocationParams) -> Result<Location, String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;
    init_db(&conn).map_err(|e| format!("数据库初始化失败: {}", e))?;
    let now = chrono::Utc::now().to_rfc3339();
    let cf = if params.custom_fields.is_empty() { "{}".to_string() } else { params.custom_fields };
    conn.execute(
        "INSERT INTO locations (project_id, name, location_type, description, climate, population, notable_features, custom_fields, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![params.project_id, params.name, params.location_type, params.description, params.climate, params.population, params.notable_features, cf, now, now],
    ).map_err(|e| format!("创建地点失败: {}", e))?;
    let id = conn.last_insert_rowid();
    Ok(Location { id, project_id: params.project_id, name: params.name, location_type: params.location_type, description: params.description, climate: params.climate, population: params.population, notable_features: params.notable_features, custom_fields: cf, created_at: now.clone(), updated_at: now })
}

/// 更新地点信息
///
/// 更新指定地点的各项属性信息。
///
/// # 参数
/// - `app_handle`: Tauri 应用句柄
/// - `location_id`: 地点 ID
/// - `params`: 更新参数（包含 name, location_type, description, climate, population, notable_features）
///
/// # 返回值
/// 成功返回更新后的地点记录，失败返回错误信息
#[tauri::command]
pub async fn update_location(app_handle: AppHandle, location_id: i64, params: UpdateLocationParams) -> Result<Location, String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;
    let now = chrono::Utc::now().to_rfc3339();
    let cf = if params.custom_fields.is_empty() { "{}".to_string() } else { params.custom_fields };
    conn.execute(
        "UPDATE locations SET name=?1,location_type=?2,description=?3,climate=?4,population=?5,notable_features=?6,custom_fields=?7,updated_at=?8 WHERE id=?9",
        params![params.name, params.location_type, params.description, params.climate, params.population, params.notable_features, cf, now, location_id],
    ).map_err(|e| format!("更新地点失败: {}", e))?;
    let l = conn.query_row(
        "SELECT id, project_id, name, location_type, description, climate, population, notable_features, custom_fields, created_at, updated_at FROM locations WHERE id = ?1", [location_id],
        |row| Ok(Location {
            id: row.get(0)?, project_id: row.get(1)?, name: row.get(2)?, location_type: row.get(3)?, description: row.get(4)?,
            climate: row.get(5)?, population: row.get(6)?, notable_features: row.get(7)?, custom_fields: row.get(8)?,
            created_at: row.get(9)?, updated_at: row.get(10)?,
        }),
    ).map_err(|e| format!("查询地点失败: {}", e))?;
    Ok(l)
}

/// 删除地点
///
/// 从数据库中删除指定的地点记录。
///
/// # 参数
/// - `app_handle`: Tauri 应用句柄
/// - `location_id`: 地点 ID
///
/// # 返回值
/// 成功返回 Ok(())，失败返回错误信息
#[tauri::command]
pub async fn delete_location(app_handle: AppHandle, location_id: i64) -> Result<(), String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;
    conn.execute("DELETE FROM locations WHERE id = ?1", [location_id]).map_err(|e| format!("删除地点失败: {}", e))?;
    Ok(())
}

/// 列出项目的所有地点
///
/// 返回指定项目下所有地点列表，按创建时间倒序排列。
///
/// # 参数
/// - `app_handle`: Tauri 应用句柄
/// - `project_id`: 项目 ID
///
/// # 返回值
/// 成功返回地点列表，失败返回错误信息
#[tauri::command]
pub async fn list_locations(app_handle: AppHandle, project_id: i64) -> Result<Vec<Location>, String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;
    let mut stmt = conn.prepare("SELECT id, project_id, name, location_type, description, climate, population, notable_features, custom_fields, created_at, updated_at FROM locations WHERE project_id = ?1 ORDER BY created_at DESC")
        .map_err(|e| format!("查询失败: {}", e))?;
    let ls: Vec<Location> = stmt.query_map([project_id], |row| {
        Ok(Location { id: row.get(0)?, project_id: row.get(1)?, name: row.get(2)?, location_type: row.get(3)?, description: row.get(4)?, climate: row.get(5)?, population: row.get(6)?, notable_features: row.get(7)?, custom_fields: row.get(8)?, created_at: row.get(9)?, updated_at: row.get(10)? })
    }).map_err(|e| format!("查询失败: {}", e))?.filter_map(|r| r.ok()).collect();
    Ok(ls)
}

// ============== Organization ==============

/// 创建组织
///
/// 在项目中创建一个新的组织记录，用于世界观构建。
///
/// # 参数
/// - `app_handle`: Tauri 应用句柄
/// - `params`: 创建组织参数（包含 name, org_type, description, leader, headquarters, member_count）
///
/// # 返回值
/// 成功返回创建的组织记录，失败返回错误信息
#[tauri::command]
pub async fn create_organization(app_handle: AppHandle, params: CreateOrganizationParams) -> Result<Organization, String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;
    init_db(&conn).map_err(|e| format!("数据库初始化失败: {}", e))?;
    let now = chrono::Utc::now().to_rfc3339();
    let cf = if params.custom_fields.is_empty() { "{}".to_string() } else { params.custom_fields };
    conn.execute(
        "INSERT INTO organizations (project_id, name, org_type, description, leader, headquarters, member_count, custom_fields, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![params.project_id, params.name, params.org_type, params.description, params.leader, params.headquarters, params.member_count, cf, now, now],
    ).map_err(|e| format!("创建组织失败: {}", e))?;
    let id = conn.last_insert_rowid();
    Ok(Organization { id, project_id: params.project_id, name: params.name, org_type: params.org_type, description: params.description, leader: params.leader, headquarters: params.headquarters, member_count: params.member_count, custom_fields: cf, created_at: now.clone(), updated_at: now })
}

/// 更新组织信息
///
/// 更新指定组织的各项属性信息。
///
/// # 参数
/// - `app_handle`: Tauri 应用句柄
/// - `organization_id`: 组织 ID
/// - `params`: 更新参数（包含 name, org_type, description, leader, headquarters, member_count）
///
/// # 返回值
/// 成功返回更新后的组织记录，失败返回错误信息
#[tauri::command]
pub async fn update_organization(app_handle: AppHandle, organization_id: i64, params: UpdateOrganizationParams) -> Result<Organization, String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;
    let now = chrono::Utc::now().to_rfc3339();
    let cf = if params.custom_fields.is_empty() { "{}".to_string() } else { params.custom_fields };
    conn.execute(
        "UPDATE organizations SET name=?1,org_type=?2,description=?3,leader=?4,headquarters=?5,member_count=?6,custom_fields=?7,updated_at=?8 WHERE id=?9",
        params![params.name, params.org_type, params.description, params.leader, params.headquarters, params.member_count, cf, now, organization_id],
    ).map_err(|e| format!("更新组织失败: {}", e))?;
    let o = conn.query_row(
        "SELECT id, project_id, name, org_type, description, leader, headquarters, member_count, custom_fields, created_at, updated_at FROM organizations WHERE id = ?1", [organization_id],
        |row| Ok(Organization {
            id: row.get(0)?, project_id: row.get(1)?, name: row.get(2)?, org_type: row.get(3)?, description: row.get(4)?,
            leader: row.get(5)?, headquarters: row.get(6)?, member_count: row.get(7)?, custom_fields: row.get(8)?,
            created_at: row.get(9)?, updated_at: row.get(10)?,
        }),
    ).map_err(|e| format!("查询组织失败: {}", e))?;
    Ok(o)
}

/// 删除组织
///
/// 从数据库中删除指定的组织记录。
///
/// # 参数
/// - `app_handle`: Tauri 应用句柄
/// - `organization_id`: 组织 ID
///
/// # 返回值
/// 成功返回 Ok(())，失败返回错误信息
#[tauri::command]
pub async fn delete_organization(app_handle: AppHandle, organization_id: i64) -> Result<(), String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;
    conn.execute("DELETE FROM organizations WHERE id = ?1", [organization_id]).map_err(|e| format!("删除组织失败: {}", e))?;
    Ok(())
}

/// 列出项目的所有组织
///
/// 返回指定项目下所有组织列表，按创建时间倒序排列。
///
/// # 参数
/// - `app_handle`: Tauri 应用句柄
/// - `project_id`: 项目 ID
///
/// # 返回值
/// 成功返回组织列表，失败返回错误信息
#[tauri::command]
pub async fn list_organizations(app_handle: AppHandle, project_id: i64) -> Result<Vec<Organization>, String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;
    let mut stmt = conn.prepare("SELECT id, project_id, name, org_type, description, leader, headquarters, member_count, custom_fields, created_at, updated_at FROM organizations WHERE project_id = ?1 ORDER BY created_at DESC")
        .map_err(|e| format!("查询失败: {}", e))?;
    let os: Vec<Organization> = stmt.query_map([project_id], |row| {
        Ok(Organization { id: row.get(0)?, project_id: row.get(1)?, name: row.get(2)?, org_type: row.get(3)?, description: row.get(4)?, leader: row.get(5)?, headquarters: row.get(6)?, member_count: row.get(7)?, custom_fields: row.get(8)?, created_at: row.get(9)?, updated_at: row.get(10)? })
    }).map_err(|e| format!("查询失败: {}", e))?.filter_map(|r| r.ok()).collect();
    Ok(os)
}
