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
/// 此函数包含所有系统所需的表初始化，包括：
/// - 项目管理相关表
/// - 写作相关表
/// - 世界观相关表
/// - 备份与迁移相关表
/// - 操作日志相关表
/// - 枚举字典相关表
///
/// # 参数
/// - `conn`: SQLite 数据库连接
///
/// # 返回值
/// 初始化成功返回 `Ok(())`，失败返回 `rusqlite::Error`
pub fn init_db(conn: &Connection) -> SqliteResult<()> {
    // 创建表注释元数据表
    create_table_comments_table(conn)?;

    // 创建项目管理相关表
    create_projects_table(conn)?;
    create_volumes_table(conn)?;
    create_chapters_table(conn)?;

    // 创建写作相关表
    create_writing_goals_table(conn)?;
    create_writing_records_table(conn)?;
    create_focus_sessions_table(conn)?;

    // 创建世界观相关表
    create_characters_table(conn)?;
    create_locations_table(conn)?;
    create_organizations_table(conn)?;
    create_relationships_table(conn)?;
    create_events_table(conn)?;

    // 创建内容与模板相关表
    create_sensitive_words_table(conn)?;
    create_user_templates_table(conn)?;
    create_inspiration_items_table(conn)?;

    // 创建备份与迁移相关表
    create_backups_table(conn)?;
    create_backup_logs_table(conn)?;
    create_migration_logs_table(conn)?;

    // 创建操作日志与统计相关表
    create_operation_logs_table(conn)?;
    create_operation_log_stats_table(conn)?;

    // 创建枚举字典表
    create_enum_dictionary_table(conn)?;

    // 初始化枚举字典数据（如果不存在）
    init_enum_dictionary_data(conn)?;

    // 初始化表注释数据（如果不存在）
    init_table_comments_data(conn)?;

    Ok(())
}

// ============================================================================
// 项目管理相关表创建函数
// ============================================================================

/// 创建项目表
///
/// 存储小说项目的基本信息，包括书名、作者、路径等元数据。
/// 支持向后兼容：检查并添加缺失的 `last_opened_at` 和 `project_id` 列。
///
/// **表结构说明：**
/// | 字段名 | 类型 | 约束 | 说明 |
/// |--------|------|------|------|
/// | id | INTEGER | PRIMARY KEY AUTOINCREMENT | 数据库主键，自增ID |
/// | project_id | TEXT | UNIQUE NOT NULL | 项目唯一标识符（6位字母数字组合），用于文件夹命名 |
/// | name | TEXT | NOT NULL | 项目名称（书名） |
/// | author | TEXT | NOT NULL DEFAULT '' | 作者名称 |
/// | description | TEXT | NOT NULL DEFAULT '' | 项目描述/简介 |
/// | path | TEXT | NOT NULL | 项目文件夹在本地的绝对路径 |
/// | created_at | TEXT | NOT NULL | 创建时间（RFC3339格式） |
/// | last_opened_at | TEXT | NULL | 最后打开时间（RFC3339格式） |
fn create_projects_table(conn: &Connection) -> SqliteResult<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS projects (
            id INTEGER PRIMARY KEY AUTOINCREMENT,    -- 数据库主键，自增ID
            project_id TEXT UNIQUE NOT NULL,          -- 项目唯一标识符（6位字母数字组合）
            name TEXT NOT NULL,                       -- 项目名称（书名）
            author TEXT NOT NULL DEFAULT '',          -- 作者名称
            description TEXT NOT NULL DEFAULT '',     -- 项目描述/简介
            path TEXT NOT NULL,                       -- 项目文件夹绝对路径
            created_at TEXT NOT NULL,                 -- 创建时间（RFC3339格式）
            last_opened_at TEXT                       -- 最后打开时间（RFC3339格式）
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
///
/// **表结构说明：**
/// | 字段名 | 类型 | 约束 | 说明 |
/// |--------|------|------|------|
/// | id | INTEGER | PRIMARY KEY AUTOINCREMENT | 数据库主键，自增ID |
/// | project_id | INTEGER | NOT NULL, FK | 所属项目ID，关联 projects(id) |
/// | name | TEXT | NOT NULL | 卷名称（如：第一卷、上部等） |
/// | sort_order | INTEGER | NOT NULL DEFAULT 0 | 排序序号，用于卷的顺序排列 |
fn create_volumes_table(conn: &Connection) -> SqliteResult<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS volumes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,    -- 数据库主键，自增ID
            project_id INTEGER NOT NULL,             -- 所属项目ID
            name TEXT NOT NULL,                      -- 卷名称
            sort_order INTEGER NOT NULL DEFAULT 0,   -- 排序序号
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
///
/// **表结构说明：**
/// | 字段名 | 类型 | 约束 | 说明 |
/// |--------|------|------|------|
/// | id | INTEGER | PRIMARY KEY AUTOINCREMENT | 数据库主键，自增ID |
/// | volume_id | INTEGER | NOT NULL, FK | 所属卷ID，关联 volumes(id) |
/// | title | TEXT | NOT NULL | 章节标题 |
/// | file_path | TEXT | NOT NULL DEFAULT '' | 章节对应的 Markdown 文件相对路径 |
/// | sort_order | INTEGER | NOT NULL DEFAULT 0 | 排序序号，用于章节顺序排列 |
/// | summary | TEXT | NOT NULL DEFAULT '' | 章节摘要/大纲 |
/// | word_count_cache | INTEGER | NOT NULL DEFAULT 0 | 字数缓存（避免每次计算） |
/// | status | TEXT | NOT NULL DEFAULT 'draft' | 章节状态：outline(大纲)/draft(草稿)/revised(修订)/final(定稿)/abandoned(废弃) |
/// | created_at | TEXT | NOT NULL | 创建时间（RFC3339格式） |
/// | updated_at | TEXT | NOT NULL | 最后更新时间（RFC3339格式） |
fn create_chapters_table(conn: &Connection) -> SqliteResult<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS chapters (
            id INTEGER PRIMARY KEY AUTOINCREMENT,        -- 数据库主键，自增ID
            volume_id INTEGER NOT NULL,                   -- 所属卷ID
            title TEXT NOT NULL,                          -- 章节标题
            file_path TEXT NOT NULL DEFAULT '',           -- Markdown文件相对路径
            sort_order INTEGER NOT NULL DEFAULT 0,        -- 排序序号
            summary TEXT NOT NULL DEFAULT '',             -- 章节摘要/大纲
            word_count_cache INTEGER NOT NULL DEFAULT 0,  -- 字数缓存
            status TEXT NOT NULL DEFAULT 'draft',         -- 章节状态
            created_at TEXT NOT NULL,                     -- 创建时间
            updated_at TEXT NOT NULL,                     -- 最后更新时间
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

// ============================================================================
// 写作相关表创建函数
// ============================================================================

/// 创建写作目标表
///
/// 存储项目的每日写作目标设置。
/// 每个项目只能有一条记录（project_id 唯一约束）。
///
/// **表结构说明：**
/// | 字段名 | 类型 | 约束 | 说明 |
/// |--------|------|------|------|
/// | id | INTEGER | PRIMARY KEY AUTOINCREMENT | 数据库主键，自增ID |
/// | project_id | INTEGER | NOT NULL UNIQUE, FK | 所属项目ID，关联 projects(id) |
/// | daily_goal | INTEGER | NOT NULL DEFAULT 3000 | 每日写作目标字数（默认3000字） |
/// | updated_at | TEXT | NOT NULL | 最后更新时间（RFC3339格式） |
fn create_writing_goals_table(conn: &Connection) -> SqliteResult<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS writing_goals (
            id INTEGER PRIMARY KEY AUTOINCREMENT,      -- 数据库主键，自增ID
            project_id INTEGER NOT NULL UNIQUE,         -- 所属项目ID
            daily_goal INTEGER NOT NULL DEFAULT 3000,  -- 每日写作目标字数
            updated_at TEXT NOT NULL,                   -- 最后更新时间
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
///
/// **表结构说明：**
/// | 字段名 | 类型 | 约束 | 说明 |
/// |--------|------|------|------|
/// | id | INTEGER | PRIMARY KEY AUTOINCREMENT | 数据库主键，自增ID |
/// | project_id | INTEGER | NOT NULL, FK | 所属项目ID，关联 projects(id) |
/// | record_date | TEXT | NOT NULL | 记录日期（格式：YYYY-MM-DD） |
/// | total_words | INTEGER | NOT NULL DEFAULT 0 | 当日写作总字数 |
/// | duration | INTEGER | NOT NULL DEFAULT 0 | 当日写作时长（分钟） |
fn create_writing_records_table(conn: &Connection) -> SqliteResult<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS writing_records (
            id INTEGER PRIMARY KEY AUTOINCREMENT,      -- 数据库主键，自增ID
            project_id INTEGER NOT NULL,               -- 所属项目ID
            record_date TEXT NOT NULL,                 -- 记录日期（YYYY-MM-DD）
            total_words INTEGER NOT NULL DEFAULT 0,    -- 当日写作总字数
            duration INTEGER NOT NULL DEFAULT 0,       -- 当日写作时长（分钟）
            UNIQUE(project_id, record_date),
            FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
        )",
        [],
    )?;
    Ok(())
}

/// 创建专注会话表（番茄钟）
///
/// 存储番茄钟专注会话记录，包括会话类型、时长、开始时间等。
/// 会话类型包括 work（工作）、short_break（短休息）、long_break（长休息）。
///
/// **表结构说明：**
/// | 字段名 | 类型 | 约束 | 说明 |
/// |--------|------|------|------|
/// | id | INTEGER | PRIMARY KEY AUTOINCREMENT | 数据库主键，自增ID |
/// | project_id | INTEGER | NOT NULL, FK | 所属项目ID，关联 projects(id) |
/// | session_type | TEXT | NOT NULL DEFAULT 'work' | 会话类型：work(工作)/short_break(短休息)/long_break(长休息) |
/// | duration_minutes | INTEGER | NOT NULL | 会话时长（分钟） |
/// | started_at | TEXT | NOT NULL | 开始时间（RFC3339格式） |
/// | completed | INTEGER | NOT NULL DEFAULT 1 | 是否完成：1=完成，0=未完成（中途退出） |
/// | created_at | TEXT | NOT NULL | 创建时间（RFC3339格式） |
fn create_focus_sessions_table(conn: &Connection) -> SqliteResult<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS focus_sessions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,         -- 数据库主键，自增ID
            project_id INTEGER NOT NULL,                  -- 所属项目ID
            session_type TEXT NOT NULL DEFAULT 'work',    -- 会话类型
            duration_minutes INTEGER NOT NULL,            -- 会话时长（分钟）
            started_at TEXT NOT NULL,                     -- 开始时间
            completed INTEGER NOT NULL DEFAULT 1,         -- 是否完成
            created_at TEXT NOT NULL,                     -- 创建时间
            FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
        )",
        [],
    )?;
    Ok(())
}

