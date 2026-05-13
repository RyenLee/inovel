use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct ProjectConfig {
    pub name: String,
    pub author: String,
    pub description: String,
    pub project_id: String,
    pub cover_path: Option<String>,
    pub encrypted: bool,
    pub writing_goal: i32,

    #[serde(flatten)]
    pub extra_fields: serde_json::Map<String, serde_json::Value>,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            name: "Untitled Project".to_string(),
            author: "".to_string(),
            description: "".to_string(),
            project_id: "".to_string(),
            cover_path: None,
            encrypted: false,
            writing_goal: 3000,
            extra_fields: serde_json::Map::new(),
        }
    }
}

impl ProjectConfig {
    pub fn new(name: &str, author: &str, description: &str, project_id: &str) -> Self {
        Self {
            name: name.to_string(),
            author: author.to_string(),
            description: description.to_string(),
            project_id: project_id.to_string(),
            ..Default::default()
        }
    }

    pub fn read_from_path(project_path: &Path) -> Result<Self, AppError> {
        let project_json_path = project_path.join("project.json");

        if !project_json_path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(&project_json_path)?;

        Self::from_json_string(&content)
    }

    pub fn from_json_string(content: &str) -> Result<Self, AppError> {
        serde_json::from_str(content).map_err(|e| e.into())
    }

    pub fn write_to_path(&self, project_path: &Path) -> Result<(), AppError> {
        let project_json_path = project_path.join("project.json");

        if let Some(parent) = project_json_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let json_string = self.to_json_string()?;

        fs::write(&project_json_path, json_string)?;

        Ok(())
    }

    pub fn to_json_string(&self) -> Result<String, AppError> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    pub fn set_encrypted(&mut self, encrypted: bool) {
        self.encrypted = encrypted;
    }

    pub fn update_basic_info(&mut self, name: &str, author: &str, description: &str) {
        self.name = name.to_string();
        self.author = author.to_string();
        self.description = description.to_string();
    }

    pub fn update_project_id(&mut self, project_id: &str) {
        self.project_id = project_id.to_string();
    }

    pub fn update_cover_path(&mut self, cover_path: Option<&str>) {
        self.cover_path = cover_path.map(|s| s.to_string());
    }

    pub fn update_writing_goal(&mut self, writing_goal: i32) {
        self.writing_goal = writing_goal;
    }

    pub fn update_window_size(&mut self, width: f64, height: f64) {
        self.extra_fields.insert(
            "window_width".to_string(),
            serde_json::Value::Number(
                serde_json::Number::from_f64(width).unwrap_or(serde_json::Number::from(1200)),
            ),
        );
        self.extra_fields.insert(
            "window_height".to_string(),
            serde_json::Value::Number(
                serde_json::Number::from_f64(height).unwrap_or(serde_json::Number::from(800)),
            ),
        );
    }

    pub fn get_window_size(&self) -> Option<(f64, f64)> {
        let width = self
            .extra_fields
            .get("window_width")
            .and_then(|v| v.as_f64());
        let height = self
            .extra_fields
            .get("window_height")
            .and_then(|v| v.as_f64());
        match (width, height) {
            (Some(w), Some(h)) => Some((w, h)),
            _ => None,
        }
    }

    pub fn clear_project_id(&mut self) {
        self.project_id = "".to_string();
    }
}

pub fn read_project_config(project_path: &Path) -> Result<ProjectConfig, AppError> {
    ProjectConfig::read_from_path(project_path)
}

pub fn write_project_config(project_path: &Path, config: &ProjectConfig) -> Result<(), AppError> {
    config.write_to_path(project_path)
}

pub fn update_project_encrypted_status(
    project_path: &Path,
    encrypted: bool,
) -> Result<(), AppError> {
    let mut config = ProjectConfig::read_from_path(project_path)?;
    config.set_encrypted(encrypted);
    config.write_to_path(project_path)
}

pub fn update_project_basic_info(
    project_path: &Path,
    name: &str,
    author: &str,
    description: &str,
) -> Result<(), AppError> {
    let mut config = ProjectConfig::read_from_path(project_path)?;
    config.update_basic_info(name, author, description);
    config.write_to_path(project_path)
}

pub fn update_project_project_id(project_path: &Path, project_id: &str) -> Result<(), AppError> {
    let mut config = ProjectConfig::read_from_path(project_path)?;
    config.update_project_id(project_id);
    config.write_to_path(project_path)
}

pub fn update_project_window_size(
    project_path: &Path,
    width: f64,
    height: f64,
) -> Result<(), AppError> {
    let mut config = ProjectConfig::read_from_path(project_path)?;
    config.update_window_size(width, height);
    config.write_to_path(project_path)
}

pub fn get_project_window_size(project_path: &Path) -> Result<Option<(f64, f64)>, AppError> {
    let config = ProjectConfig::read_from_path(project_path)?;
    Ok(config.get_window_size())
}

pub fn clear_project_project_id(project_path: &Path) -> Result<(), AppError> {
    let mut config = ProjectConfig::read_from_path(project_path)?;
    config.clear_project_id();
    config.write_to_path(project_path)
}
