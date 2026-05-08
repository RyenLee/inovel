use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumDefinition {
    pub category: String,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub sort_order: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumCategory {
    pub category: String,
    pub description: String,
}

pub const ENUM_CATEGORIES: &[(&str, &str)] = &[
    ("chapter_status", "章节状态"),
    ("session_type", "专注会话类型"),
    ("backup_status", "备份状态"),
    ("operation_result", "操作结果"),
    ("operation_category", "操作类别"),
    ("gender", "性别"),
    ("mention_type", "提及类型"),
    ("relationship_type", "关系类型"),
    ("event_type", "时间线事件类型"),
    ("template_category", "模板类别"),
    ("location_type", "地点类型"),
    ("organization_type", "组织类型"),
];

pub const CHAPTER_STATUS_ENUMS: &[(&str, &str, i32)] = &[
    ("outline", "大纲", 1),
    ("draft", "草稿", 2),
    ("revised", "修订", 3),
    ("final", "定稿", 4),
    ("abandoned", "废弃", 5),
];

pub const SESSION_TYPE_ENUMS: &[(&str, &str, i32)] = &[
    ("work", "工作", 1),
    ("short_break", "短休息", 2),
    ("long_break", "长休息", 3),
];

pub const BACKUP_STATUS_ENUMS: &[(&str, &str, i32)] = &[
    ("pending", "待处理", 1),
    ("completed", "已完成", 2),
    ("failed", "失败", 3),
];

pub const OPERATION_RESULT_ENUMS: &[(&str, &str, i32)] = &[
    ("success", "成功", 1),
    ("failed", "失败", 2),
    ("partial", "部分成功", 3),
];

pub const GENDER_ENUMS: &[(&str, &str, i32)] = &[
    ("male", "男", 1),
    ("female", "女", 2),
    ("other", "其他", 3),
    ("unknown", "未知", 4),
];

pub const LOCATION_TYPE_ENUMS: &[(&str, &str, i32)] = &[
    ("city", "城市", 1),
    ("town", "城镇", 2),
    ("village", "村庄", 3),
    ("building", "建筑", 4),
    ("region", "区域", 5),
    ("country", "国家", 6),
    ("kingdom", "王国", 7),
    ("mountain", "山脉", 8),
    ("forest", "森林", 9),
    ("ocean", "海洋", 10),
    ("other", "其他", 99),
];

pub const ORGANIZATION_TYPE_ENUMS: &[(&str, &str, i32)] = &[
    ("kingdom", "王国", 1),
    ("guild", "公会", 2),
    ("gang", "帮派", 3),
    ("cult", "教派", 4),
    ("company", "商会", 5),
    ("military", "军队", 6),
    ("secret_society", "秘密组织", 7),
    ("family", "家族", 8),
    ("church", "教会", 9),
    ("other", "其他", 99),
];

static DB_INIT: std::sync::Once = std::sync::Once::new();

pub fn ensure_enum_dictionary_table(conn: &Connection) -> Result<(), String> {
    DB_INIT.call_once(|| {
        let sql = r#"
            CREATE TABLE IF NOT EXISTS enum_dictionary (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                category TEXT NOT NULL,
                code TEXT NOT NULL,
                name TEXT NOT NULL,
                description TEXT,
                sort_order INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                UNIQUE(category, code)
            );

            CREATE INDEX IF NOT EXISTS idx_enum_dict_category ON enum_dictionary(category);
            CREATE INDEX IF NOT EXISTS idx_enum_dict_code ON enum_dictionary(code);
        "#;
        let _ = conn.execute_batch(sql);
    });
    Ok(())
}

pub fn init_enum_dictionary(app_handle: &tauri::AppHandle) -> Result<(), String> {
    use crate::config::get_db_path;

    let db_path = get_db_path(app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("打开数据库失败: {}", e))?;

    ensure_enum_dictionary_table(&conn)?;

    // 检查是否已有数据
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM enum_dictionary", [], |row| row.get(0))
        .unwrap_or(0);

    if count > 0 {
        return Ok(()); // 已有数据，无需初始化
    }

    // 插入所有枚举值
    insert_enums(conn)?;

    Ok(())
}

fn insert_enums(conn: Connection) -> Result<(), String> {
    // 插入章节状态
    for (code, name, order) in CHAPTER_STATUS_ENUMS {
        conn.execute(
            "INSERT OR IGNORE INTO enum_dictionary (category, code, name, sort_order) VALUES (?1, ?2, ?3, ?4)",
            ("chapter_status", code, name, order),
        )
        .map_err(|e| format!("插入枚举失败: {}", e))?;
    }

    // 插入专注会话类型
    for (code, name, order) in SESSION_TYPE_ENUMS {
        conn.execute(
            "INSERT OR IGNORE INTO enum_dictionary (category, code, name, sort_order) VALUES (?1, ?2, ?3, ?4)",
            ("session_type", code, name, order),
        )
        .map_err(|e| format!("插入枚举失败: {}", e))?;
    }

    // 插入备份状态
    for (code, name, order) in BACKUP_STATUS_ENUMS {
        conn.execute(
            "INSERT OR IGNORE INTO enum_dictionary (category, code, name, sort_order) VALUES (?1, ?2, ?3, ?4)",
            ("backup_status", code, name, order),
        )
        .map_err(|e| format!("插入枚举失败: {}", e))?;
    }

    // 插入操作结果
    for (code, name, order) in OPERATION_RESULT_ENUMS {
        conn.execute(
            "INSERT OR IGNORE INTO enum_dictionary (category, code, name, sort_order) VALUES (?1, ?2, ?3, ?4)",
            ("operation_result", code, name, order),
        )
        .map_err(|e| format!("插入枚举失败: {}", e))?;
    }

    // 插入性别
    for (code, name, order) in GENDER_ENUMS {
        conn.execute(
            "INSERT OR IGNORE INTO enum_dictionary (category, code, name, sort_order) VALUES (?1, ?2, ?3, ?4)",
            ("gender", code, name, order),
        )
        .map_err(|e| format!("插入枚举失败: {}", e))?;
    }

    // 插入地点类型
    for (code, name, order) in LOCATION_TYPE_ENUMS {
        conn.execute(
            "INSERT OR IGNORE INTO enum_dictionary (category, code, name, sort_order) VALUES (?1, ?2, ?3, ?4)",
            ("location_type", code, name, order),
        )
        .map_err(|e| format!("插入枚举失败: {}", e))?;
    }

    // 插入组织类型
    for (code, name, order) in ORGANIZATION_TYPE_ENUMS {
        conn.execute(
            "INSERT OR IGNORE INTO enum_dictionary (category, code, name, sort_order) VALUES (?1, ?2, ?3, ?4)",
            ("organization_type", code, name, order),
        )
        .map_err(|e| format!("插入枚举失败: {}", e))?;
    }

    // 插入提及类型
    let mention_types = vec![
        ("character", "人物", 1),
        ("location", "地点", 2),
        ("organization", "组织", 3),
    ];
    for (code, name, order) in mention_types {
        conn.execute(
            "INSERT OR IGNORE INTO enum_dictionary (category, code, name, sort_order) VALUES (?1, ?2, ?3, ?4)",
            ("mention_type", code, name, order),
        )
        .map_err(|e| format!("插入枚举失败: {}", e))?;
    }

    // 插入操作类别
    let operation_categories = vec![
        ("project", "项目管理", 1),
        ("chapter", "章节管理", 2),
        ("volume", "卷管理", 3),
        ("writing", "写作", 4),
        ("worldbuilding", "世界观", 5),
        ("character", "角色管理", 6),
        ("relationship", "关系管理", 7),
        ("timeline", "时间线", 8),
        ("export", "导出", 9),
        ("backup", "备份", 10),
        ("encryption", "加密", 11),
        ("template", "模板", 12),
        ("settings", "设置", 13),
        ("focus", "专注", 14),
        ("stats", "统计", 15),
    ];
    for (code, name, order) in operation_categories {
        conn.execute(
            "INSERT OR IGNORE INTO enum_dictionary (category, code, name, sort_order) VALUES (?1, ?2, ?3, ?4)",
            ("operation_category", code, name, order),
        )
        .map_err(|e| format!("插入枚举失败: {}", e))?;
    }

    Ok(())
}

pub fn get_enums_by_category(
    conn: &Connection,
    category: &str,
) -> Result<Vec<EnumDefinition>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT category, code, name, description, sort_order
             FROM enum_dictionary
             WHERE category = ?1
             ORDER BY sort_order",
        )
        .map_err(|e| format!("查询准备失败: {}", e))?;

    let enums = stmt
        .query_map([category], |row| {
            Ok(EnumDefinition {
                category: row.get(0)?,
                code: row.get(1)?,
                name: row.get(2)?,
                description: row.get(3)?,
                sort_order: row.get(4)?,
            })
        })
        .map_err(|e| format!("查询失败: {}", e))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(enums)
}