// ============================================================================
// 世界观相关表创建函数
// ============================================================================

/// 创建角色表
///
/// 存储小说角色的详细信息，包括外貌、性格、背景故事等。
/// custom_fields 字段用于存储用户自定义的额外属性（JSON 格式）。
///
/// **表结构说明：**
/// | 字段名 | 类型 | 约束 | 说明 |
/// |--------|------|------|------|
/// | id | INTEGER | PRIMARY KEY AUTOINCREMENT | 数据库主键，自增ID |
/// | project_id | INTEGER | NOT NULL, FK | 所属项目ID，关联 projects(id) |
/// | name | TEXT | NOT NULL | 角色名称 |
/// | gender | TEXT | NOT NULL DEFAULT '' | 性别：male(男)/female(女)/other(其他)/unknown(未知) |
/// | age | INTEGER | NULL | 年龄 |
/// | appearance | TEXT | NOT NULL DEFAULT '' | 外貌描述 |
/// | personality | TEXT | NOT NULL DEFAULT '' | 性格描述 |
/// | background | TEXT | NOT NULL DEFAULT '' | 背景故事 |
/// | custom_fields | TEXT | NOT NULL DEFAULT '{}' | 自定义字段（JSON格式） |
/// | created_at | TEXT | NOT NULL | 创建时间（RFC3339格式） |
/// | updated_at | TEXT | NOT NULL | 最后更新时间（RFC3339格式） |
fn create_characters_table(conn: &Connection) -> SqliteResult<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS characters (
            id INTEGER PRIMARY KEY AUTOINCREMENT,       -- 数据库主键，自增ID
            project_id INTEGER NOT NULL,                -- 所属项目ID
            name TEXT NOT NULL,                         -- 角色名称
            gender TEXT NOT NULL DEFAULT '',            -- 性别
            age INTEGER,                                -- 年龄
            appearance TEXT NOT NULL DEFAULT '',        -- 外貌描述
            personality TEXT NOT NULL DEFAULT '',       -- 性格描述
            background TEXT NOT NULL DEFAULT '',        -- 背景故事
            custom_fields TEXT NOT NULL DEFAULT '{}',   -- 自定义字段（JSON）
            created_at TEXT NOT NULL,                   -- 创建时间
            updated_at TEXT NOT NULL,                   -- 最后更新时间
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
///
/// **表结构说明：**
/// | 字段名 | 类型 | 约束 | 说明 |
/// |--------|------|------|------|
/// | id | INTEGER | PRIMARY KEY AUTOINCREMENT | 数据库主键，自增ID |
/// | project_id | INTEGER | NOT NULL, FK | 所属项目ID，关联 projects(id) |
/// | name | TEXT | NOT NULL | 地点名称 |
/// | location_type | TEXT | NOT NULL DEFAULT '' | 地点类型：city(城市)/town(城镇)/village(村庄)/building(建筑)/region(区域)/country(国家)/kingdom(王国)/mountain(山脉)/forest(森林)/ocean(海洋)/other(其他) |
/// | description | TEXT | NOT NULL DEFAULT '' | 地点描述 |
/// | climate | TEXT | NOT NULL DEFAULT '' | 气候特征 |
/// | population | INTEGER | NULL | 人口数量 |
/// | notable_features | TEXT | NOT NULL DEFAULT '' | 显著特征 |
/// | custom_fields | TEXT | NOT NULL DEFAULT '{}' | 自定义字段（JSON格式） |
/// | created_at | TEXT | NOT NULL | 创建时间（RFC3339格式） |
/// | updated_at | TEXT | NOT NULL | 最后更新时间（RFC3339格式） |
fn create_locations_table(conn: &Connection) -> SqliteResult<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS locations (
            id INTEGER PRIMARY KEY AUTOINCREMENT,       -- 数据库主键，自增ID
            project_id INTEGER NOT NULL,                -- 所属项目ID
            name TEXT NOT NULL,                         -- 地点名称
            location_type TEXT NOT NULL DEFAULT '',     -- 地点类型
            description TEXT NOT NULL DEFAULT '',       -- 地点描述
            climate TEXT NOT NULL DEFAULT '',           -- 气候特征
            population INTEGER,                         -- 人口数量
            notable_features TEXT NOT NULL DEFAULT '',  -- 显著特征
            custom_fields TEXT NOT NULL DEFAULT '{}',   -- 自定义字段（JSON）
            created_at TEXT NOT NULL,                   -- 创建时间
            updated_at TEXT NOT NULL,                   -- 最后更新时间
            FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
        )",
        [],
    )?;
    Ok(())
}

