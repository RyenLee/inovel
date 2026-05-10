use chrono::Utc;
use rusqlite::{Connection, Result as SqliteResult};

use crate::models::InspirationItem;

pub fn create_inspiration_item(
    conn: &Connection,
    project_id: i64,
    column_key: &str,
    content: &str,
) -> SqliteResult<InspirationItem> {
    let sort_order: i32 = conn
        .query_row(
            "SELECT COALESCE(MAX(sort_order), -1) + 1 FROM inspiration_items WHERE project_id = ?1 AND column_key = ?2",
            (project_id, column_key),
            |row| row.get(0),
        )
        .unwrap_or(0);

    let now = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO inspiration_items (project_id, column_key, column_name, content, sort_order, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        (project_id, column_key, "", content, sort_order, &now, &now),
    )?;

    Ok(InspirationItem {
        id: conn.last_insert_rowid(),
        project_id,
        column_key: column_key.to_string(),
        column_name: String::new(),
        content: content.to_string(),
        sort_order,
        created_at: now.clone(),
        updated_at: now,
    })
}

pub fn update_inspiration_item(conn: &Connection, id: i64, content: &str) -> SqliteResult<()> {
    let now = Utc::now().to_rfc3339();

    conn.execute(
        "UPDATE inspiration_items SET content = ?1, updated_at = ?2 WHERE id = ?3",
        (content, &now, id),
    )?;

    Ok(())
}

pub fn delete_inspiration_item(conn: &Connection, id: i64) -> SqliteResult<usize> {
    conn.execute("DELETE FROM inspiration_items WHERE id = ?1", [id])
}

pub fn get_inspiration_items(conn: &Connection, project_id: i64) -> SqliteResult<Vec<InspirationItem>> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, column_key, column_name, content, sort_order, created_at, updated_at
         FROM inspiration_items WHERE project_id = ?1 ORDER BY column_key, sort_order",
    )?;

    let items = stmt
        .query_map([project_id], |row| {
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
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(items)
}
