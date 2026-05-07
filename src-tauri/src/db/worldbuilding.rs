use chrono::Utc;
use rusqlite::{Connection, Result as SqliteResult};

use crate::models::{Character, Location, Organization};

pub fn create_character(
    conn: &Connection,
    project_id: i64,
    name: &str,
    gender: &str,
    age: Option<i32>,
    appearance: &str,
    personality: &str,
    background: &str,
    custom_fields: &str,
) -> SqliteResult<Character> {
    let now = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO characters (project_id, name, gender, age, appearance, personality, background, custom_fields, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        (project_id, name, gender, age, appearance, personality, background, custom_fields, &now, &now),
    )?;

    Ok(Character {
        id: conn.last_insert_rowid(),
        project_id,
        name: name.to_string(),
        gender: gender.to_string(),
        age,
        appearance: appearance.to_string(),
        personality: personality.to_string(),
        background: background.to_string(),
        custom_fields: custom_fields.to_string(),
        created_at: now.clone(),
        updated_at: now,
    })
}

pub fn update_character(
    conn: &Connection,
    id: i64,
    name: &str,
    gender: &str,
    age: Option<i32>,
    appearance: &str,
    personality: &str,
    background: &str,
    custom_fields: &str,
) -> SqliteResult<()> {
    let now = Utc::now().to_rfc3339();

    conn.execute(
        "UPDATE characters SET name = ?1, gender = ?2, age = ?3, appearance = ?4, 
         personality = ?5, background = ?6, custom_fields = ?7, updated_at = ?8 WHERE id = ?9",
        (name, gender, age, appearance, personality, background, custom_fields, &now, id),
    )?;

    Ok(())
}

pub fn delete_character(conn: &Connection, id: i64) -> SqliteResult<usize> {
    conn.execute("DELETE FROM characters WHERE id = ?1", [id])
}

pub fn list_characters(conn: &Connection, project_id: i64) -> SqliteResult<Vec<Character>> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, name, gender, age, appearance, personality, background, custom_fields, created_at, updated_at
         FROM characters WHERE project_id = ?1 ORDER BY id",
    )?;

    let characters = stmt
        .query_map([project_id], |row| {
            Ok(Character {
                id: row.get(0)?,
                project_id: row.get(1)?,
                name: row.get(2)?,
                gender: row.get(3)?,
                age: row.get(4)?,
                appearance: row.get(5)?,
                personality: row.get(6)?,
                background: row.get(7)?,
                custom_fields: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(characters)
}

pub fn create_location(
    conn: &Connection,
    project_id: i64,
    name: &str,
    location_type: &str,
    description: &str,
    climate: &str,
    population: Option<i32>,
    notable_features: &str,
    custom_fields: &str,
) -> SqliteResult<Location> {
    let now = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO locations (project_id, name, location_type, description, climate, population, notable_features, custom_fields, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        (project_id, name, location_type, description, climate, population, notable_features, custom_fields, &now, &now),
    )?;

    Ok(Location {
        id: conn.last_insert_rowid(),
        project_id,
        name: name.to_string(),
        location_type: location_type.to_string(),
        description: description.to_string(),
        climate: climate.to_string(),
        population,
        notable_features: notable_features.to_string(),
        custom_fields: custom_fields.to_string(),
        created_at: now.clone(),
        updated_at: now,
    })
}

pub fn list_locations(conn: &Connection, project_id: i64) -> SqliteResult<Vec<Location>> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, name, location_type, description, climate, population, notable_features, custom_fields, created_at, updated_at
         FROM locations WHERE project_id = ?1 ORDER BY id",
    )?;

    let locations = stmt
        .query_map([project_id], |row| {
            Ok(Location {
                id: row.get(0)?,
                project_id: row.get(1)?,
                name: row.get(2)?,
                location_type: row.get(3)?,
                description: row.get(4)?,
                climate: row.get(5)?,
                population: row.get(6)?,
                notable_features: row.get(7)?,
                custom_fields: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(locations)
}

pub fn create_organization(
    conn: &Connection,
    project_id: i64,
    name: &str,
    org_type: &str,
    description: &str,
    leader: &str,
    headquarters: &str,
    member_count: Option<i32>,
    custom_fields: &str,
) -> SqliteResult<Organization> {
    let now = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO organizations (project_id, name, org_type, description, leader, headquarters, member_count, custom_fields, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        (project_id, name, org_type, description, leader, headquarters, member_count, custom_fields, &now, &now),
    )?;

    Ok(Organization {
        id: conn.last_insert_rowid(),
        project_id,
        name: name.to_string(),
        org_type: org_type.to_string(),
        description: description.to_string(),
        leader: leader.to_string(),
        headquarters: headquarters.to_string(),
        member_count,
        custom_fields: custom_fields.to_string(),
        created_at: now.clone(),
        updated_at: now,
    })
}

pub fn list_organizations(conn: &Connection, project_id: i64) -> SqliteResult<Vec<Organization>> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, name, org_type, description, leader, headquarters, member_count, custom_fields, created_at, updated_at
         FROM organizations WHERE project_id = ?1 ORDER BY id",
    )?;

    let organizations = stmt
        .query_map([project_id], |row| {
            Ok(Organization {
                id: row.get(0)?,
                project_id: row.get(1)?,
                name: row.get(2)?,
                org_type: row.get(3)?,
                description: row.get(4)?,
                leader: row.get(5)?,
                headquarters: row.get(6)?,
                member_count: row.get(7)?,
                custom_fields: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(organizations)
}