/// 创建组织表
///
/// 存储小说中的组织/势力信息，包括类型、领袖、总部位置等。
///
/// **表结构说明：**
/// | 字段名 | 类型 | 约束 | 说明 |
/// |--------|------|------|------|
/// | id | INTEGER | PRIMARY KEY AUTOINCREMENT | 数据库主键，自增ID |
/// | project_id | INTEGER | NOT NULL, FK | 所属项目ID，关联 projects(id) |
/// | name | TEXT | NOT NULL | 组织名称 |
/// | org_type | TEXT | NOT NULL DEFAULT '' | 组织类型 |
/// | description | TEXT | NOT NULL DEFAULT '' | 组织描述 |
/// | leader | TEXT | NOT NULL DEFAULT '' | 领袖名称 |
/// | headquarters | TEXT | NOT NULL DEFAULT '' | 总部位置 |
/// | member_count | INTEGER | NULL | 成员数量 |
/// | custom_fields | TEXT | NOT NULL DEFAULT '{}' | 自定义字段（JSON格式） |
/// | created_at | TEXT | NOT NULL | 创建时间（RFC3339格式） |
/// | updated_at | TEXT | NOT NULL | 最后更新时间（RFC3339格式） |
fn create_organizations_table(conn: &Connection) -> SqliteResult<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS organizations (
            id INTEGER PRIMARY KEY AUTOINCREMENT,       -- 数据库主键，自增ID
            project_id INTEGER NOT NULL,                -- 所属项目ID
            name TEXT NOT NULL,                         -- 组织名称
            org_type TEXT NOT NULL DEFAULT '',          -- 组织类型
            description TEXT NOT NULL DEFAULT '',       -- 组织描述
            leader TEXT NOT NULL DEFAULT '',            -- 领袖名称
            headquarters TEXT NOT NULL DEFAULT '',      -- 总部位置
            member_count INTEGER,                       -- 成员数量
            custom_fields TEXT NOT NULL DEFAULT '{}',   -- 自定义字段（JSON）
            created_at TEXT NOT NULL,                   -- 创建时间
            updated_at TEXT NOT NULL,                   -- 最后更新时间
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
///
/// **表结构说明：**
/// | 字段名 | 类型 | 约束 | 说明 |
/// |--------|------|------|------|
/// | id | INTEGER | PRIMARY KEY AUTOINCREMENT | 数据库主键，自增ID |
/// | project_id | INTEGER | NOT NULL, FK | 所属项目ID，关联 projects(id) |
/// | source_id | INTEGER | NOT NULL, FK | 关系源角色ID，关联 characters(id) |
/// | target_id | INTEGER | NOT NULL, FK | 关系目标角色ID，关联 characters(id) |
/// | relation_type | TEXT | NOT NULL DEFAULT '' | 关系类型：父子、师徒、朋友、敌人等 |
/// | created_at | TEXT | NOT NULL | 创建时间（RFC3339格式） |
fn create_relationships_table(conn: &Connection) -> SqliteResult<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS relationships (
            id INTEGER PRIMARY KEY AUTOINCREMENT,       -- 数据库主键，自增ID
            project_id INTEGER NOT NULL,                -- 所属项目ID
            source_id INTEGER NOT NULL,                 -- 关系源角色ID
            target_id INTEGER NOT NULL,                 -- 关系目标角色ID
            relation_type TEXT NOT NULL DEFAULT '',     -- 关系类型
            created_at TEXT NOT NULL,                   -- 创建时间
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
///
/// **表结构说明：**
/// | 字段名 | 类型 | 约束 | 说明 |
/// |--------|------|------|------|
/// | id | INTEGER | PRIMARY KEY AUTOINCREMENT | 数据库主键，自增ID |
/// | project_id | INTEGER | NOT NULL, FK | 所属项目ID，关联 projects(id) |
/// | title | TEXT | NOT NULL DEFAULT '' | 事件标题 |
/// | story_time | TEXT | NOT NULL DEFAULT '' | 故事时间（事件在故事中的发生时间） |
/// | description | TEXT | NOT NULL DEFAULT '' | 事件描述 |
/// | chapter_id | INTEGER | NULL, FK | 关联章节ID，关联 chapters(id) |
/// | created_at | TEXT | NOT NULL | 创建时间（RFC3339格式） |
/// | updated_at | TEXT | NOT NULL | 最后更新时间（RFC3339格式） |
fn create_events_table(conn: &Connection) -> SqliteResult<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS events (
            id INTEGER PRIMARY KEY AUTOINCREMENT,         -- 数据库主键，自增ID
            project_id INTEGER NOT NULL,                  -- 所属项目ID
            title TEXT NOT NULL DEFAULT '',               -- 事件标题
            story_time TEXT NOT NULL DEFAULT '',          -- 故事时间
            description TEXT NOT NULL DEFAULT '',         -- 事件描述
            chapter_id INTEGER,                           -- 关联章节ID
            created_at TEXT NOT NULL,                     -- 创建时间
            updated_at TEXT NOT NULL,                     -- 最后更新时间
            FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
            FOREIGN KEY (chapter_id) REFERENCES chapters(id) ON DELETE SET NULL
        )",
        [],
    )?;
    Ok(())
}

// ============================================================================
// 内容与模板相关表创建函数
// ============================================================================

/// 创建敏感词表
///
/// 存储项目自定义的敏感词列表，用于内容审查。
/// 每个项目的敏感词唯一（project_id + word 唯一约束）。
///
/// **表结构说明：**
/// | 字段名 | 类型 | 约束 | 说明 |
/// |--------|------|------|------|
/// | id | INTEGER | PRIMARY KEY AUTOINCREMENT | 数据库主键，自增ID |
/// | project_id | INTEGER | NOT NULL, FK | 所属项目ID，关联 projects(id) |
/// | word | TEXT | NOT NULL | 敏感词内容 |
/// | created_at | TEXT | NOT NULL | 创建时间（RFC3339格式） |
fn create_sensitive_words_table(conn: &Connection) -> SqliteResult<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS sensitive_words (
            id INTEGER PRIMARY KEY AUTOINCREMENT,      -- 数据库主键，自增ID
            project_id INTEGER NOT NULL,               -- 所属项目ID
            word TEXT NOT NULL,                        -- 敏感词内容
            created_at TEXT NOT NULL,                  -- 创建时间
            UNIQUE(project_id, word),
            FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
        )",
        [],
    )?;
    Ok(())
}

