use chrono::Utc;
use rusqlite::{Connection, Result as SqliteResult};

pub fn create_backup(
    conn: &Connection,
    project_id: i64,
    backup_type: &str,
    file_path: &str,
    file_size: i64,
    git_commit: Option<&str>,
    description: &str,
) -> SqliteResult<i64> {
    let now = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO backups (project_id, backup_type, file_path, file_size, git_commit, description, created_at, status)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'completed')",
        (project_id, backup_type, file_path, file_size, git_commit, description, now),
    )?;

    Ok(conn.last_insert_rowid())
}

pub fn list_backups(conn: &Connection, project_id: i64) -> SqliteResult<Vec<(i64, String, String, i64, String)>> {
    let mut stmt = conn.prepare(
        "SELECT id, backup_type, file_path, file_size, created_at FROM backups WHERE project_id = ?1 ORDER BY created_at DESC",
    )?;

    let backups = stmt
        .query_map([project_id], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
            ))
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(backups)
}

pub fn delete_backup_record(conn: &Connection, id: i64) -> SqliteResult<usize> {
    conn.execute("DELETE FROM backups WHERE id = ?1", [id])
}

pub fn add_backup_log(
    conn: &Connection,
    project_id: i64,
    backup_id: Option<i64>,
    operation: &str,
    message: &str,
    level: &str,
) -> SqliteResult<()> {
    let now = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO backup_logs (project_id, backup_id, operation, message, level, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        (project_id, backup_id, operation, message, level, now),
    )?;

    Ok(())
}
