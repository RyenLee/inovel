use notify::Watcher;
use std::path::PathBuf;
use std::sync::{Arc, atomic::AtomicBool};
use tracing::{error, info, warn};

pub type SharedConfig = Arc<std::sync::RwLock<AppConfig>>;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub app: AppInfoConfig,
    #[serde(default)]
    pub gzip: GzipConfig,
    #[serde(default)]
    pub cache: CacheConfig,
    #[serde(default)]
    pub pagination: PaginationConfig,
    #[serde(default, rename = "request_merging")]
    pub request_merging: RequestMergingConfig,
    #[serde(default)]
    pub api: ApiConfig,
    #[serde(default)]
    pub performance: PerformanceConfig,
    #[serde(default)]
    pub window: WindowConfig,
    #[serde(default)]
    pub security: SecurityConfig,
    #[serde(default)]
    pub features: FeaturesConfig,
    #[serde(default)]
    pub editor: EditorConfig,
    #[serde(default, rename = "entry_config")]
    pub entry_config: EntryConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            app: AppInfoConfig::default(),
            gzip: GzipConfig::default(),
            cache: CacheConfig::default(),
            pagination: PaginationConfig::default(),
            request_merging: RequestMergingConfig::default(),
            api: ApiConfig::default(),
            performance: PerformanceConfig::default(),
            window: WindowConfig::default(),
            security: SecurityConfig::default(),
            features: FeaturesConfig::default(),
            editor: EditorConfig::default(),
            entry_config: EntryConfig::default(),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AppInfoConfig {
    pub name: String,
    pub version: String,
    pub environment: String,
    pub description: String,
}

