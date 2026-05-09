use super::model::AppConfig;
use super::encryption::{decrypt, encrypt};
use serde_json;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

const CONFIG_FILE_NAME: &str = "app_config.json";

pub fn get_config_path() -> Result<std::path::PathBuf, String> {
    let mut path = dirs::config_dir().ok_or_else(|| "Unable to get config directory".to_string())?;
    path.push("inovel");
    path.push(CONFIG_FILE_NAME);
    Ok(path)
}

pub fn load_config() -> Result<AppConfig, String> {
    let path = get_config_path()?;
    
    if !path.exists() {
        return Ok(AppConfig::default());
    }
    
    let mut file = File::open(&path).map_err(|e| format!("Failed to open config file: {}", e))?;
    let mut content = String::new();
    file.read_to_string(&mut content)
        .map_err(|e| format!("Failed to read config file: {}", e))?;
    
    let mut config: AppConfig = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse config file: {}", e))?;
    
    for (_, value) in config.items.iter_mut() {
        if value.encrypted && !value.value.is_empty() {
            value.value = decrypt(&value.value)?;
        }
    }
    
    Ok(config)
}

pub fn save_config(config: &AppConfig) -> Result<(), String> {
    let path = get_config_path()?;
    
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create config directory: {}", e))?;
    }
    
    let mut config_to_save = config.clone();
    
    for (_, value) in config_to_save.items.iter_mut() {
        if value.encrypted && !value.value.is_empty() {
            value.value = encrypt(&value.value)?;
        }
    }
    
    let content = serde_json::to_string_pretty(&config_to_save)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
    
    let mut file = File::create(&path).map_err(|e| format!("Failed to create config file: {}", e))?;
    file.write_all(content.as_bytes())
        .map_err(|e| format!("Failed to write config file: {}", e))?;
    
    Ok(())
}

pub fn export_config(config: &AppConfig, export_path: &str) -> Result<(), String> {
    let path = Path::new(export_path);
    
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(|e| format!("Failed to create export directory: {}", e))?;
        }
    }
    
    let content = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
    
    fs::write(path, content).map_err(|e| format!("Failed to write export file: {}", e))?;
    
    Ok(())
}

pub fn import_config(import_path: &str) -> Result<AppConfig, String> {
    let content = fs::read_to_string(import_path)
        .map_err(|e| format!("Failed to read import file: {}", e))?;
    
    let mut config: AppConfig = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse import file: {}", e))?;
    
    for (_, value) in config.items.iter_mut() {
        if value.encrypted && !value.value.is_empty() {
            value.value = decrypt(&value.value)?;
        }
    }
    
    Ok(config)
}

pub fn reset_config() -> Result<AppConfig, String> {
    let default_config = AppConfig::default();
    save_config(&default_config)?;
    Ok(default_config)
}