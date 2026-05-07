use chrono::Utc;
use rusqlite::{Connection, Result as SqliteResult};

use crate::models::UserTemplate;

pub fn get_user_templates(conn: &Connection, project_id: i64) -> SqliteResult<Vec<UserTemplate>> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, name, description, category, content, created_at, updated_at
         FROM user_templates WHERE project_id = ?1 ORDER BY id",
    )?;

    let templates = stmt
        .query_map([project_id], |row| {
            Ok(UserTemplate {
                id: row.get(0)?,
                project_id: row.get(1)?,
                name: row.get(2)?,
                description: row.get(3)?,
                category: row.get(4)?,
                content: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(templates)
}

pub fn save_user_template(
    conn: &Connection,
    project_id: i64,
    name: &str,
    description: &str,
    category: &str,
    content: &str,
) -> SqliteResult<UserTemplate> {
    let now = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO user_templates (project_id, name, description, category, content, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        (project_id, name, description, category, content, &now, &now),
    )?;

    Ok(UserTemplate {
        id: conn.last_insert_rowid(),
        project_id,
        name: name.to_string(),
        description: description.to_string(),
        category: category.to_string(),
        content: content.to_string(),
        created_at: now.clone(),
        updated_at: now,
    })
}

pub fn update_user_template(
    conn: &Connection,
    id: i64,
    name: Option<&str>,
    description: Option<&str>,
    category: Option<&str>,
    content: Option<&str>,
) -> SqliteResult<()> {
    let now = Utc::now().to_rfc3339();

    match (name, description, category, content) {
        (Some(n), Some(d), Some(c), Some(cont)) => {
            conn.execute(
                "UPDATE user_templates SET name = ?1, description = ?2, category = ?3, content = ?4, updated_at = ?5 WHERE id = ?6",
                (n, d, c, cont, &now, id),
            )?;
        }
        (Some(n), Some(d), Some(c), None) => {
            conn.execute(
                "UPDATE user_templates SET name = ?1, description = ?2, category = ?3, updated_at = ?4 WHERE id = ?5",
                (n, d, c, &now, id),
            )?;
        }
        (Some(n), Some(d), None, Some(cont)) => {
            conn.execute(
                "UPDATE user_templates SET name = ?1, description = ?2, content = ?3, updated_at = ?4 WHERE id = ?5",
                (n, d, cont, &now, id),
            )?;
        }
        (Some(n), None, Some(c), Some(cont)) => {
            conn.execute(
                "UPDATE user_templates SET name = ?1, category = ?2, content = ?3, updated_at = ?4 WHERE id = ?5",
                (n, c, cont, &now, id),
            )?;
        }
        (None, Some(d), Some(c), Some(cont)) => {
            conn.execute(
                "UPDATE user_templates SET description = ?1, category = ?2, content = ?3, updated_at = ?4 WHERE id = ?5",
                (d, c, cont, &now, id),
            )?;
        }
        (Some(n), Some(d), None, None) => {
            conn.execute(
                "UPDATE user_templates SET name = ?1, description = ?2, updated_at = ?3 WHERE id = ?4",
                (n, d, &now, id),
            )?;
        }
        (Some(n), None, Some(c), None) => {
            conn.execute(
                "UPDATE user_templates SET name = ?1, category = ?2, updated_at = ?3 WHERE id = ?4",
                (n, c, &now, id),
            )?;
        }
        (Some(n), None, None, Some(cont)) => {
            conn.execute(
                "UPDATE user_templates SET name = ?1, content = ?2, updated_at = ?3 WHERE id = ?4",
                (n, cont, &now, id),
            )?;
        }
        (None, Some(d), Some(c), None) => {
            conn.execute(
                "UPDATE user_templates SET description = ?1, category = ?2, updated_at = ?3 WHERE id = ?4",
                (d, c, &now, id),
            )?;
        }
        (None, Some(d), None, Some(cont)) => {
            conn.execute(
                "UPDATE user_templates SET description = ?1, content = ?2, updated_at = ?3 WHERE id = ?4",
                (d, cont, &now, id),
            )?;
        }
        (None, None, Some(c), Some(cont)) => {
            conn.execute(
                "UPDATE user_templates SET category = ?1, content = ?2, updated_at = ?3 WHERE id = ?4",
                (c, cont, &now, id),
            )?;
        }
        (Some(n), None, None, None) => {
            conn.execute(
                "UPDATE user_templates SET name = ?1, updated_at = ?2 WHERE id = ?3",
                (n, &now, id),
            )?;
        }
        (None, Some(d), None, None) => {
            conn.execute(
                "UPDATE user_templates SET description = ?1, updated_at = ?2 WHERE id = ?3",
                (d, &now, id),
            )?;
        }
        (None, None, Some(c), None) => {
            conn.execute(
                "UPDATE user_templates SET category = ?1, updated_at = ?2 WHERE id = ?3",
                (c, &now, id),
            )?;
        }
        (None, None, None, Some(cont)) => {
            conn.execute(
                "UPDATE user_templates SET content = ?1, updated_at = ?2 WHERE id = ?3",
                (cont, &now, id),
            )?;
        }
        (None, None, None, None) => {}
    }

    Ok(())
}

pub fn delete_user_template(conn: &Connection, id: i64) -> SqliteResult<usize> {
    conn.execute("DELETE FROM user_templates WHERE id = ?1", [id])
}
