use std::time::Instant;
use tauri::State;
use tracing::info;

use crate::optimization::{OptimizationEngine, PerformanceMonitor};
use crate::settings::AppConfig;
use crate::settings::SharedConfig;

#[tauri::command(rename_all = "snake_case")]
pub async fn get_app_config(config: State<'_, SharedConfig>) -> Result<AppConfig, String> {
    let cfg = config.read().map_err(|e| format!("读取配置失败: {}", e))?;
    Ok(cfg.clone())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn update_app_config(
    new_config: AppConfig,
    config: State<'_, SharedConfig>,
    engine: State<'_, OptimizationEngine>,
) -> Result<AppConfig, String> {
    let validation = crate::settings::validate_config(&new_config);
    if !validation.valid {
        return Err(format!("配置验证失败: {}", validation.errors.join("; ")));
    }

    {
        let mut cfg = config.write().map_err(|e| format!("写入配置失败: {}", e))?;
        *cfg = new_config;
    }

    engine.refresh(&config.inner().clone());

    info!("应用配置已更新");
    let cfg = config.read().map_err(|e| format!("读取配置失败: {}", e))?;
    Ok(cfg.clone())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn reset_app_config(
    config: State<'_, SharedConfig>,
    engine: State<'_, OptimizationEngine>,
) -> Result<AppConfig, String> {
    let default_config = AppConfig::default();
    {
        let mut cfg = config.write().map_err(|e| format!("写入配置失败: {}", e))?;
        *cfg = default_config;
    }
    engine.refresh(&config.inner().clone());

    info!("应用配置已重置为默认值");
    let cfg = config.read().map_err(|e| format!("读取配置失败: {}", e))?;
    Ok(cfg.clone())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_cache_stats(
    engine: State<'_, OptimizationEngine>,
) -> Result<crate::optimization::CacheStats, String> {
    Ok(engine.cache.lock().unwrap().get_stats())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn clear_cache(engine: State<'_, OptimizationEngine>) -> Result<(), String> {
    engine.cache.lock().unwrap().clear();
    info!("缓存已清空");
    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_performance_report(
    engine: State<'_, OptimizationEngine>,
) -> Result<String, String> {
    Ok(engine.monitor.generate_report())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn clear_performance_metrics(
    engine: State<'_, OptimizationEngine>,
) -> Result<(), String> {
    engine.monitor.clear();
    info!("性能指标已清空");
    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn test_gzip_compression(
    content: String,
    engine: State<'_, OptimizationEngine>,
) -> Result<crate::optimization::CompressedData, String> {
    Ok(engine.gzip.lock().unwrap().compress_json(&content))
}

pub fn record_command_metrics(
    monitor: &PerformanceMonitor,
    command: &str,
    start: Instant,
    response_size: usize,
    cache_hit: bool,
    gzip_compressed: bool,
    compressed_size: Option<usize>,
) {
    monitor.record(
        command,
        start,
        response_size,
        cache_hit,
        gzip_compressed,
        compressed_size,
    );
}
