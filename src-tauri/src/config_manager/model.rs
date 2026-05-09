use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConfigValue {
    pub value: String,
    pub encrypted: bool,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConfigItem {
    pub key: String,
    pub value: ConfigValue,
    pub category: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConfigSnapshot {
    pub version: String,
    pub items: Vec<ConfigItem>,
    pub created_at: DateTime<Utc>,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConfigHistory {
    pub id: String,
    pub snapshot: ConfigSnapshot,
    pub action: HistoryAction,
    pub operator: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum HistoryAction {
    Created,
    Updated,
    RolledBack,
    Exported,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConfigCategory {
    pub name: String,
    pub label: String,
    pub description: Option<String>,
    pub items: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub version: String,
    pub app_name: String,
    pub api_base_url: String,
    pub environment: Environment,
    pub categories: Vec<ConfigCategory>,
    pub items: HashMap<String, ConfigValue>,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Environment {
    Development,
    Staging,
    Production,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            version: "1.1.0".to_string(),
            app_name: "iNovel".to_string(),
            api_base_url: "http://localhost:8080".to_string(),
            environment: Environment::Development,
            categories: vec![
                ConfigCategory {
                    name: "app".to_string(),
                    label: "应用配置".to_string(),
                    description: Some("应用基础配置".to_string()),
                    items: vec![
                        "app_name".to_string(),
                        "version".to_string(),
                        "environment".to_string(),
                    ],
                },
                ConfigCategory {
                    name: "api".to_string(),
                    label: "API配置".to_string(),
                    description: Some("接口相关配置".to_string()),
                    items: vec!["api_base_url".to_string(), "api_timeout".to_string()],
                },
                ConfigCategory {
                    name: "security".to_string(),
                    label: "安全配置".to_string(),
                    description: Some("敏感配置项".to_string()),
                    items: vec!["api_key".to_string(), "secret_token".to_string()],
                },
                ConfigCategory {
                    name: "feature".to_string(),
                    label: "功能开关".to_string(),
                    description: Some("功能特性配置".to_string()),
                    items: vec!["auto_save_enabled".to_string(), "sync_enabled".to_string()],
                },
            ],
            items: HashMap::from([
                (
                    "app_name".to_string(),
                    ConfigValue {
                        value: "iNovel".to_string(),
                        encrypted: false,
                        description: Some("应用名称".to_string()),
                    },
                ),
                (
                    "version".to_string(),
                    ConfigValue {
                        value: "1.1.0".to_string(),
                        encrypted: false,
                        description: Some("应用版本号".to_string()),
                    },
                ),
                (
                    "environment".to_string(),
                    ConfigValue {
                        value: "development".to_string(),
                        encrypted: false,
                        description: Some("运行环境".to_string()),
                    },
                ),
                (
                    "api_base_url".to_string(),
                    ConfigValue {
                        value: "http://localhost:1420".to_string(),
                        encrypted: false,
                        description: Some("API基础地址".to_string()),
                    },
                ),
                (
                    "api_timeout".to_string(),
                    ConfigValue {
                        value: "30000".to_string(),
                        encrypted: false,
                        description: Some("API超时时间(毫秒)".to_string()),
                    },
                ),
                (
                    "api_key".to_string(),
                    ConfigValue {
                        value: "".to_string(),
                        encrypted: true,
                        description: Some("API密钥".to_string()),
                    },
                ),
                (
                    "secret_token".to_string(),
                    ConfigValue {
                        value: "".to_string(),
                        encrypted: true,
                        description: Some("安全令牌".to_string()),
                    },
                ),
                (
                    "auto_save_enabled".to_string(),
                    ConfigValue {
                        value: "true".to_string(),
                        encrypted: false,
                        description: Some("自动保存开关".to_string()),
                    },
                ),
                (
                    "sync_enabled".to_string(),
                    ConfigValue {
                        value: "false".to_string(),
                        encrypted: false,
                        description: Some("云同步开关".to_string()),
                    },
                ),
            ]),
            last_updated: Utc::now(),
        }
    }
}
