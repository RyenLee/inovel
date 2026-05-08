use std::path::PathBuf;
use std::sync::OnceLock;
use tauri::AppHandle;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{EnvFilter, Layer, fmt, layer::SubscriberExt, util::SubscriberInitExt};

static LOG_GUARDS: OnceLock<Vec<WorkerGuard>> = OnceLock::new();
static LOG_INIT: OnceLock<()> = OnceLock::new();

pub struct LogConfig {
    pub log_dir: PathBuf,
    pub app_name: String,
    pub max_log_files: usize,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            log_dir: PathBuf::from("logs"),
            app_name: "iNovel".to_string(),
            max_log_files: 7,
        }
    }
}

pub fn init_logging(config: LogConfig) -> Result<(), String> {
    if LOG_INIT.get().is_some() {
        return Ok(());
    }

    let _ = std::fs::create_dir_all(&config.log_dir);

    let (file_non_blocking, file_guard) = tracing_appender::non_blocking(
        tracing_appender::rolling::daily(&config.log_dir, "app.log"),
    );

    let error_appender = super::error_log::ErrorLogWriter::new(&super::error_log::ErrorLogConfig {
        log_dir: config.log_dir.clone(),
        ..Default::default()
    });

    let (error_non_blocking, error_guard) = tracing_appender::non_blocking(error_appender);

    if LOG_GUARDS.set(vec![file_guard, error_guard]).is_err() {
        return Err("日志初始化失败：重复初始化".to_string());
    }

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        if cfg!(debug_assertions) {
            EnvFilter::new("inovel=debug,wry=info,tauri=debug")
        } else {
            EnvFilter::new("inovel=info,wry=warn,tauri=info")
        }
    });

    let console_layer = fmt::layer()
        .with_target(true)
        .with_thread_ids(cfg!(debug_assertions))
        .with_file(cfg!(debug_assertions))
        .with_line_number(cfg!(debug_assertions))
        .with_ansi(true);

    let file_layer = fmt::layer()
        .with_writer(file_non_blocking)
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .with_ansi(false)
        .json();

    let error_layer = fmt::layer()
        .with_writer(error_non_blocking)
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .with_ansi(false)
        .json()
        .with_filter(tracing_subscriber::filter::LevelFilter::ERROR);

    tracing_subscriber::registry()
        .with(env_filter)
        .with(console_layer)
        .with(file_layer)
        .with(error_layer)
        .init();

    let _ = LOG_INIT.set(());

    Ok(())
}

pub fn init_logging_with_app(app_handle: &AppHandle) -> Result<(), String> {
    use crate::config;

    let log_dir = config::get_log_dir(app_handle);
    let config = LogConfig {
        log_dir,
        app_name: "iNovel".to_string(),
        ..Default::default()
    };

    init_logging(config)?;

    super::operation::init_operation_log_db(app_handle)?;
    super::enum_dict::init_enum_dictionary(app_handle)?;

    Ok(())
}
