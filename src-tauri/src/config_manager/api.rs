use super::model::{AppConfig, ConfigHistory, ConfigItem, ConfigValue};
use super::history::{add_history, create_snapshot, get_history, get_history_by_id, get_history_count};
use super::loader::{export_config, import_config, load_config, reset_config, save_config};
use chrono::Utc;
use std::collections::HashMap;

#[derive(Debug, serde::Serialize)]
pub struct ConfigUpdateResult {
    pub success: bool,
    pub message: String,
    pub updated_items: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct ConfigQueryResult {
    pub success: bool,
    pub config: Option<AppConfig>,
    pub message: String,
}

#[derive(Debug, serde::Serialize)]
pub struct HistoryQueryResult {
    pub success: bool,
    pub history: Vec<ConfigHistory>,
    pub total: usize,
    pub message: String,
}

#[derive(Debug, serde::Serialize)]
pub struct ExportResult {
    pub success: bool,
    pub path: String,
    pub message: String,
}

pub struct ConfigManager {
    config: AppConfig,
}

impl ConfigManager {
    pub fn new() -> Self {
        let config = load_config().unwrap_or_else(|_| AppConfig::default());
        Self { config }
    }

    pub fn get_config(&self) -> AppConfig {
        self.config.clone()
    }

    pub fn get_value(&self, key: &str) -> Option<ConfigValue> {
        self.config.items.get(key).cloned()
    }

    pub fn get_values_by_category(&self, category: &str) -> Vec<ConfigItem> {
        let category_items = self
            .config
            .categories
            .iter()
            .find(|c| c.name == category)
            .map(|c| c.items.clone())
            .unwrap_or_default();

        category_items
            .into_iter()
            .filter_map(|key| {
                self.config.items.get(&key).map(|value| ConfigItem {
                    key: key.clone(),
                    value: value.clone(),
                    category: category.to_string(),
                    updated_at: self.config.last_updated,
                })
            })
            .collect()
    }

    pub fn set_value(&mut self, key: &str, value: &str, encrypted: bool) -> bool {
        if self.config.items.contains_key(key) {
            let old_value = self.config.items[key].value.clone();
            
            self.config.items.get_mut(key).unwrap().value = value.to_string();
            self.config.items.get_mut(key).unwrap().encrypted = encrypted;
            self.config.last_updated = Utc::now();
            
            let config_clone = self.config.clone();
            if save_config(&config_clone).is_err() {
                self.config.items.get_mut(key).unwrap().value = old_value;
                return false;
            }
            
            add_history(
                create_snapshot(&self.config, Some(format!("Updated config: {}", key))),
                super::model::HistoryAction::Updated,
                None,
            );
            
            true
        } else {
            false
        }
    }

    pub fn set_values(&mut self, values: HashMap<String, String>) -> ConfigUpdateResult {
        let mut updated_items = Vec::new();
        
        for (key, value) in values {
            if self.config.items.contains_key(&key) {
                self.config.items.get_mut(&key).unwrap().value = value;
                self.config.last_updated = Utc::now();
                updated_items.push(key);
            }
        }
        
        if !updated_items.is_empty() {
            let config_clone = self.config.clone();
            if save_config(&config_clone).is_err() {
                return ConfigUpdateResult {
                    success: false,
                    message: "Failed to save config".to_string(),
                    updated_items: Vec::new(),
                };
            }
            
            add_history(
                create_snapshot(&self.config, Some(format!("Bulk update: {} items", updated_items.len()))),
                super::model::HistoryAction::Updated,
                None,
            );
        }
        
        ConfigUpdateResult {
            success: true,
            message: "配置更新成功".to_string(),
            updated_items,
        }
    }