pub fn get_all_enums(conn: &Connection) -> Result<HashMap<String, Vec<EnumDefinition>>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT category, code, name, description, sort_order
             FROM enum_dictionary
             ORDER BY category, sort_order",
        )
        .map_err(|e| format!("查询准备失败: {}", e))?;

    let enums: Vec<EnumDefinition> = stmt
        .query_map([], |row| {
            Ok(EnumDefinition {
                category: row.get(0)?,
                code: row.get(1)?,
                name: row.get(2)?,
                description: row.get(3)?,
                sort_order: row.get(4)?,
            })
        })
        .map_err(|e| format!("查询失败: {}", e))?
        .filter_map(|r| r.ok())
        .collect();

    let mut result: HashMap<String, Vec<EnumDefinition>> = HashMap::new();
    for enum_def in enums {
        result
            .entry(enum_def.category.clone())
            .or_default()
            .push(enum_def);
    }

    Ok(result)
}

pub fn get_enum_name(conn: &Connection, category: &str, code: &str) -> Result<String, String> {
    conn.query_row(
        "SELECT name FROM enum_dictionary WHERE category = ?1 AND code = ?2",
        [category, code],
        |row| row.get(0),
    )
    .map_err(|e| format!("查询枚举名称失败: {}", e))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationContext {
    pub user_id: String,
    pub project_id: Option<i64>,
    pub ip_address: Option<String>,
}

impl Default for OperationContext {
    fn default() -> Self {
        Self {
            user_id: "system".to_string(),
            project_id: None,
            ip_address: None,
        }
    }
}
