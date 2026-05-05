use crate::config::get_db_path as config_get_db_path;
use rusqlite::{Connection, Result as SqliteResult};
use std::path::PathBuf;
use tauri::AppHandle;

/// 获取数据库路径（从配置读取）
pub(crate) fn get_db_path(app_handle: &AppHandle) -> PathBuf {
    config_get_db_path(app_handle).expect("Failed to get database path from config")
}

pub(crate) fn init_db(conn: &Connection) -> SqliteResult<()> {
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

    // 迁移：添加 last_opened_at 列（如果不存在）
    let has_last_opened: bool = conn.query_row(
        "SELECT COUNT(*) > 0 FROM pragma_table_info('projects') WHERE name='last_opened_at'",
        [],
        |row| row.get(0),
    ).unwrap_or(false);

    if !has_last_opened {
        let _ = conn.execute(
            "ALTER TABLE projects ADD COLUMN last_opened_at TEXT",
            [],
        );
    }

    // 迁移：添加 project_id 列（如果不存在）
    let has_project_id: bool = conn.query_row(
        "SELECT COUNT(*) > 0 FROM pragma_table_info('projects') WHERE name='project_id'",
        [],
        |row| row.get(0),
    ).unwrap_or(false);

    if !has_project_id {
        let _ = conn.execute(
            "ALTER TABLE projects ADD COLUMN project_id TEXT",
            [],
        );
    }

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

    // 迁移：添加 status 列（如果不存在）
    let has_status: bool = conn.query_row(
        "SELECT COUNT(*) > 0 FROM pragma_table_info('chapters') WHERE name='status'",
        [],
        |row| row.get(0),
    ).unwrap_or(false);

    if !has_status {
        let _ = conn.execute(
            "ALTER TABLE chapters ADD COLUMN status TEXT NOT NULL DEFAULT 'draft'",
            [],
        );
    }

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

    // 迁移：创建 migration_logs 表（记录迁移操作历史）
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

pub(crate) fn check_project_exists(path: &str) -> bool {
    let project_path = PathBuf::from(path);
    project_path.exists() && project_path.is_dir()
}