/// 创建用户模板表
///
/// 存储用户自定义的写作模板，包括名称、分类、内容等。
/// 用户可以基于模板快速创建章节内容。
///
/// **表结构说明：**
/// | 字段名 | 类型 | 约束 | 说明 |
/// |--------|------|------|------|
/// | id | INTEGER | PRIMARY KEY AUTOINCREMENT | 数据库主键，自增ID |
/// | project_id | INTEGER | NOT NULL DEFAULT 0 | 所属项目ID（0表示全局模板） |
/// | name | TEXT | NOT NULL | 模板名称 |
/// | description | TEXT | NOT NULL DEFAULT '' | 模板描述 |
/// | category | TEXT | NOT NULL DEFAULT '自定义' | 模板分类 |
/// | content | TEXT | NOT NULL DEFAULT '' | 模板内容（Markdown格式） |
/// | created_at | TEXT | NOT NULL | 创建时间（RFC3339格式） |
/// | updated_at | TEXT | NOT NULL | 最后更新时间（RFC3339格式） |
fn create_user_templates_table(conn: &Connection) -> SqliteResult<()> {
    // 首先检查表是否存在
    let table_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='user_templates'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(false);

    if !table_exists {
        // 表不存在，直接创建
        conn.execute(
            "CREATE TABLE user_templates (
                id INTEGER PRIMARY KEY AUTOINCREMENT,        -- 数据库主键，自增ID
                project_id INTEGER NOT NULL DEFAULT 0,        -- 所属项目ID（0表示全局模板）
                name TEXT NOT NULL,                          -- 模板名称
                description TEXT NOT NULL DEFAULT '',         -- 模板描述
                category TEXT NOT NULL DEFAULT '自定义',      -- 模板分类
                content TEXT NOT NULL DEFAULT '',             -- 模板内容（Markdown）
                created_at TEXT NOT NULL,                     -- 创建时间
                updated_at TEXT NOT NULL                      -- 最后更新时间
            )",
            [],
        )?;
    } else {
        // 表已存在，进行向后兼容迁移
        // 检查是否有 project_id 的 DEFAULT 值，以及是否有外键约束
        let has_project_id_default: bool = conn
            .query_row(
                "SELECT dflt_value IS NOT NULL FROM pragma_table_info('user_templates') WHERE name='project_id'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(false);

        if !has_project_id_default {
            // 需要迁移表结构
            let transaction = conn.unchecked_transaction()?;
            transaction.execute("PRAGMA foreign_keys = OFF", [])?;
            // 1. 重命名旧表
            transaction.execute(
                "ALTER TABLE user_templates RENAME TO user_templates_old",
                [],
            )?;
            // 2. 创建新表
            transaction.execute(
                "CREATE TABLE user_templates (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    project_id INTEGER NOT NULL DEFAULT 0,
                    name TEXT NOT NULL,
                    description TEXT NOT NULL DEFAULT '',
                    category TEXT NOT NULL DEFAULT '自定义',
                    content TEXT NOT NULL DEFAULT '',
                    created_at TEXT NOT NULL,
                    updated_at TEXT NOT NULL
                )",
                [],
            )?;
            // 3. 复制数据
            transaction.execute(
                "INSERT INTO user_templates (id, project_id, name, description, category, content, created_at, updated_at)
                 SELECT id, project_id, name, description, category, content, created_at, updated_at FROM user_templates_old",
                [],
            )?;
            // 4. 删除旧表
            transaction.execute("DROP TABLE user_templates_old", [])?;
            transaction.execute("PRAGMA foreign_keys = ON", [])?;
            transaction.commit()?;
        }
    }
    Ok(())
}

/// 创建灵感条目表
///
/// 存储灵感看板的条目信息，支持多列管理灵感。
/// 支持向后兼容：检查并添加缺失的 `column_key` 和 `column_name` 列。
///
/// **表结构说明：**
/// | 字段名 | 类型 | 约束 | 说明 |
/// |--------|------|------|------|
/// | id | INTEGER | PRIMARY KEY AUTOINCREMENT | 数据库主键，自增ID |
/// | project_id | INTEGER | NOT NULL, FK | 所属项目ID，关联 projects(id) |
/// | column_key | TEXT | NOT NULL DEFAULT 'inspiration' | 列标识：inspiration(灵感)/dialogue(对白)/scene(场景) |
/// | column_name | TEXT | NOT NULL DEFAULT '' | 列显示名称 |
/// | content | TEXT | NOT NULL DEFAULT '' | 灵感内容 |
/// | sort_order | INTEGER | NOT NULL DEFAULT 0 | 排序序号 |
/// | created_at | TEXT | NOT NULL | 创建时间（RFC3339格式） |
/// | updated_at | TEXT | NOT NULL | 最后更新时间（RFC3339格式） |
fn create_inspiration_items_table(conn: &Connection) -> SqliteResult<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS inspiration_items (
            id INTEGER PRIMARY KEY AUTOINCREMENT,         -- 数据库主键，自增ID
            project_id INTEGER NOT NULL,                  -- 所属项目ID
            column_key TEXT NOT NULL DEFAULT 'inspiration', -- 列标识
            column_name TEXT NOT NULL DEFAULT '',         -- 列显示名称
            content TEXT NOT NULL DEFAULT '',             -- 灵感内容
            sort_order INTEGER NOT NULL DEFAULT 0,        -- 排序序号
            created_at TEXT NOT NULL,                     -- 创建时间
            updated_at TEXT NOT NULL,                     -- 最后更新时间
            FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
        )",
        [],
    )?;

    // 向后兼容：检查是否存在 column_key 列，不存在则添加
    let has_column_key: bool = conn
        .query_row(
            "SELECT COUNT(*) > 0 FROM pragma_table_info('inspiration_items') WHERE name='column_key'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(false);

    if !has_column_key {
        let _ = conn.execute(
            "ALTER TABLE inspiration_items ADD COLUMN column_key TEXT NOT NULL DEFAULT 'inspiration'",
            [],
        );

        // 迁移旧数据：根据 column_name 映射到 column_key
        let _ = conn.execute(
            "UPDATE inspiration_items SET column_key = CASE 
             WHEN column_name = '灵感' OR column_name = 'Inspiration' THEN 'inspiration' 
             WHEN column_name = '对白' OR column_name = 'Dialogue' THEN 'dialogue' 
             WHEN column_name = '场景' OR column_name = 'Scene' THEN 'scene' 
             ELSE 'custom-' || id 
             END 
             WHERE column_key = 'inspiration' AND column_name != '灵感' AND column_name != 'Inspiration'",
            [],
        );
    }

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
            "ALTER TABLE inspiration_items ADD COLUMN column_name TEXT NOT NULL DEFAULT ''",
            [],
        );
    }

    Ok(())
}

// ============================================================================
// 备份与迁移相关表创建函数
// ============================================================================

