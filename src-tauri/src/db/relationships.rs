use chrono::Utc;
use rusqlite::{Connection, Result as SqliteResult};

use crate::models::Relationship;

pub fn create_relationship(
    conn: &Connection,
    project_id: i64,
    source_id: i64,
    target_id: i64,
    relation_type: &str,
) -> SqliteResult<Relationship> {
    let now = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO relationships (project_id, source_id, target_id, relation_type, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        (project_id, source_id, target_id, relation_type, &now),
    )?;

    Ok(Relationship {
        id: conn.last_insert_rowid(),
        project_id,
        source_id,
        target_id,
        relation_type: relation_type.to_string(),
        created_at: now,
    })
}

pub fn update_relationship(conn: &Connection, id: i64, relation_type: &str) -> SqliteResult<()> {
    conn.execute(
        "UPDATE relationships SET relation_type = ?1 WHERE id = ?2",
        (relation_type, id),
    )?;
    Ok(())
}

pub fn delete_relationship(conn: &Connection, id: i64) -> SqliteResult<usize> {
    conn.execute("DELETE FROM relationships WHERE id = ?1", [id])
}

pub fn get_relationships(conn: &Connection, project_id: i64) -> SqliteResult<Vec<Relationship>> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, source_id, target_id, relation_type, created_at
         FROM relationships WHERE project_id = ?1 ORDER BY id",
    )?;

    let relationships = stmt
        .query_map([project_id], |row| {
            Ok(Relationship {
                id: row.get(0)?,
                project_id: row.get(1)?,
                source_id: row.get(2)?,
                target_id: row.get(3)?,
                relation_type: row.get(4)?,
                created_at: row.get(5)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(relationships)
}
