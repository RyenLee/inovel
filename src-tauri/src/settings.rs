use notify::Watcher;
use std::path::PathBuf;
use std::sync::{Arc, atomic::AtomicBool};
use tracing::{error, info, warn};

pub type SharedConfig = Arc<std::sync::RwLock<AppConfig>>;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AppConfig {
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
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            gzip: GzipConfig::default(),
            cache: CacheConfig::default(),
            pagination: PaginationConfig::default(),
            request_merging: RequestMergingConfig::default(),
            api: ApiConfig::default(),
            performance: PerformanceConfig::default(),
            window: WindowConfig::default(),
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

#[derive(Debug, Default)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
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
