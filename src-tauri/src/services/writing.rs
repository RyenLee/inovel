use chrono::Utc;
use rusqlite::Connection;
use tauri::AppHandle;
use tracing::info;

use crate::config;
use crate::error::Result;
use crate::models::{WritingGoal, WritingRecord};

pub struct WritingService;

impl WritingService {
    pub fn get_writing_goal(
        app_handle: &AppHandle,
        project_id: i64,
    ) -> Result<Option<WritingGoal>> {
        let db_path = config::get_db_path(app_handle);
        let conn = Connection::open(&db_path)?;

        let goal = conn
            .query_row(
                "SELECT id, project_id, daily_goal, updated_at 
                 FROM writing_goals WHERE project_id = ?1",
                [project_id],
                |row| {
                    Ok(WritingGoal {
                        id: row.get(0)?,
                        project_id: row.get(1)?,
                        daily_goal: row.get(2)?,
                        updated_at: row.get(3)?,
                    })
                },
            )
            .ok();

        Ok(goal)
    }

    pub fn save_writing_goal(
        app_handle: &AppHandle,
        project_id: i64,
        daily_goal: i32,
    ) -> Result<WritingGoal> {
        let db_path = config::get_db_path(app_handle);
        let conn = Connection::open(&db_path)?;
        let now = Utc::now().to_rfc3339();

        let existing: Option<i64> = conn
            .query_row(
                "SELECT id FROM writing_goals WHERE project_id = ?1",
                [project_id],
                |row| row.get(0),
            )
            .ok();

        match existing {
            Some(id) => {
                conn.execute(
                    "UPDATE writing_goals SET daily_goal = ?1, updated_at = ?2 WHERE id = ?3",
                    (daily_goal, &now, id),
                )?;

                Ok(WritingGoal {
                    id,
                    project_id,
                    daily_goal,
                    updated_at: now,
                })
            }
            None => {
                let id = Utc::now().timestamp_millis();
                conn.execute(
                    "INSERT INTO writing_goals (id, project_id, daily_goal, updated_at) 
                     VALUES (?1, ?2, ?3, ?4)",
                    (id, project_id, daily_goal, &now),
                )?;

                info!(project_id = %project_id, daily_goal = %daily_goal, "写作目标已保存");

                Ok(WritingGoal {
                    id,
                    project_id,
                    daily_goal,
                    updated_at: now,
                })
            }
        }
    }

    pub fn get_writing_stats(
        app_handle: &AppHandle,
        project_id: i64,
    ) -> Result<Vec<WritingRecord>> {
        let db_path = config::get_db_path(app_handle);
        let conn = Connection::open(&db_path)?;

        let mut stmt = conn.prepare(
            "SELECT date, total_words, duration FROM writing_records 
             WHERE project_id = ?1 ORDER BY date DESC LIMIT 30",
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

    pub fn record_writing_session(
        app_handle: &AppHandle,
        project_id: i64,
        words_written: i32,
        duration_minutes: i32,
    ) -> Result<()> {
        let db_path = config::get_db_path(app_handle);
        let conn = Connection::open(&db_path)?;

        let today = Utc::now().format("%Y-%m-%d").to_string();
        let id = Utc::now().timestamp_millis();

        conn.execute(
            "INSERT INTO writing_records (id, project_id, date, total_words, duration) 
             VALUES (?1, ?2, ?3, ?4, ?5)",
            (id, project_id, &today, words_written, duration_minutes),
        )?;

        info!(project_id = %project_id, words = %words_written, "写作记录已保存");
        Ok(())
    }
}
