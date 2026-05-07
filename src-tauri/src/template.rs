use crate::db::get_db_path;
use crate::models::{
    CreateUserTemplateParams, UpdateUserTemplateParams, UserTemplate, WritingTemplate,
};
use rusqlite::Connection;
use serde_json;
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

/// 获取内置模板文件路径
fn get_builtin_templates_path(app: &AppHandle) -> PathBuf {
    // 尝试从应用资源目录获取
    if let Ok(resource_dir) = app.path().resource_dir() {
        let template_path = resource_dir.join("resources/builtin_templates.json");
        if template_path.exists() {
            return template_path;
        }
    }

    // 回退到 src-tauri/resources 目录
    PathBuf::from("src-tauri/resources/builtin_templates.json")
}

/// 读取内置模板
#[tauri::command]
pub async fn get_builtin_templates(app_handle: AppHandle) -> Result<Vec<WritingTemplate>, String> {
    let path = get_builtin_templates_path(&app_handle);

    if !path.exists() {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(&path).map_err(|e| format!("读取内置模板文件失败: {}", e))?;

    let templates: Vec<WritingTemplate> =
        serde_json::from_str(&content).map_err(|e| format!("解析内置模板失败: {}", e))?;

    Ok(templates)
}

/// 获取用户自定义模板
#[tauri::command]
pub async fn get_user_templates(
    app_handle: AppHandle,
    project_id: i64,
) -> Result<Vec<UserTemplate>, String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;

    let mut stmt = conn
        .prepare(
            "SELECT id, project_id, name, description, category, content, created_at, updated_at 
         FROM user_templates 
         WHERE project_id = ?1 
         ORDER BY created_at DESC",
        )
        .map_err(|e| format!("准备查询失败: {}", e))?;

    let templates: Vec<UserTemplate> = stmt
        .query_map([project_id], |row| {
            Ok(UserTemplate {
                id: row.get(0)?,
                project_id: row.get(1)?,
                name: row.get(2)?,
                description: row.get(3)?,
                category: row.get(4)?,
                content: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })
        .map_err(|e| format!("查询用户模板失败: {}", e))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(templates)
}

/// 保存用户自定义模板
#[tauri::command]
pub async fn save_user_template(
    app_handle: AppHandle,
    params: CreateUserTemplateParams,
) -> Result<UserTemplate, String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;

    let now = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO user_templates (project_id, name, description, category, content, created_at, updated_at) 
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        (
            params.project_id,
            &params.name,
            &params.description,
            &params.category,
            &params.content,
            &now,
            &now,
        ),
    ).map_err(|e| format!("保存模板失败: {}", e))?;

    let id = conn.last_insert_rowid();

    Ok(UserTemplate {
        id,
        project_id: params.project_id,
        name: params.name,
        description: params.description,
        category: params.category,
        content: params.content,
        created_at: now.clone(),
        updated_at: now,
    })
}

/// 更新用户自定义模板
#[tauri::command]
pub async fn update_user_template(
    app_handle: AppHandle,
    template_id: i64,
    params: UpdateUserTemplateParams,
) -> Result<UserTemplate, String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;

    // 检查模板是否存在
    let exists: bool = conn
        .query_row(
            "SELECT COUNT(*) > 0 FROM user_templates WHERE id = ?1",
            [template_id],
            |row| row.get(0),
        )
        .unwrap_or(false);

    if !exists {
        return Err("模板不存在".to_string());
    }

    let now = chrono::Utc::now().to_rfc3339();

    // 构建动态更新语句
    let mut updates = Vec::new();
    let mut values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(name) = &params.name {
        updates.push("name = ?");
        values.push(Box::new(name.clone()));
    }
    if let Some(desc) = &params.description {
        updates.push("description = ?");
        values.push(Box::new(desc.clone()));
    }
    if let Some(cat) = &params.category {
        updates.push("category = ?");
        values.push(Box::new(cat.clone()));
    }
    if let Some(content) = &params.content {
        updates.push("content = ?");
        values.push(Box::new(content.clone()));
    }

    updates.push("updated_at = ?");
    values.push(Box::new(now.clone()));

    if !updates.is_empty() {
        let sql = format!(
            "UPDATE user_templates SET {} WHERE id = ?",
            updates.join(", ")
        );
        values.push(Box::new(template_id));

        conn.execute(&sql, rusqlite::params_from_iter(values.iter()))
            .map_err(|e| format!("更新模板失败: {}", e))?;
    }

    // 返回更新后的模板
    let template: UserTemplate = conn
        .query_row(
            "SELECT id, project_id, name, description, category, content, created_at, updated_at 
         FROM user_templates WHERE id = ?1",
            [template_id],
            |row| {
                Ok(UserTemplate {
                    id: row.get(0)?,
                    project_id: row.get(1)?,
                    name: row.get(2)?,
                    description: row.get(3)?,
                    category: row.get(4)?,
                    content: row.get(5)?,
                    created_at: row.get(6)?,
                    updated_at: row.get(7)?,
                })
            },
        )
        .map_err(|e| format!("获取更新后的模板失败: {}", e))?;

    Ok(template)
}

/// 删除用户自定义模板
#[tauri::command]
pub async fn delete_user_template(app_handle: AppHandle, template_id: i64) -> Result<(), String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;

    let rows_affected = conn
        .execute("DELETE FROM user_templates WHERE id = ?1", [template_id])
        .map_err(|e| format!("删除模板失败: {}", e))?;

    if rows_affected == 0 {
        return Err("模板不存在".to_string());
    }

    Ok(())
}

/// 获取所有模板（内置 + 用户自定义）
#[tauri::command]
pub async fn get_all_templates(
    app_handle: AppHandle,
    project_id: i64,
) -> Result<(Vec<WritingTemplate>, Vec<UserTemplate>), String> {
    let builtin = get_builtin_templates(app_handle.clone()).await?;
    let user = get_user_templates(app_handle, project_id).await?;
    Ok((builtin, user))
}