/// 创建备份表
///
/// 存储项目备份记录，包括备份类型、文件路径、大小等信息。
/// 支持全量备份（full）和增量备份（incremental）。
///
/// **表结构说明：**
/// | 字段名 | 类型 | 约束 | 说明 |
/// |--------|------|------|------|
/// | id | INTEGER | PRIMARY KEY AUTOINCREMENT | 数据库主键，自增ID |
/// | project_id | INTEGER | NOT NULL, FK | 所属项目ID，关联 projects(id) |
/// | backup_type | TEXT | NOT NULL DEFAULT 'full' | 备份类型：full(全量)/incremental(增量) |
/// | file_path | TEXT | NOT NULL | 备份文件路径 |
/// | file_size | INTEGER | NOT NULL DEFAULT 0 | 文件大小（字节） |
/// | git_commit | TEXT | NULL | Git 提交哈希（版本控制备份时使用） |
/// | description | TEXT | NOT NULL DEFAULT '' | 备份描述 |
/// | created_at | TEXT | NOT NULL | 创建时间（RFC3339格式） |
/// | status | TEXT | NOT NULL DEFAULT 'completed' | 备份状态：pending(待处理)/completed(已完成)/failed(失败) |
fn create_backups_table(conn: &Connection) -> SqliteResult<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS backups (
            id INTEGER PRIMARY KEY AUTOINCREMENT,        -- 数据库主键，自增ID
            project_id INTEGER NOT NULL,                 -- 所属项目ID
            backup_type TEXT NOT NULL DEFAULT 'full',    -- 备份类型
            file_path TEXT NOT NULL,                     -- 备份文件路径
            file_size INTEGER NOT NULL DEFAULT 0,        -- 文件大小（字节）
            git_commit TEXT,                             -- Git提交哈希
            description TEXT NOT NULL DEFAULT '',        -- 备份描述
            created_at TEXT NOT NULL,                    -- 创建时间
            status TEXT NOT NULL DEFAULT 'completed',    -- 备份状态
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
///
/// **表结构说明：**
/// | 字段名 | 类型 | 约束 | 说明 |
/// |--------|------|------|------|
/// | id | INTEGER | PRIMARY KEY AUTOINCREMENT | 数据库主键，自增ID |
/// | project_id | INTEGER | NOT NULL, FK | 所属项目ID，关联 projects(id) |
/// | backup_id | INTEGER | NULL, FK | 关联备份ID，关联 backups(id) |
/// | operation | TEXT | NOT NULL | 操作类型 |
/// | message | TEXT | NOT NULL | 日志消息 |
/// | level | TEXT | NOT NULL DEFAULT 'info' | 日志级别：info(信息)/warning(警告)/error(错误) |
/// | created_at | TEXT | NOT NULL | 创建时间（RFC3339格式） |
fn create_backup_logs_table(conn: &Connection) -> SqliteResult<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS backup_logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,       -- 数据库主键，自增ID
            project_id INTEGER NOT NULL,                -- 所属项目ID
            backup_id INTEGER,                          -- 关联备份ID
            operation TEXT NOT NULL,                    -- 操作类型
            message TEXT NOT NULL,                      -- 日志消息
            level TEXT NOT NULL DEFAULT 'info',         -- 日志级别
            created_at TEXT NOT NULL,                   -- 创建时间
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
/// 记录迁移前后的路径、项目 ID 和状态。
///
/// **表结构说明：**
/// | 字段名 | 类型 | 约束 | 说明 |
/// |--------|------|------|------|
/// | id | INTEGER | PRIMARY KEY AUTOINCREMENT | 数据库主键，自增ID |
/// | operation | TEXT | NOT NULL | 操作类型 |
/// | project_db_id | INTEGER | NOT NULL | 项目数据库ID |
/// | old_project_id | TEXT | NULL | 迁移前的项目标识符 |
/// | new_project_id | TEXT | NOT NULL | 迁移后的项目标识符 |
/// | old_path | TEXT | NOT NULL | 迁移前的项目路径 |
/// | new_path | TEXT | NOT NULL | 迁移后的项目路径 |
/// | status | TEXT | NOT NULL DEFAULT 'success' | 迁移状态：success(成功)/failed(失败) |
/// | error_message | TEXT | DEFAULT '' | 错误信息（迁移失败时记录） |
/// | created_at | TEXT | NOT NULL | 创建时间（RFC3339格式） |
fn create_migration_logs_table(conn: &Connection) -> SqliteResult<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS migration_logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,        -- 数据库主键，自增ID
            operation TEXT NOT NULL,                     -- 操作类型
            project_db_id INTEGER NOT NULL,              -- 项目数据库ID
            old_project_id TEXT,                         -- 迁移前的项目标识符
            new_project_id TEXT NOT NULL,                -- 迁移后的项目标识符
            old_path TEXT NOT NULL,                      -- 迁移前的项目路径
            new_path TEXT NOT NULL,                      -- 迁移后的项目路径
            status TEXT NOT NULL DEFAULT 'success',      -- 迁移状态
            error_message TEXT DEFAULT '',               -- 错误信息
            created_at TEXT NOT NULL                     -- 创建时间
        )",
        [],
    )?;
    Ok(())
}

// ============================================================================
// 操作日志相关表创建函数
// ============================================================================

/// 创建操作日志表
///
/// 存储用户和系统操作的详细记录，用于审计、追踪和统计。
/// 包含操作类型、时间、结果、时长等信息。
/// 创建了多个索引以提高查询性能。
///
/// **表结构说明：**
/// | 字段名 | 类型 | 约束 | 说明 |
/// |--------|------|------|------|
/// | id | INTEGER | PRIMARY KEY AUTOINCREMENT | 数据库主键，自增ID |
/// | timestamp | TEXT | NOT NULL DEFAULT (datetime('now')) | 操作时间戳 |
/// | user_id | TEXT | NOT NULL DEFAULT 'system' | 用户ID（'system'表示系统操作） |
/// | operation_type | TEXT | NOT NULL | 操作类型 |
/// | operation_action | TEXT | NOT NULL | 操作动作 |
/// | target_type | TEXT | NOT NULL DEFAULT 'unknown' | 目标类型 |
/// | target_id | TEXT | NULL | 目标ID |
/// | details | TEXT | NULL | 操作详情（JSON格式） |
/// | result | TEXT | NOT NULL DEFAULT 'success' | 操作结果：success(成功)/failed(失败)/partial(部分成功) |
/// | duration_ms | INTEGER | NULL | 操作时长（毫秒） |
/// | ip_address | TEXT | NULL | IP地址 |
/// | project_id | INTEGER | NULL | 关联项目ID |
/// | created_at | TEXT | NOT NULL DEFAULT (datetime('now')) | 创建时间 |
fn create_operation_logs_table(conn: &Connection) -> SqliteResult<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS operation_logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,           -- 数据库主键，自增ID
            timestamp TEXT NOT NULL DEFAULT (datetime('now')), -- 操作时间戳
            user_id TEXT NOT NULL DEFAULT 'system',          -- 用户ID
            operation_type TEXT NOT NULL,                    -- 操作类型
            operation_action TEXT NOT NULL,                  -- 操作动作
            target_type TEXT NOT NULL DEFAULT 'unknown',     -- 目标类型
            target_id TEXT,                                  -- 目标ID
            details TEXT,                                    -- 操作详情（JSON）
            result TEXT NOT NULL DEFAULT 'success',          -- 操作结果
            duration_ms INTEGER,                             -- 操作时长（毫秒）
            ip_address TEXT,                                 -- IP地址
            project_id INTEGER,                              -- 关联项目ID
            created_at TEXT NOT NULL DEFAULT (datetime('now')) -- 创建时间
        )",
        [],
    )?;

    // 创建索引以提高查询性能
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_operation_logs_timestamp ON operation_logs(timestamp)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_operation_logs_type ON operation_logs(operation_type)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_operation_logs_project ON operation_logs(project_id)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_operation_logs_result ON operation_logs(result)",
        [],
    )?;

    Ok(())
}

/// 创建操作日志统计表
///
/// 存储每日操作统计数据，用于快速查询和展示统计信息。
/// 按日期和操作类型进行分组统计。
///
/// **表结构说明：**
/// | 字段名 | 类型 | 约束 | 说明 |
/// |--------|------|------|------|
/// | id | INTEGER | PRIMARY KEY AUTOINCREMENT | 数据库主键，自增ID |
/// | date | TEXT | NOT NULL | 统计日期（格式：YYYY-MM-DD） |
/// | operation_type | TEXT | NOT NULL | 操作类型 |
/// | count | INTEGER | NOT NULL DEFAULT 0 | 操作总数 |
/// | success_count | INTEGER | NOT NULL DEFAULT 0 | 成功操作数 |
/// | failed_count | INTEGER | NOT NULL DEFAULT 0 | 失败操作数 |
fn create_operation_log_stats_table(conn: &Connection) -> SqliteResult<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS operation_log_stats (
            id INTEGER PRIMARY KEY AUTOINCREMENT,       -- 数据库主键，自增ID
            date TEXT NOT NULL,                          -- 统计日期（YYYY-MM-DD）
            operation_type TEXT NOT NULL,                -- 操作类型
            count INTEGER NOT NULL DEFAULT 0,            -- 操作总数
            success_count INTEGER NOT NULL DEFAULT 0,    -- 成功操作数
            failed_count INTEGER NOT NULL DEFAULT 0,     -- 失败操作数
            UNIQUE(date, operation_type)
        )",
        [],
    )?;

    Ok(())
}

// ============================================================================
// 枚举字典相关表创建函数
// ============================================================================

