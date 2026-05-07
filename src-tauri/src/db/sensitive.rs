use chrono::Utc;
use rusqlite::{Connection, Result as SqliteResult};

use crate::models::SensitiveWord;

pub fn add_sensitive_word(conn: &Connection, project_id: i64, word: &str) -> SqliteResult<SensitiveWord> {
    let now = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO sensitive_words (project_id, word, created_at) VALUES (?1, ?2, ?3)",
        (project_id, word, &now),
    )?;

    Ok(SensitiveWord {
        id: conn.last_insert_rowid(),
        project_id,
        word: word.to_string(),
        created_at: now,
    })
}

pub fn remove_sensitive_word(conn: &Connection, id: i64) -> SqliteResult<usize> {
    conn.execute("DELETE FROM sensitive_words WHERE id = ?1", [id])
}

pub fn list_sensitive_words(conn: &Connection, project_id: i64) -> SqliteResult<Vec<SensitiveWord>> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, word, created_at FROM sensitive_words WHERE project_id = ?1 ORDER BY id",
    )?;

    let words = stmt
        .query_map([project_id], |row| {
            Ok(SensitiveWord {
                id: row.get(0)?,
                project_id: row.get(1)?,
                word: row.get(2)?,
                created_at: row.get(3)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(words)
}

pub fn import_sensitive_words(conn: &Connection, project_id: i64, words: &[&str]) -> SqliteResult<()> {
    let now = Utc::now().to_rfc3339();

    for word in words {
        let _ = conn.execute(
            "INSERT OR IGNORE INTO sensitive_words (project_id, word, created_at) VALUES (?1, ?2, ?3)",
            (project_id, word, &now),
        );
    }

    Ok(())
}
