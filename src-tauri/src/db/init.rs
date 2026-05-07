use rusqlite::{Connection, Result as SqliteResult};
use std::path::PathBuf;
use tauri::AppHandle;

/// 获取主数据库文件路径
///
/// 从应用配置中获取 SQLite 数据库文件的绝对路径
///
/// # 参数
/// - `app_handle`: Tauri 应用句柄，用于访问应用资源和配置
///
/// # 返回值
/// 数据库文件的绝对路径
pub fn get_db_path(app_handle: &AppHandle) -> PathBuf {
    crate::config::get_db_path(app_handle)
}

/// 初始化数据库表结构
///
/// 按顺序创建所有必要的数据库表，确保表结构完整。
/// 如果表已存在则跳过创建，但会检查并添加缺失的列（向后兼容）。
///
/// # 参数
/// - `conn`: SQLite 数据库连接
///
/// # 返回值
/// 初始化成功返回 `Ok(())`，失败返回 `rusqlite::Error`
pub fn init_db(conn: &Connection) -> SqliteResult<()> {
    // 创建项目表
    create_projects_table(conn)?;
    // 创建卷表
    create_volumes_table(conn)?;
    // 创建章节表
    create_chapters_table(conn)?;
    // 创建写作目标表
    create_writing_goals_table(conn)?;
    // 创建写作记录表
    create_writing_records_table(conn)?;
    // 创建角色表
    create_characters_table(conn)?;
    // 创建地点表
    create_locations_table(conn)?;
    // 创建组织表
    create_organizations_table(conn)?;
    // 创建角色关系表
    create_relationships_table(conn)?;
    // 创建事件表
    create_events_table(conn)?;
    // 创建敏感词表
    create_sensitive_words_table(conn)?;
    // 创建备份表
    create_backups_table(conn)?;
    // 创建备份日志表
    create_backup_logs_table(conn)?;
    // 创建迁移日志表
    create_migration_logs_table(conn)?;
    // 创建专注会话表
    create_focus_sessions_table(conn)?;
    // 创建灵感条目表
    create_inspiration_items_table(conn)?;
    // 创建用户模板表
    create_user_templates_table(conn)?;

    Ok(())
}

/// 创建项目表
///
/// 存储小说项目的基本信息，包括书名、作者、路径等元数据。
/// 支持向后兼容：检查并添加缺失的 `last_opened_at` 和 `project_id` 列。
fn create_projects_table(conn: &Connection) -> SqliteResult<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS projects (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            project_id TEXT UNIQUE NOT NULL,
            name TEXT NOT NULL,
            author TEXT NOT NULL DEFAULT '',
            description TEXT NOT NULL DEFAULT '',
            path TEXT NOT NULL,
            created_at TEXT NOT NULL,
            last_opened_at TEXT
        )",
        [],
    )?;

    // 向后兼容：检查是否存在 last_opened_at 列，不存在则添加
    let has_last_opened: bool = conn
        .query_row(
            "SELECT COUNT(*) > 0 FROM pragma_table_info('projects') WHERE name='last_opened_at'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(false);

    if !has_last_opened {
        let _ = conn.execute("ALTER TABLE projects ADD COLUMN last_opened_at TEXT", []);
    }

    // 向后兼容：检查是否存在 project_id 列，不存在则添加
    let has_project_id: bool = conn
        .query_row(
            "SELECT COUNT(*) > 0 FROM pragma_table_info('projects') WHERE name='project_id'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(false);

    if !has_project_id {
        let _ = conn.execute("ALTER TABLE projects ADD COLUMN project_id TEXT", []);
    }

    Ok(())
}

/// 创建卷表
///
/// 存储小说的分卷信息，用于组织章节结构。
/// 外键关联 projects 表，项目删除时自动级联删除卷。
fn create_volumes_table(conn: &Connection) -> SqliteResult<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS volumes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            project_id INTEGER NOT NULL,
            name TEXT NOT NULL,
            sort_order INTEGER NOT NULL DEFAULT 0,
            FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
        )",
        [],
    )?;
    Ok(())
}

/// 创建章节表
///
/// 存储小说章节信息，每个章节对应一个 Markdown 文件。
/// 外键关联 volumes 表，卷删除时自动级联删除章节。
/// 支持向后兼容：检查并添加缺失的 `status` 列。
fn create_chapters_table(conn: &Connection) -> SqliteResult<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS chapters (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            volume_id INTEGER NOT NULL,
            title TEXT NOT NULL,
            file_path TEXT NOT NULL DEFAULT '',
            sort_order INTEGER NOT NULL DEFAULT 0,
            summary TEXT NOT NULL DEFAULT '',
            word_count_cache INTEGER NOT NULL DEFAULT 0,
            status TEXT NOT NULL DEFAULT 'draft',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (volume_id) REFERENCES volumes(id) ON DELETE CASCADE
        )",
        [],
    )?;

    // 向后兼容：检查是否存在 status 列，不存在则添加
    let has_status: bool = conn
        .query_row(
            "SELECT COUNT(*) > 0 FROM pragma_table_info('chapters') WHERE name='status'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(false);

    if !has_status {
        let _ = conn.execute(
            "ALTER TABLE chapters ADD COLUMN status TEXT NOT NULL DEFAULT 'draft'",
            [],
        );
    }

    Ok(())
}