/// 创建枚举字典表
///
/// 存储系统中使用的枚举数据，用于前端展示和数据验证。
/// 包含类别、代码、名称、描述和排序顺序等字段。
/// 创建了索引以提高按类别查询的性能。
///
/// **表结构说明：**
/// | 字段名 | 类型 | 约束 | 说明 |
/// |--------|------|------|------|
/// | id | INTEGER | PRIMARY KEY AUTOINCREMENT | 数据库主键，自增ID |
/// | category | TEXT | NOT NULL | 枚举类别（如：chapter_status、gender等） |
/// | code | TEXT | NOT NULL | 枚举代码（唯一标识） |
/// | name | TEXT | NOT NULL | 枚举名称（显示用） |
/// | description | TEXT | NULL | 枚举描述 |
/// | sort_order | INTEGER | NOT NULL DEFAULT 0 | 排序序号 |
/// | created_at | TEXT | NOT NULL DEFAULT (datetime('now')) | 创建时间 |
fn create_enum_dictionary_table(conn: &Connection) -> SqliteResult<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS enum_dictionary (
            id INTEGER PRIMARY KEY AUTOINCREMENT,           -- 数据库主键，自增ID
            category TEXT NOT NULL,                          -- 枚举类别
            code TEXT NOT NULL,                              -- 枚举代码
            name TEXT NOT NULL,                              -- 枚举名称
            description TEXT,                                -- 枚举描述
            sort_order INTEGER NOT NULL DEFAULT 0,           -- 排序序号
            created_at TEXT NOT NULL DEFAULT (datetime('now')), -- 创建时间
            UNIQUE(category, code)
        )",
        [],
    )?;

    // 创建索引以提高查询性能
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_enum_dict_category ON enum_dictionary(category)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_enum_dict_code ON enum_dictionary(code)",
        [],
    )?;

    Ok(())
}

// ============================================================================
// 枚举字典数据初始化函数
// ============================================================================

/// 初始化枚举字典数据
///
/// 检查枚举字典表中是否已有数据，如没有则插入默认的枚举数据。
/// 使用 INSERT OR IGNORE 避免重复插入已有的数据。
///
/// # 参数
/// - `conn`: SQLite 数据库连接
///
/// # 返回值
/// 初始化成功返回 `Ok(())`，失败返回 `rusqlite::Error`
fn init_enum_dictionary_data(conn: &Connection) -> SqliteResult<()> {
    // 检查是否已有数据
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM enum_dictionary", [], |row| row.get(0))
        .unwrap_or(0);

    if count > 0 {
        return Ok(()); // 已有数据，无需初始化
    }

    // 章节状态枚举
    let chapter_status_enums = vec![
        ("chapter_status", "outline", "大纲", 1),
        ("chapter_status", "draft", "草稿", 2),
        ("chapter_status", "revised", "修订", 3),
        ("chapter_status", "final", "定稿", 4),
        ("chapter_status", "abandoned", "废弃", 5),
    ];
    insert_enums_batch(conn, &chapter_status_enums)?;

    // 专注会话类型枚举
    let session_type_enums = vec![
        ("session_type", "work", "工作", 1),
        ("session_type", "short_break", "短休息", 2),
        ("session_type", "long_break", "长休息", 3),
    ];
    insert_enums_batch(conn, &session_type_enums)?;

    // 备份状态枚举
    let backup_status_enums = vec![
        ("backup_status", "pending", "待处理", 1),
        ("backup_status", "completed", "已完成", 2),
        ("backup_status", "failed", "失败", 3),
    ];
    insert_enums_batch(conn, &backup_status_enums)?;

    // 操作结果枚举
    let operation_result_enums = vec![
        ("operation_result", "success", "成功", 1),
        ("operation_result", "failed", "失败", 2),
        ("operation_result", "partial", "部分成功", 3),
    ];
    insert_enums_batch(conn, &operation_result_enums)?;

    // 性别枚举
    let gender_enums = vec![
        ("gender", "male", "男", 1),
        ("gender", "female", "女", 2),
        ("gender", "other", "其他", 3),
        ("gender", "unknown", "未知", 4),
    ];
    insert_enums_batch(conn, &gender_enums)?;

    // 地点类型枚举
    let location_type_enums = vec![
        ("location_type", "city", "城市", 1),
        ("location_type", "town", "城镇", 2),
        ("location_type", "village", "村庄", 3),
        ("location_type", "building", "建筑", 4),
        ("location_type", "region", "区域", 5),
        ("location_type", "country", "国家", 6),
        ("location_type", "kingdom", "王国", 7),
        ("location_type", "mountain", "山脉", 8),
        ("location_type", "forest", "森林", 9),
        ("location_type", "ocean", "海洋", 10),
        ("location_type", "other", "其他", 99),
    ];
    insert_enums_batch(conn, &location_type_enums)?;

    // 组织类型枚举
    let organization_type_enums = vec![
        ("organization_type", "kingdom", "王国", 1),
        ("organization_type", "guild", "公会", 2),
        ("organization_type", "gang", "帮派", 3),
        ("organization_type", "cult", "教派", 4),
        ("organization_type", "company", "商会", 5),
        ("organization_type", "military", "军队", 6),
        ("organization_type", "secret_society", "秘密组织", 7),
        ("organization_type", "family", "家族", 8),
        ("organization_type", "church", "教会", 9),
        ("organization_type", "other", "其他", 99),
    ];
    insert_enums_batch(conn, &organization_type_enums)?;

    // 提及类型枚举
    let mention_type_enums = vec![
        ("mention_type", "character", "人物", 1),
        ("mention_type", "location", "地点", 2),
        ("mention_type", "organization", "组织", 3),
    ];
    insert_enums_batch(conn, &mention_type_enums)?;

    // 操作类别枚举
    let operation_category_enums = vec![
        ("operation_category", "project", "项目管理", 1),
        ("operation_category", "chapter", "章节管理", 2),
        ("operation_category", "volume", "卷管理", 3),
        ("operation_category", "writing", "写作", 4),
        ("operation_category", "worldbuilding", "世界观", 5),
        ("operation_category", "character", "角色管理", 6),
        ("operation_category", "relationship", "关系管理", 7),
        ("operation_category", "timeline", "时间线", 8),
        ("operation_category", "export", "导出", 9),
        ("operation_category", "backup", "备份", 10),
        ("operation_category", "encryption", "加密", 11),
        ("operation_category", "template", "模板", 12),
        ("operation_category", "settings", "设置", 13),
        ("operation_category", "focus", "专注", 14),
        ("operation_category", "stats", "统计", 15),
    ];
    insert_enums_batch(conn, &operation_category_enums)?;

    Ok(())
}

/// 批量插入枚举数据
///
/// 使用 INSERT OR IGNORE 避免重复插入已有的数据。
///
/// # 参数
/// - `conn`: SQLite 数据库连接
/// - `enums`: 枚举数据数组，每个元素是 (category, code, name, sort_order) 元组
///
/// # 返回值
/// 插入成功返回 `Ok(())`，失败返回 `rusqlite::Error`
fn insert_enums_batch(conn: &Connection, enums: &[(&str, &str, &str, i32)]) -> SqliteResult<()> {
    let mut stmt = conn.prepare_cached(
        "INSERT OR IGNORE INTO enum_dictionary (category, code, name, sort_order)
         VALUES (?1, ?2, ?3, ?4)",
    )?;

    for &(category, code, name, order) in enums {
        stmt.execute((category, code, name, order))?;
    }

    Ok(())
}

// ============================================================================
// 表注释元数据相关函数
// ============================================================================