    pub fn update_version(&mut self, new_version: &str) -> bool {
        let old_version = self.config.version.clone();
        
        self.config.version = new_version.to_string();
        self.config.last_updated = Utc::now();
        
        let config_clone = self.config.clone();
        if save_config(&config_clone).is_err() {
            self.config.version = old_version;
            return false;
        }
        
        add_history(
            create_snapshot(&self.config, Some(format!("Version updated: {} -> {}", old_version, new_version))),
            super::model::HistoryAction::Updated,
            None,
        );
        
        true
    }

    pub fn reload(&mut self) -> bool {
        match load_config() {
            Ok(config) => {
                self.config = config;
                true
            }
            Err(_) => false,
        }
    }

    pub fn export(&self, path: &str) -> ExportResult {
        match export_config(&self.config, path) {
            Ok(_) => ExportResult {
                success: true,
                path: path.to_string(),
                message: "配置导出成功".to_string(),
            },
            Err(e) => ExportResult {
                success: false,
                path: path.to_string(),
                message: format!("导出失败: {}", e),
            },
        }
    }

    pub fn import(&mut self, path: &str) -> ConfigQueryResult {
        match import_config(path) {
            Ok(config) => {
                let old_snapshot = create_snapshot(&self.config, Some("Before import".to_string()));
                add_history(old_snapshot, super::model::HistoryAction::Updated, None);
                
                self.config = config;
                self.config.last_updated = Utc::now();
                
                let config_clone = self.config.clone();
                if save_config(&config_clone).is_err() {
                    return ConfigQueryResult {
                        success: false,
                        config: None,
                        message: "保存配置失败".to_string(),
                    };
                }
                
                add_history(
                    create_snapshot(&self.config, Some("After import".to_string())),
                    super::model::HistoryAction::Updated,
                    None,
                );
                
                ConfigQueryResult {
                    success: true,
                    config: Some(self.config.clone()),
                    message: "配置导入成功".to_string(),
                }
            }
            Err(e) => ConfigQueryResult {
                success: false,
                config: None,
                message: format!("导入失败: {}", e),
            },
        }
    }

    pub fn reset(&mut self) -> ConfigQueryResult {
        match reset_config() {
            Ok(config) => {
                add_history(
                    create_snapshot(&config, Some("Config reset to default".to_string())),
                    super::model::HistoryAction::Updated,
                    None,
                );
                
                self.config = config;
                
                ConfigQueryResult {
                    success: true,
                    config: Some(self.config.clone()),
                    message: "配置已重置为默认值".to_string(),
                }
            }
            Err(e) => ConfigQueryResult {
                success: false,
                config: None,
                message: format!("重置失败: {}", e),
            },
        }
    }

    pub fn get_history(&self, page: usize, page_size: usize) -> HistoryQueryResult {
        let history = get_history(page, page_size);
        let total = get_history_count();
        
        HistoryQueryResult {
            success: true,
            history,
            total,
            message: "查询成功".to_string(),
        }
    }

    pub fn rollback_to(&mut self, history_id: &str) -> ConfigQueryResult {
        match get_history_by_id(history_id) {
            Some(history_item) => {
                let snapshot = &history_item.snapshot;
                
                let mut new_items = HashMap::new();
                for item in &snapshot.items {
                    new_items.insert(item.key.clone(), item.value.clone());
                }
                
                self.config.items = new_items;
                self.config.version = snapshot.version.clone();
                self.config.last_updated = Utc::now();
                
                let config_clone = self.config.clone();
                if save_config(&config_clone).is_err() {
                    return ConfigQueryResult {
                        success: false,
                        config: None,
                        message: "回滚失败".to_string(),
                    };
                }
                
                add_history(
                    create_snapshot(&self.config, Some(format!("Rolled back to {}", history_id))),
                    super::model::HistoryAction::RolledBack,
                    None,
                );
                
                ConfigQueryResult {
                    success: true,
                    config: Some(self.config.clone()),
                    message: "回滚成功".to_string(),
                }
            }
            None => ConfigQueryResult {
                success: false,
                config: None,
                message: "未找到历史记录".to_string(),
            },
        }
    }
}