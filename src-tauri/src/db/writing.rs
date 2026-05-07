use chrono::Utc;
use rusqlite::{Connection, OptionalExtension, Result as SqliteResult};

use crate::models::{WritingGoal, WritingRecord};

pub fn get_writing_goal(conn: &Connection, project_id: i64) -> SqliteResult<Option<WritingGoal>> {
    conn.query_row(
        "SELECT id, project_id, daily_goal, updated_at FROM writing_goals WHERE project_id = ?1",
        [project_id],
        |row| {
            Ok(WritingGoal {
                id: row.get(0)?,
                project_id: row.get(1)?,
                daily_goal: row.get(2)?,
                updated_at: row.get(3)?,
            })
        },
    ).optional()
}

pub fn save_writing_goal(conn: &Connection, project_id: i64, daily_goal: i32) -> SqliteResult<WritingGoal> {
    let now = Utc::now().to_rfc3339();

    let existing: Option<i64> = conn
        .query_row(
            "SELECT id FROM writing_goals WHERE project_id = ?1",
            [project_id],
            |row| row.get(0),
        )
        .optional()?;

    match existing {
        Some(id) => {
            conn.execute(
                "UPDATE writing_goals SET daily_goal = ?1, updated_at = ?2 WHERE id = ?3",
                (daily_goal, &now, id),
            )?;
            Ok(WritingGoal { id, project_id, daily_goal, updated_at: now })
        }
        None => {
            conn.execute(
                "INSERT INTO writing_goals (project_id, daily_goal, updated_at) VALUES (?1, ?2, ?3)",
                (project_id, daily_goal, &now),
            )?;
            Ok(WritingGoal {
                id: conn.last_insert_rowid(),
                project_id,
                daily_goal,
                updated_at: now,
            })
        }
    }
}

pub fn get_writing_records(conn: &Connection, project_id: i64) -> SqliteResult<Vec<WritingRecord>> {
    let mut stmt = conn.prepare(
        "SELECT record_date, total_words, duration 
         FROM writing_records WHERE project_id = ?1 ORDER BY record_date DESC LIMIT 30",
    )?;

    let records = stmt
        .query_map([project_id], |row| {
            Ok(WritingRecord {
                date: row.get(0)?,
                total_words: row.get(1)?,
                duration: row.get(2)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(records)
}

pub fn upsert_writing_record(
    conn: &Connection,
    project_id: i64,
    record_date: &str,
    total_words: i32,
    duration: i32,
) -> SqliteResult<()> {
    conn.execute(
        "INSERT INTO writing_records (project_id, record_date, total_words, duration)
         VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(project_id, record_date) DO UPDATE SET
         total_words = total_words + excluded.total_words,
         duration = duration + excluded.duration",
        (project_id, record_date, total_words, duration),
    )?;

    Ok(())
}
