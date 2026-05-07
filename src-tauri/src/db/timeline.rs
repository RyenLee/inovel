use chrono::Utc;
use rusqlite::{Connection, Result as SqliteResult};

use crate::models::Event;

pub fn create_event(
    conn: &Connection,
    project_id: i64,
    title: &str,
    story_time: &str,
    description: &str,
    chapter_id: Option<i64>,
) -> SqliteResult<Event> {
    let now = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO events (project_id, title, story_time, description, chapter_id, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        (project_id, title, story_time, description, chapter_id, &now, &now),
    )?;

    Ok(Event {
        id: conn.last_insert_rowid(),
        project_id,
        title: title.to_string(),
        story_time: story_time.to_string(),
        description: description.to_string(),
        chapter_id,
        created_at: now.clone(),
        updated_at: now,
    })
}

pub fn update_event(
    conn: &Connection,
    id: i64,
    title: &str,
    story_time: &str,
    description: &str,
    chapter_id: Option<i64>,
) -> SqliteResult<()> {
    let now = Utc::now().to_rfc3339();

    conn.execute(
        "UPDATE events SET title = ?1, story_time = ?2, description = ?3, chapter_id = ?4, updated_at = ?5 WHERE id = ?6",
        (title, story_time, description, chapter_id, &now, id),
    )?;

    Ok(())
}

pub fn delete_event(conn: &Connection, id: i64) -> SqliteResult<usize> {
    conn.execute("DELETE FROM events WHERE id = ?1", [id])
}

pub fn list_events(conn: &Connection, project_id: i64) -> SqliteResult<Vec<Event>> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, title, story_time, description, chapter_id, created_at, updated_at
         FROM events WHERE project_id = ?1 ORDER BY story_time, id",
    )?;

    let events = stmt
        .query_map([project_id], |row| {
            Ok(Event {
                id: row.get(0)?,
                project_id: row.get(1)?,
                title: row.get(2)?,
                story_time: row.get(3)?,
                description: row.get(4)?,
                chapter_id: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(events)
}