/// 创建写作目标表
///
/// 存储项目的每日写作目标设置。
/// 每个项目只能有一条记录（project_id 唯一约束）。
fn create_writing_goals_table(conn: &Connection) -> SqliteResult<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS writing_goals (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            project_id INTEGER NOT NULL UNIQUE,
            daily_goal INTEGER NOT NULL DEFAULT 3000,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
        )",
        [],
    )?;
    Ok(())
}

/// 创建写作记录表
///
/// 存储每日写作统计数据，用于追踪写作进度。
/// 每个项目每天只能有一条记录（project_id + record_date 唯一约束）。
fn create_writing_records_table(conn: &Connection) -> SqliteResult<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS writing_records (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            project_id INTEGER NOT NULL,
            record_date TEXT NOT NULL,
            total_words INTEGER NOT NULL DEFAULT 0,
            duration INTEGER NOT NULL DEFAULT 0,
            UNIQUE(project_id, record_date),
            FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
        )",
        [],
    )?;
    Ok(())
}

/// 创建角色表
///
/// 存储小说角色的详细信息，包括外貌、性格、背景故事等。
/// custom_fields 字段用于存储用户自定义的额外属性（JSON 格式）。
fn create_characters_table(conn: &Connection) -> SqliteResult<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS characters (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            project_id INTEGER NOT NULL,
            name TEXT NOT NULL,
            gender TEXT NOT NULL DEFAULT '',
            age INTEGER,
            appearance TEXT NOT NULL DEFAULT '',
            personality TEXT NOT NULL DEFAULT '',
            background TEXT NOT NULL DEFAULT '',
            custom_fields TEXT NOT NULL DEFAULT '{}',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
        )",
        [],
    )?;
    Ok(())
}

/// 创建地点表
///
/// 存储小说中的地点信息，包括类型、气候、人口、显著特征等。
/// 支持自定义字段扩展。
fn create_locations_table(conn: &Connection) -> SqliteResult<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS locations (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            project_id INTEGER NOT NULL,
            name TEXT NOT NULL,
            location_type TEXT NOT NULL DEFAULT '',
            description TEXT NOT NULL DEFAULT '',
            climate TEXT NOT NULL DEFAULT '',
            population INTEGER,
            notable_features TEXT NOT NULL DEFAULT '',
            custom_fields TEXT NOT NULL DEFAULT '{}',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
        )",
        [],
    )?;
    Ok(())
}

/// 创建组织表
///
/// 存储小说中的组织/势力信息，包括类型、领袖、总部位置等。
fn create_organizations_table(conn: &Connection) -> SqliteResult<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS organizations (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            project_id INTEGER NOT NULL,
            name TEXT NOT NULL,
            org_type TEXT NOT NULL DEFAULT '',
            description TEXT NOT NULL DEFAULT '',
            leader TEXT NOT NULL DEFAULT '',
            headquarters TEXT NOT NULL DEFAULT '',
            member_count INTEGER,
            custom_fields TEXT NOT NULL DEFAULT '{}',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
        )",
        [],
    )?;
    Ok(())
}

/// 创建角色关系表
///
/// 存储角色之间的关系（如父子、师徒、朋友、敌人等）。
/// 外键关联 characters 表，角色删除时自动级联删除关系。
fn create_relationships_table(conn: &Connection) -> SqliteResult<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS relationships (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            project_id INTEGER NOT NULL,
            source_id INTEGER NOT NULL,
            target_id INTEGER NOT NULL,
            relation_type TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL,
            FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
            FOREIGN KEY (source_id) REFERENCES characters(id) ON DELETE CASCADE,
            FOREIGN KEY (target_id) REFERENCES characters(id) ON DELETE CASCADE
        )",
        [],
    )?;
    Ok(())
}

/// 创建事件表
///
/// 存储小说中的关键事件节点，用于时间线管理。
/// 可关联到具体章节，章节删除时关联设置为 NULL。
fn create_events_table(conn: &Connection) -> SqliteResult<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS events (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            project_id INTEGER NOT NULL,
            title TEXT NOT NULL DEFAULT '',
            story_time TEXT NOT NULL DEFAULT '',
            description TEXT NOT NULL DEFAULT '',
            chapter_id INTEGER,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
            FOREIGN KEY (chapter_id) REFERENCES chapters(id) ON DELETE SET NULL
        )",
        [],
    )?;
    Ok(())
}