/// 创建表注释元数据表
///
/// 由于 SQLite 原生不支持 COMMENT 语法，使用此表存储表和字段的中文注释。
/// 可通过 SQL 查询获取表结构注释信息，用于文档生成和前端展示。
///
/// **表结构说明：**
/// | 字段名 | 类型 | 约束 | 说明 |
/// |--------|------|------|------|
/// | id | INTEGER | PRIMARY KEY AUTOINCREMENT | 数据库主键，自增ID |
/// | table_name | TEXT | NOT NULL | 表名 |
/// | column_name | TEXT | NULL | 字段名（NULL表示表级注释） |
/// | comment | TEXT | NOT NULL | 注释内容 |
/// | created_at | TEXT | NOT NULL DEFAULT (datetime('now')) | 创建时间 |
fn create_table_comments_table(conn: &Connection) -> SqliteResult<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS table_comments (
            id INTEGER PRIMARY KEY AUTOINCREMENT,     -- 数据库主键，自增ID
            table_name TEXT NOT NULL,                  -- 表名
            column_name TEXT,                          -- 字段名（NULL表示表级注释）
            comment TEXT NOT NULL,                     -- 注释内容
            created_at TEXT NOT NULL DEFAULT (datetime('now')), -- 创建时间
            UNIQUE(table_name, column_name)
        )",
        [],
    )?;

    Ok(())
}

/// 初始化表注释数据
///
/// 检查表注释表中是否已有数据，如没有则插入所有表和字段的中文注释。
/// 使用 INSERT OR IGNORE 避免重复插入已有的数据。
fn init_table_comments_data(conn: &Connection) -> SqliteResult<()> {
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM table_comments", [], |row| row.get(0))
        .unwrap_or(0);

    if count > 0 {
        return Ok(());
    }

    // 批量插入表注释
    insert_table_comments_batch(conn, &get_all_table_comments())?;

    Ok(())
}