impl Default for AppInfoConfig {
    fn default() -> Self {
        Self {
            name: "iNovel".to_string(),
            version: "1.1.0".to_string(),
            environment: "development".to_string(),
            description: "一款现代化的小说创作工具".to_string(),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SecurityConfig {
    pub api_key: String,
    pub secret_token: String,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            api_key: "".to_string(),
            secret_token: "".to_string(),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FeaturesConfig {
    #[serde(rename = "auto_save_enabled")]
    pub auto_save_enabled: bool,
    #[serde(rename = "sync_enabled")]
    pub sync_enabled: bool,
    #[serde(rename = "writing_stats_enabled")]
    pub writing_stats_enabled: bool,
    #[serde(rename = "inspiration_board_enabled")]
    pub inspiration_board_enabled: bool,
}

impl Default for FeaturesConfig {
    fn default() -> Self {
        Self {
            auto_save_enabled: true,
            sync_enabled: false,
            writing_stats_enabled: true,
            inspiration_board_enabled: true,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EditorConfig {
    #[serde(rename = "default_font_size")]
    pub default_font_size: u32,
    #[serde(rename = "default_font")]
    pub default_font: String,
    #[serde(rename = "line_spacing")]
    pub line_spacing: f64,
    #[serde(rename = "show_line_numbers")]
    pub show_line_numbers: bool,
    #[serde(rename = "spell_check_enabled")]
    pub spell_check_enabled: bool,
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            default_font_size: 16,
            default_font: "微软雅黑".to_string(),
            line_spacing: 1.5,
            show_line_numbers: true,
            spell_check_enabled: true,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EntryConfig {
    pub enabled: bool,
    pub display_name: String,
    pub icon: String,
    pub tooltip: String,
    pub locations: Vec<String>,
    pub allowed_roles: Vec<String>,
    pub shortcut_key: String,
    #[serde(rename = "shortcut_modifiers")]
    pub shortcut_modifiers: Vec<String>,
}

impl Default for EntryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            display_name: "配置管理".to_string(),
            icon: "settings".to_string(),
            tooltip: "打开配置管理页面".to_string(),
            locations: vec!["menu_bar".to_string(), "toolbar".to_string()],
            allowed_roles: vec!["admin".to_string(), "advanced".to_string()],
            shortcut_key: "C".to_string(),
            shortcut_modifiers: vec!["Ctrl".to_string(), "Shift".to_string()],
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GzipConfig {
    pub enabled: bool,
    pub level: u32,
    pub min_size: i64,
    pub compress_types: Vec<String>,
}

impl Default for GzipConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            level: 6,
            min_size: 1024,
            compress_types: vec![
                "application/json".to_string(),
                "text/plain".to_string(),
                "text/html".to_string(),
                "text/css".to_string(),
                "text/javascript".to_string(),
                "application/javascript".to_string(),
                "application/xml".to_string(),
                "text/xml".to_string(),
            ],
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CacheConfig {
    pub enabled: bool,
    pub max_entries: usize,
    pub ttl_seconds: u64,
    pub cached_commands: Vec<String>,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_entries: 1000,
            ttl_seconds: 300,
            cached_commands: vec![],
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PaginationConfig {
    pub default_page_size: usize,
    pub max_page_size: usize,
}

impl Default for PaginationConfig {
    fn default() -> Self {
        Self {
            default_page_size: 20,
            max_page_size: 100,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RequestMergingConfig {
    pub enabled: bool,
    pub window_ms: u64,
    pub max_batch_size: usize,
}

impl Default for RequestMergingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            window_ms: 100,
            max_batch_size: 10,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ApiConfig {
    pub base_url: String,
    pub timeout_ms: u64,
    pub max_retries: u32,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:3000".to_string(),
            timeout_ms: 30000,
            max_retries: 3,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PerformanceConfig {
    pub monitoring_enabled: bool,
    pub slow_request_threshold_ms: u64,
    pub log_payload_size: bool,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            monitoring_enabled: true,
            slow_request_threshold_ms: 500,
            log_payload_size: false,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PortraitConfig {
    pub enabled: bool,
    pub default_width: f64,
    pub default_height: f64,
    pub min_width: f64,
    pub min_height: f64,
}

impl Default for PortraitConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_width: 800.0,
            default_height: 1200.0,
            min_width: 600.0,
            min_height: 800.0,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WindowConfig {
    pub default_width: f64,
    pub default_height: f64,
    pub min_width: f64,
    pub min_height: f64,
    pub max_width: f64,
    pub max_height: f64,
    pub resizable: bool,
    #[serde(default)]
    pub portrait: PortraitConfig,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            default_width: 1200.0,
            default_height: 800.0,
            min_width: 600.0,
            min_height: 800.0,
            max_width: 1920.0,
            max_height: 1200.0,
            resizable: true,
            portrait: PortraitConfig::default(),
        }
    }
}

#[derive(Debug)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self {
            valid: true, // 默认应该是有效的，只有发现错误才设为 false
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }
}

impl ValidationResult {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_error(&mut self, error: &str) {
        self.valid = false;
        self.errors.push(error.to_string());
    }

    pub fn add_warning(&mut self, warning: &str) {
        self.warnings.push(warning.to_string());
    }
}

pub fn validate_config(config: &AppConfig) -> ValidationResult {
    let mut result = ValidationResult::new();

    if config.gzip.level > 9 {
        result.add_error("gzip.level 必须在 0-9 范围内");
    }

    if config.gzip.min_size < 0 {
        result.add_error("gzip.min_size 不能为负数");
    }

    if config.cache.max_entries == 0 {
        result.add_error("cache.max_entries 必须大于 0");
    }

    if config.cache.ttl_seconds == 0 {
        result.add_warning("cache.ttl_seconds 为 0，缓存将立即过期");
    }

    if config.pagination.default_page_size == 0 {
        result.add_error("pagination.default_page_size 必须大于 0");
    }

    if config.pagination.max_page_size == 0 {
        result.add_error("pagination.max_page_size 必须大于 0");
    }

    if config.pagination.default_page_size > config.pagination.max_page_size {
        result.add_error("pagination.default_page_size 不能大于 pagination.max_page_size");
    }

    if config.request_merging.window_ms == 0 {
        result.add_error("request_merging.window_ms 必须大于 0");
    }

    if config.request_merging.max_batch_size == 0 {
        result.add_error("request_merging.max_batch_size 必须大于 0");
    }

    if config.api.timeout_ms == 0 {
        result.add_warning("api.timeout_ms 为 0，可能导致无限等待");
    }

    result
}

pub struct ConfigWatcher {
    config_path: PathBuf,
    config: SharedConfig,
    running: Arc<AtomicBool>,
}

impl ConfigWatcher {
    pub fn new(config_path: PathBuf, initial_config: AppConfig) -> (Self, SharedConfig) {
        let config = Arc::new(std::sync::RwLock::new(initial_config));
        let watcher = Self {
            config_path,
            config: config.clone(),
            running: Arc::new(AtomicBool::new(false)),
        };
        (watcher, config)
    }

    pub fn start(&self) -> Result<(), String> {
        if self.running.load(std::sync::atomic::Ordering::SeqCst) {
            return Ok(());
        }

        let config_path = self.config_path.clone();
        let config = self.config.clone();
        let running = self.running.clone();

        running.store(true, std::sync::atomic::Ordering::SeqCst);

        std::thread::Builder::new()
            .name("config-watcher".to_string())
            .spawn(move || {
                info!("配置热加载监听已启动: {:?}", config_path);

                let (tx, rx) = std::sync::mpsc::channel();

                let mut watcher = match notify::recommended_watcher(
                    move |res: Result<notify::Event, notify::Error>| {
                        if let Ok(event) = res {
                            let _ = tx.send(event);
                        }
                    },
                ) {
                    Ok(w) => w,
                    Err(e) => {
                        error!("创建文件监听器失败: {}", e);
                        running.store(false, std::sync::atomic::Ordering::SeqCst);
                        return;
                    }
                };

                if let Err(e) = watcher.watch(&config_path, notify::RecursiveMode::NonRecursive) {
                    error!("监听配置文件失败: {}", e);
                    running.store(false, std::sync::atomic::Ordering::SeqCst);
                    return;
                }

                while running.load(std::sync::atomic::Ordering::SeqCst) {
                    match rx.recv_timeout(std::time::Duration::from_secs(1)) {
                        Ok(event) => {
                            if matches!(
                                event.kind,
                                notify::EventKind::Modify(_) | notify::EventKind::Create(_)
                            ) {
                                info!("检测到配置文件变化，正在重新加载...");
                                match load_from_file(&config_path) {
                                    Ok(new_config) => {
                                        if let Ok(mut current) = config.write() {
                                            *current = new_config;
                                            info!("配置已成功热加载");
                                        }
                                    }
                                    Err(e) => {
                                        warn!("配置重新加载失败，继续使用当前配置: {}", e);
                                    }
                                }
                            }
                        }
                        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                            // 超时，继续循环
                        }
                        Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                            break;
                        }
                    }
                }

                info!("配置热加载监听已停止");
            })
            .map_err(|e| format!("启动配置监听线程失败: {}", e))?;

        Ok(())
    }

    pub fn stop(&self) {
        self.running
            .store(false, std::sync::atomic::Ordering::SeqCst);
    }
}

pub fn load_config() -> (AppConfig, PathBuf) {
    let config_paths = vec![get_exe_dir_config_path(), PathBuf::from("config.toml")];

    for path in &config_paths {
        if path.exists() {
            info!("从 {} 加载配置文件", path.display());
            match load_from_file(path) {
                Ok(config) => return (config, path.clone()),
                Err(e) => {
                    warn!("加载配置文件失败: {}，尝试下一个路径", e);
                }
            }
        }
    }

    info!("未找到配置文件，使用默认配置");
    (AppConfig::default(), PathBuf::from("config.toml"))
}

pub fn load_from_file(path: &PathBuf) -> Result<AppConfig, String> {
    let content = std::fs::read_to_string(path).map_err(|e| format!("读取配置文件失败: {}", e))?;

    let config: AppConfig =
        toml::from_str(&content).map_err(|e| format!("解析配置文件失败: {}", e))?;

    let validation = validate_config(&config);
    if !validation.valid {
        return Err(format!("配置验证失败: {}", validation.errors.join("; ")));
    }

    for warning in &validation.warnings {
        warn!("配置警告: {}", warning);
    }

    Ok(config)
}

pub fn save_to_file(config: &AppConfig, path: &PathBuf) -> Result<(), String> {
    let content = toml::to_string_pretty(config).map_err(|e| format!("序列化配置失败: {}", e))?;

    std::fs::write(path, content).map_err(|e| format!("写入配置文件失败: {}", e))?;

    info!("配置已保存到: {}", path.display());

    sync_app_metadata(config);

    Ok(())
}

pub fn sync_app_metadata(config: &AppConfig) {
    let app_info = &config.app;

    sync_tauri_conf(&app_info.name, &app_info.version);
    sync_cargo_toml(&app_info.name, &app_info.version, &app_info.description);
    sync_package_json(&app_info.name, &app_info.version);
}

fn sync_tauri_conf(name: &str, version: &str) {
    let path = PathBuf::from("tauri.conf.json");
    match std::fs::read_to_string(&path) {
        Ok(content) => {
            let mut json: serde_json::Value = serde_json::from_str(&content).unwrap_or_default();
            if let Some(obj) = json.as_object_mut() {
                obj.insert(
                    "productName".to_string(),
                    serde_json::Value::String(name.to_string()),
                );
                obj.insert(
                    "version".to_string(),
                    serde_json::Value::String(version.to_string()),
                );
            }
            if let Ok(new_content) = serde_json::to_string_pretty(&json) {
                if let Err(e) = std::fs::write(&path, new_content) {
                    warn!("同步 tauri.conf.json 失败: {}", e);
                } else {
                    info!(
                        "已同步 tauri.conf.json: productName={}, version={}",
                        name, version
                    );
                }
            }
        }
        Err(e) => warn!("读取 tauri.conf.json 失败: {}", e),
    }
}

fn sync_cargo_toml(name: &str, version: &str, description: &str) {
    let path = PathBuf::from("Cargo.toml");
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) => {
            warn!("读取 Cargo.toml 失败: {}", e);
            return;
        }
    };

    let package_name = name.to_lowercase();
    let re_name = regex::Regex::new(r#"(?m)^name\s*=\s*"[^"]*""#).unwrap();
    let re_version = regex::Regex::new(r#"(?m)^version\s*=\s*"[^"]*""#).unwrap();
    let re_desc = regex::Regex::new(r#"(?m)^description\s*=\s*"[^"]*""#).unwrap();

    let mut new_content = re_name
        .replace(&content, format!(r#"name = "{}""#, package_name))
        .to_string();
    new_content = re_version
        .replace(&new_content, format!(r#"version = "{}""#, version))
        .to_string();
    new_content = re_desc
        .replace(&new_content, format!(r#"description = "{}""#, description))
        .to_string();

    if new_content != content {
        if let Err(e) = std::fs::write(&path, new_content) {
            warn!("同步 Cargo.toml 失败: {}", e);
        } else {
            info!(
                "已同步 Cargo.toml: name={}, version={}, description={}",
                package_name, version, description
            );
        }
    }
}

fn sync_package_json(name: &str, version: &str) {
    let path = PathBuf::from("../package.json");
    match std::fs::read_to_string(&path) {
        Ok(content) => {
            let mut json: serde_json::Value = serde_json::from_str(&content).unwrap_or_default();
            let package_name = name.to_lowercase();
            if let Some(obj) = json.as_object_mut() {
                obj.insert(
                    "name".to_string(),
                    serde_json::Value::String(package_name.clone()),
                );
                obj.insert(
                    "version".to_string(),
                    serde_json::Value::String(version.to_string()),
                );
            }
            if let Ok(new_content) = serde_json::to_string_pretty(&json) {
                if let Err(e) = std::fs::write(&path, new_content) {
                    warn!("同步 package.json 失败: {}", e);
                } else {
                    info!(
                        "已同步 package.json: name={}, version={}",
                        package_name, version
                    );
                }
            }
        }
        Err(e) => warn!("读取 package.json 失败: {}", e),
    }
}

pub fn get_config_file_path() -> PathBuf {
    let config_paths = vec![get_exe_dir_config_path(), PathBuf::from("config.toml")];

    for path in &config_paths {
        if path.exists() {
            return path.clone();
        }
    }

    PathBuf::from("config.toml")
}

fn get_exe_dir_config_path() -> PathBuf {
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(parent) = exe_path.parent() {
            return parent.join("config.toml");
        }
    }
    PathBuf::from("config.toml")
}

pub fn init_config() -> (SharedConfig, ConfigWatcher) {
    let (config, config_path) = load_config();
    let (watcher, shared_config) = ConfigWatcher::new(config_path, config);

    if let Err(e) = watcher.start() {
        warn!("启动配置热加载失败: {}", e);
    }

    (shared_config, watcher)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert!(config.gzip.enabled);
        assert_eq!(config.gzip.level, 6);
        assert_eq!(config.cache.max_entries, 1000);
        assert_eq!(config.pagination.default_page_size, 20);
    }

    #[test]
    fn test_validate_default_config() {
        let config = AppConfig::default();
        let result = validate_config(&config);
        assert!(result.valid);
    }
}
