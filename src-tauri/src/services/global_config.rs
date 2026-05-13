use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

const GLOBAL_CONFIG_FILENAME: &str = "project.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GlobalConfig {
    pub encrypted: bool,
    pub password_hash: String,
    pub language: Vec<LanguageOption>,
    pub locale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageOption {
    pub value: String,
    pub label: String,
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            encrypted: false,
            password_hash: String::new(),
            language: vec![
                LanguageOption {
                    value: "zh-CN".to_string(),
                    label: "简体中文".to_string(),
                },
                LanguageOption {
                    value: "en-US".to_string(),
                    label: "English".to_string(),
                },
                LanguageOption {
                    value: "zh-TW".to_string(),
                    label: "繁体中文".to_string(),
                },
            ],
            locale: "zh-CN".to_string(),
        }
    }
}

fn get_global_config_path(app: &AppHandle) -> PathBuf {
    if let Ok(resource_dir) = app.path().resource_dir() {
        let config_path = resource_dir.join("resources").join(GLOBAL_CONFIG_FILENAME);
        if config_path.exists() {
            return config_path;
        }
        return config_path;
    }
    PathBuf::from("src-tauri/resources").join(GLOBAL_CONFIG_FILENAME)
}

pub fn ensure_global_config(app: &AppHandle) -> Result<GlobalConfig, String> {
    let config_path = get_global_config_path(app);

    if config_path.exists() {
        let content =
            fs::read_to_string(&config_path).map_err(|e| format!("读取全局配置文件失败: {}", e))?;
        return serde_json::from_str(&content).map_err(|e| format!("解析全局配置文件失败: {}", e));
    }

    let source_config_path = PathBuf::from("src-tauri/resources").join(GLOBAL_CONFIG_FILENAME);

    if source_config_path.exists() {
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("创建 resources 目录失败: {}", e))?;
        }
        fs::copy(&source_config_path, &config_path)
            .map_err(|e| format!("复制配置文件失败: {}", e))?;
        let content =
            fs::read_to_string(&config_path).map_err(|e| format!("读取全局配置文件失败: {}", e))?;
        return serde_json::from_str(&content).map_err(|e| format!("解析全局配置文件失败: {}", e));
    }

    let config = GlobalConfig::default();

    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("创建 resources 目录失败: {}", e))?;
    }

    let json =
        serde_json::to_string_pretty(&config).map_err(|e| format!("序列化全局配置失败: {}", e))?;
    fs::write(&config_path, json).map_err(|e| format!("写入全局配置文件失败: {}", e))?;

    Ok(config)
}

pub fn read_global_config(app: &AppHandle) -> Result<GlobalConfig, String> {
    let config_path = get_global_config_path(app);

    if config_path.exists() {
        let content =
            fs::read_to_string(&config_path).map_err(|e| format!("读取全局配置文件失败: {}", e))?;
        return serde_json::from_str(&content).map_err(|e| format!("解析全局配置文件失败: {}", e));
    }
    Ok(GlobalConfig::default())
}

pub fn write_global_config(app: &AppHandle, config: &GlobalConfig) -> Result<(), String> {
    let config_path = get_global_config_path(app);
    let json =
        serde_json::to_string_pretty(config).map_err(|e| format!("序列化全局配置失败: {}", e))?;

    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {}", e))?;
    }

    fs::write(&config_path, json).map_err(|e| format!("写入全局配置文件失败: {}", e))
}

pub fn hash_password(password: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| format!("密码哈希失败: {}", e))?;

    Ok(hash.to_string())
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, String> {
    if hash.is_empty() {
        return Ok(false);
    }

    let parsed_hash = PasswordHash::new(hash).map_err(|e| format!("解析密码哈希失败: {}", e))?;

    let argon2 = Argon2::default();

    Ok(argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

pub fn enable_encryption(app: &AppHandle, password: &str) -> Result<(), String> {
    let mut config = ensure_global_config(app)?;

    if config.encrypted {
        return Err("加密功能已启用".to_string());
    }

    let hash = hash_password(password)?;

    config.encrypted = true;
    config.password_hash = hash;

    write_global_config(app, &config)
}

pub fn disable_encryption(app: &AppHandle, password: &str) -> Result<(), String> {
    let config = read_global_config(app)?;

    if !config.encrypted {
        return Err("加密功能未启用".to_string());
    }

    if !verify_password(password, &config.password_hash)? {
        return Err("密码错误".to_string());
    }

    let new_config = GlobalConfig {
        encrypted: false,
        password_hash: String::new(),
        language: config.language,
        locale: config.locale,
    };

    write_global_config(app, &new_config)
}

pub fn change_password(
    app: &AppHandle,
    old_password: &str,
    new_password: &str,
) -> Result<(), String> {
    let mut config = read_global_config(app)?;

    if !config.encrypted {
        return Err("加密功能未启用".to_string());
    }

    if !verify_password(old_password, &config.password_hash)? {
        return Err("旧密码错误".to_string());
    }

    let hash = hash_password(new_password)?;
    config.password_hash = hash;

    write_global_config(app, &config)
}

pub fn get_encryption_status(app: &AppHandle) -> Result<bool, String> {
    let config = read_global_config(app)?;
    Ok(config.encrypted)
}

pub fn get_language_list(app: &AppHandle) -> Result<Vec<LanguageOption>, String> {
    let config = ensure_global_config(app)?;
    Ok(config.language)
}

pub fn get_locale(app: &AppHandle) -> Result<String, String> {
    let config = ensure_global_config(app)?;
    Ok(config.locale)
}

pub fn set_locale(app: &AppHandle, locale: &str) -> Result<(), String> {
    let mut config = ensure_global_config(app)?;

    let locale_exists = config.language.iter().any(|l| l.value == locale);
    if !locale_exists {
        return Err(format!("不支持的语言: {}", locale));
    }

    config.locale = locale.to_string();
    write_global_config(app, &config)
}