/// 获取所有表的注释数据
fn get_all_table_comments() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![
        // ==================== 项目管理相关表 ====================
        (
            "projects",
            "",
            "存储小说项目的基本信息，包括书名、作者、路径等元数据",
        ),
        ("projects", "id", "数据库主键，自增ID"),
        (
            "projects",
            "project_id",
            "项目唯一标识符（6位字母数字组合），用于文件夹命名",
        ),
        ("projects", "name", "项目名称（书名）"),
        ("projects", "author", "作者名称"),
        ("projects", "description", "项目描述/简介"),
        ("projects", "path", "项目文件夹在本地的绝对路径"),
        ("projects", "created_at", "创建时间（RFC3339格式）"),
        ("projects", "last_opened_at", "最后打开时间（RFC3339格式）"),
        ("volumes", "", "存储小说的分卷信息，用于组织章节结构"),
        ("volumes", "id", "数据库主键，自增ID"),
        ("volumes", "project_id", "所属项目ID"),
        ("volumes", "name", "卷名称（如：第一卷、上部等）"),
        ("volumes", "sort_order", "排序序号，用于卷的顺序排列"),
        (
            "chapters",
            "",
            "存储小说章节信息，每个章节对应一个Markdown文件",
        ),
        ("chapters", "id", "数据库主键，自增ID"),
        ("chapters", "volume_id", "所属卷ID"),
        ("chapters", "title", "章节标题"),
        ("chapters", "file_path", "章节对应的Markdown文件相对路径"),
        ("chapters", "sort_order", "排序序号，用于章节顺序排列"),
        ("chapters", "summary", "章节摘要/大纲"),
        ("chapters", "word_count_cache", "字数缓存（避免每次计算）"),
        (
            "chapters",
            "status",
            "章节状态：outline(大纲)/draft(草稿)/revised(修订)/final(定稿)/abandoned(废弃)",
        ),
        ("chapters", "created_at", "创建时间（RFC3339格式）"),
        ("chapters", "updated_at", "最后更新时间（RFC3339格式）"),
        // ==================== 写作相关表 ====================
        ("writing_goals", "", "存储项目的每日写作目标设置"),
        ("writing_goals", "id", "数据库主键，自增ID"),
        ("writing_goals", "project_id", "所属项目ID"),
        (
            "writing_goals",
            "daily_goal",
            "每日写作目标字数（默认3000字）",
        ),
        ("writing_goals", "updated_at", "最后更新时间（RFC3339格式）"),
        (
            "writing_records",
            "",
            "存储每日写作统计数据，用于追踪写作进度",
        ),
        ("writing_records", "id", "数据库主键，自增ID"),
        ("writing_records", "project_id", "所属项目ID"),
        (
            "writing_records",
            "record_date",
            "记录日期（格式：YYYY-MM-DD）",
        ),
        ("writing_records", "total_words", "当日写作总字数"),
        ("writing_records", "duration", "当日写作时长（分钟）"),
        ("focus_sessions", "", "存储番茄钟专注会话记录"),
        ("focus_sessions", "id", "数据库主键，自增ID"),
        ("focus_sessions", "project_id", "所属项目ID"),
        (
            "focus_sessions",
            "session_type",
            "会话类型：work(工作)/short_break(短休息)/long_break(长休息)",
        ),
        ("focus_sessions", "duration_minutes", "会话时长（分钟）"),
        ("focus_sessions", "started_at", "开始时间（RFC3339格式）"),
        (
            "focus_sessions",
            "completed",
            "是否完成：1=完成，0=未完成（中途退出）",
        ),
        ("focus_sessions", "created_at", "创建时间（RFC3339格式）"),
        // ==================== 世界观相关表 ====================
        (
            "characters",
            "",
            "存储小说角色的详细信息，包括外貌、性格、背景故事等",
        ),
        ("characters", "id", "数据库主键，自增ID"),
        ("characters", "project_id", "所属项目ID"),
        ("characters", "name", "角色名称"),
        (
            "characters",
            "gender",
            "性别：male(男)/female(女)/other(其他)/unknown(未知)",
        ),
        ("characters", "age", "年龄"),
        ("characters", "appearance", "外貌描述"),
        ("characters", "personality", "性格描述"),
        ("characters", "background", "背景故事"),
        ("characters", "custom_fields", "自定义字段（JSON格式）"),
        ("characters", "created_at", "创建时间（RFC3339格式）"),
        ("characters", "updated_at", "最后更新时间（RFC3339格式）"),
        (
            "locations",
            "",
            "存储小说中的地点信息，包括类型、气候、人口、显著特征等",
        ),
        ("locations", "id", "数据库主键，自增ID"),
        ("locations", "project_id", "所属项目ID"),
        ("locations", "name", "地点名称"),
        (
            "locations",
            "location_type",
            "地点类型：city(城市)/town(城镇)/village(村庄)/building(建筑)/region(区域)/country(国家)/kingdom(王国)/mountain(山脉)/forest(森林)/ocean(海洋)/other(其他)",
        ),
        ("locations", "description", "地点描述"),
        ("locations", "climate", "气候特征"),
        ("locations", "population", "人口数量"),
        ("locations", "notable_features", "显著特征"),
        ("locations", "custom_fields", "自定义字段（JSON格式）"),
        ("locations", "created_at", "创建时间（RFC3339格式）"),
        ("locations", "updated_at", "最后更新时间（RFC3339格式）"),
        (
            "organizations",
            "",
            "存储小说中的组织/势力信息，包括类型、领袖、总部位置等",
        ),
        ("organizations", "id", "数据库主键，自增ID"),
        ("organizations", "project_id", "所属项目ID"),
        ("organizations", "name", "组织名称"),
        ("organizations", "org_type", "组织类型"),
        ("organizations", "description", "组织描述"),
        ("organizations", "leader", "领袖名称"),
        ("organizations", "headquarters", "总部位置"),
        ("organizations", "member_count", "成员数量"),
        ("organizations", "custom_fields", "自定义字段（JSON格式）"),
        ("organizations", "created_at", "创建时间（RFC3339格式）"),
        ("organizations", "updated_at", "最后更新时间（RFC3339格式）"),
        (
            "relationships",
            "",
            "存储角色之间的关系（如父子、师徒、朋友、敌人等）",
        ),
        ("relationships", "id", "数据库主键，自增ID"),
        ("relationships", "project_id", "所属项目ID"),
        ("relationships", "source_id", "关系源角色ID"),
        ("relationships", "target_id", "关系目标角色ID"),
        (
            "relationships",
            "relation_type",
            "关系类型：父子、师徒、朋友、敌人等",
        ),
        ("relationships", "created_at", "创建时间（RFC3339格式）"),
        ("events", "", "存储小说中的关键事件节点，用于时间线管理"),
        ("events", "id", "数据库主键，自增ID"),
        ("events", "project_id", "所属项目ID"),
        ("events", "title", "事件标题"),
        ("events", "story_time", "故事时间（事件在故事中的发生时间）"),
        ("events", "description", "事件描述"),
        ("events", "chapter_id", "关联章节ID"),
        ("events", "created_at", "创建时间（RFC3339格式）"),
        ("events", "updated_at", "最后更新时间（RFC3339格式）"),
        // ==================== 内容与模板相关表 ====================
        (
            "sensitive_words",
            "",
            "存储项目自定义的敏感词列表，用于内容审查",
        ),
        ("sensitive_words", "id", "数据库主键，自增ID"),
        ("sensitive_words", "project_id", "所属项目ID"),
        ("sensitive_words", "word", "敏感词内容"),
        ("sensitive_words", "created_at", "创建时间（RFC3339格式）"),
        (
            "user_templates",
            "",
            "存储用户自定义的写作模板，包括名称、分类、内容等",
        ),
        ("user_templates", "id", "数据库主键，自增ID"),
        (
            "user_templates",
            "project_id",
            "所属项目ID（0表示全局模板）",
        ),
        ("user_templates", "name", "模板名称"),
        ("user_templates", "description", "模板描述"),
        ("user_templates", "category", "模板分类"),
        ("user_templates", "content", "模板内容（Markdown格式）"),
        ("user_templates", "created_at", "创建时间（RFC3339格式）"),
        (
            "user_templates",
            "updated_at",
            "最后更新时间（RFC3339格式）",
        ),
        (
            "inspiration_items",
            "",
            "存储灵感看板的条目信息，支持多列管理灵感",
        ),
        ("inspiration_items", "id", "数据库主键，自增ID"),
        ("inspiration_items", "project_id", "所属项目ID"),
        (
            "inspiration_items",
            "column_key",
            "列标识：inspiration(灵感)/dialogue(对白)/scene(场景)",
        ),
        ("inspiration_items", "column_name", "列显示名称"),
        ("inspiration_items", "content", "灵感内容"),
        ("inspiration_items", "sort_order", "排序序号"),
        ("inspiration_items", "created_at", "创建时间（RFC3339格式）"),
        (
            "inspiration_items",
            "updated_at",
            "最后更新时间（RFC3339格式）",
        ),
        // ==================== 备份与迁移相关表 ====================
        (
            "backups",
            "",
            "存储项目备份记录，包括备份类型、文件路径、大小等信息",
        ),
        ("backups", "id", "数据库主键，自增ID"),
        ("backups", "project_id", "所属项目ID"),
        (
            "backups",
            "backup_type",
            "备份类型：full(全量)/incremental(增量)",
        ),
        ("backups", "file_path", "备份文件路径"),
        ("backups", "file_size", "文件大小（字节）"),
        ("backups", "git_commit", "Git提交哈希（版本控制备份时使用）"),
        ("backups", "description", "备份描述"),
        ("backups", "created_at", "创建时间（RFC3339格式）"),
        (
            "backups",
            "status",
            "备份状态：pending(待处理)/completed(已完成)/failed(失败)",
        ),
        (
            "backup_logs",
            "",
            "存储备份操作的详细日志，用于追踪备份过程和排查问题",
        ),
        ("backup_logs", "id", "数据库主键，自增ID"),
        ("backup_logs", "project_id", "所属项目ID"),
        ("backup_logs", "backup_id", "关联备份ID"),
        ("backup_logs", "operation", "操作类型"),
        ("backup_logs", "message", "日志消息"),
        (
            "backup_logs",
            "level",
            "日志级别：info(信息)/warning(警告)/error(错误)",
        ),
        ("backup_logs", "created_at", "创建时间（RFC3339格式）"),
        (
            "migration_logs",
            "",
            "存储项目迁移操作的记录，用于追踪旧版本项目迁移到新版本的过程",
        ),
        ("migration_logs", "id", "数据库主键，自增ID"),
        ("migration_logs", "operation", "操作类型"),
        ("migration_logs", "project_db_id", "项目数据库ID"),
        ("migration_logs", "old_project_id", "迁移前的项目标识符"),
        ("migration_logs", "new_project_id", "迁移后的项目标识符"),
        ("migration_logs", "old_path", "迁移前的项目路径"),
        ("migration_logs", "new_path", "迁移后的项目路径"),
        (
            "migration_logs",
            "status",
            "迁移状态：success(成功)/failed(失败)",
        ),
        (
            "migration_logs",
            "error_message",
            "错误信息（迁移失败时记录）",
        ),
        ("migration_logs", "created_at", "创建时间（RFC3339格式）"),
        // ==================== 操作日志与统计相关表 ====================
        (
            "operation_logs",
            "",
            "存储用户和系统操作的详细记录，用于审计、追踪和统计",
        ),
        ("operation_logs", "id", "数据库主键，自增ID"),
        ("operation_logs", "timestamp", "操作时间戳"),
        (
            "operation_logs",
            "user_id",
            "用户ID（'system'表示系统操作）",
        ),
        ("operation_logs", "operation_type", "操作类型"),
        ("operation_logs", "operation_action", "操作动作"),
        ("operation_logs", "target_type", "目标类型"),
        ("operation_logs", "target_id", "目标ID"),
        ("operation_logs", "details", "操作详情（JSON格式）"),
        (
            "operation_logs",
            "result",
            "操作结果：success(成功)/failed(失败)/partial(部分成功)",
        ),
        ("operation_logs", "duration_ms", "操作时长（毫秒）"),
        ("operation_logs", "ip_address", "IP地址"),
        ("operation_logs", "project_id", "关联项目ID"),
        ("operation_logs", "created_at", "创建时间（RFC3339格式）"),
        (
            "operation_log_stats",
            "",
            "存储每日操作统计数据，用于快速查询和展示统计信息",
        ),
        ("operation_log_stats", "id", "数据库主键，自增ID"),
        (
            "operation_log_stats",
            "date",
            "统计日期（格式：YYYY-MM-DD）",
        ),
        ("operation_log_stats", "operation_type", "操作类型"),
        ("operation_log_stats", "count", "操作总数"),
        ("operation_log_stats", "success_count", "成功操作数"),
        ("operation_log_stats", "failed_count", "失败操作数"),
        // ==================== 枚举字典表 ====================
        (
            "enum_dictionary",
            "",
            "存储系统中使用的枚举数据，用于前端展示和数据验证",
        ),
        ("enum_dictionary", "id", "数据库主键，自增ID"),
        (
            "enum_dictionary",
            "category",
            "枚举类别（如：chapter_status、gender等）",
        ),
        ("enum_dictionary", "code", "枚举代码（唯一标识）"),
        ("enum_dictionary", "name", "枚举名称（显示用）"),
        ("enum_dictionary", "description", "枚举描述"),
        ("enum_dictionary", "sort_order", "排序序号"),
        ("enum_dictionary", "created_at", "创建时间（RFC3339格式）"),
    ]
}

/// 批量插入表注释数据
fn insert_table_comments_batch(
    conn: &Connection,
    comments: &[(&str, &str, &str)],
) -> SqliteResult<()> {
    let mut stmt = conn.prepare_cached(
        "INSERT OR IGNORE INTO table_comments (table_name, column_name, comment)
         VALUES (?1, ?2, ?3)",
    )?;

    for &(table_name, column_name, comment) in comments {
        stmt.execute((table_name, column_name, comment))?;
    }

    Ok(())
}