/// 创建敏感词表
///
/// 存储项目自定义的敏感词列表，用于内容审查。
/// 每个项目的敏感词唯一（project_id + word 唯一约束）。
fn create_sensitive_words_table(conn: &Connection) -> SqliteResult<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS sensitive_words (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            project_id INTEGER NOT NULL,
            word TEXT NOT NULL,
            created_at TEXT NOT NULL,
            UNIQUE(project_id, word),
            FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
        )",
        [],
    )?;
    Ok(())
}

/// 创建备份表
///
/// 存储项目备份记录，包括备份类型、文件路径、大小等信息。
/// 支持全量备份（full）和增量备份（incremental）。
fn create_backups_table(conn: &Connection) -> SqliteResult<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS backups (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            project_id INTEGER NOT NULL,
            backup_type TEXT NOT NULL DEFAULT 'full',
            file_path TEXT NOT NULL,
            file_size INTEGER NOT NULL DEFAULT 0,
            git_commit TEXT,
            description TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'completed',
            FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
        )",
        [],
    )?;
    Ok(())
}

/// 创建备份日志表
///
/// 存储备份操作的详细日志，用于追踪备份过程和排查问题。
/// 日志级别包括 info、warning、error。
fn create_backup_logs_table(conn: &Connection) -> SqliteResult<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS backup_logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            project_id INTEGER NOT NULL,
            backup_id INTEGER,
            operation TEXT NOT NULL,
            message TEXT NOT NULL,
            level TEXT NOT NULL DEFAULT 'info',
            created_at TEXT NOT NULL,
            FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
            FOREIGN KEY (backup_id) REFERENCES backups(id) ON DELETE SET NULL
        )",
        [],
    )?;
    Ok(())
}

/// 创建迁移日志表
///
/// 存储项目迁移操作的记录，用于追踪旧版本项目迁移到新版本的过程。
/// 记录迁移前后的路径、项目ID和状态。
fn create_migration_logs_table(conn: &Connection) -> SqliteResult<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS migration_logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            operation TEXT NOT NULL,
            project_db_id INTEGER NOT NULL,
            old_project_id TEXT,
            new_project_id TEXT NOT NULL,
            old_path TEXT NOT NULL,
            new_path TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'success',
            error_message TEXT DEFAULT '',
            created_at TEXT NOT NULL
        )",
        [],
    )?;
    Ok(())
}

/// 创建专注会话表
///
/// 存储番茄钟专注会话记录，包括会话类型、时长、开始时间等。
/// 会话类型包括 work（工作）、short_break（短休息）、long_break（长休息）。
fn create_focus_sessions_table(conn: &Connection) -> SqliteResult<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS focus_sessions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            project_id INTEGER NOT NULL,
            session_type TEXT NOT NULL DEFAULT 'work',
            duration_minutes INTEGER NOT NULL,
            started_at TEXT NOT NULL,
            completed INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL,
            FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
        )",
        [],
    )?;
    Ok(())
}

/// 创建灵感条目表
///
/// 存储灵感看板的条目信息，支持多列管理灵感。
/// 支持向后兼容：检查并添加缺失的 `column_name` 列。
fn create_inspiration_items_table(conn: &Connection) -> SqliteResult<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS inspiration_items (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            project_id INTEGER NOT NULL,
            column_name TEXT NOT NULL DEFAULT '灵感',
            content TEXT NOT NULL DEFAULT '',
            sort_order INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
        )",
        [],
    )?;

    // 向后兼容：检查是否存在 column_name 列，不存在则添加
    let has_column_name: bool = conn
        .query_row(
            "SELECT COUNT(*) > 0 FROM pragma_table_info('inspiration_items') WHERE name='column_name'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(true);

    if !has_column_name {
        let _ = conn.execute(
            "ALTER TABLE inspiration_items ADD COLUMN column_name TEXT NOT NULL DEFAULT '灵感'",
            [],
        );
    }

    Ok(())
}

/// 创建用户模板表
///
/// 存储用户自定义的写作模板，包括名称、分类、内容等。
/// 用户可以基于模板快速创建章节内容。
fn create_user_templates_table(conn: &Connection) -> SqliteResult<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS user_templates (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            project_id INTEGER NOT NULL,
            name TEXT NOT NULL,
            description TEXT NOT NULL DEFAULT '',
            category TEXT NOT NULL DEFAULT '自定义',
            content TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
        )",
        [],
    )?;
    Ok(())
}
