use std::sync::{Arc, Mutex};
use std::time::Instant;
use tracing::{info, warn};

use crate::settings::SharedConfig;

pub mod cache;
pub mod gzip;
pub mod merger;
pub mod pagination;

pub use cache::{CacheStats, ResponseCache};
pub use gzip::{CompressedData, GzipCompressor};
pub use merger::RequestMerger;
pub use pagination::{PageRequest, PageResponse, PaginationHelper};

#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub command: String,
    pub duration_ms: u64,
    pub response_size: usize,
    pub cache_hit: bool,
    pub gzip_compressed: bool,
    pub compressed_size: Option<usize>,
    pub timestamp: String,
}

pub struct PerformanceMonitor {
    metrics: std::sync::Mutex<Vec<PerformanceMetrics>>,
    enabled: bool,
    slow_threshold_ms: u64,
}

impl PerformanceMonitor {
    pub fn new(config: &SharedConfig) -> Self {
        let cfg = config.read().unwrap();
        Self {
            metrics: std::sync::Mutex::new(Vec::new()),
            enabled: cfg.performance.monitoring_enabled,
            slow_threshold_ms: cfg.performance.slow_request_threshold_ms,
        }
    }

    pub fn record(
        &self,
        command: &str,
        start: Instant,
        response_size: usize,
        cache_hit: bool,
        gzip_compressed: bool,
        compressed_size: Option<usize>,
    ) {
        if !self.enabled {
            return;
        }

        let duration_ms = start.elapsed().as_millis() as u64;

        if duration_ms > self.slow_threshold_ms {
            warn!(
                "slow request: {} cost {}ms (threshold {}ms)",
                command, duration_ms, self.slow_threshold_ms
            );
        }

        let metric = PerformanceMetrics {
            command: command.to_string(),
            duration_ms,
            response_size,
            cache_hit,
            gzip_compressed,
            compressed_size,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        let mut metrics = self.metrics.lock().unwrap();
        metrics.push(metric);
    }

    pub fn get_metrics(&self) -> Vec<PerformanceMetrics> {
        self.metrics.lock().unwrap().clone()
    }

    pub fn clear(&self) {
        self.metrics.lock().unwrap().clear();
    }

    pub fn generate_report(&self) -> String {
        let metrics = self.metrics.lock().unwrap();
        if metrics.is_empty() {
            return "no performance data".to_string();
        }

        let total_requests = metrics.len();
        let total_duration: u64 = metrics.iter().map(|m| m.duration_ms).sum();
        let avg_duration = total_duration as f64 / total_requests as f64;
        let max_duration = metrics.iter().map(|m| m.duration_ms).max().unwrap_or(0);
        let min_duration = metrics.iter().map(|m| m.duration_ms).min().unwrap_or(0);

        let cache_hits = metrics.iter().filter(|m| m.cache_hit).count();
        let cache_hit_rate = (cache_hits as f64 / total_requests as f64) * 100.0;

        let gzip_count = metrics.iter().filter(|m| m.gzip_compressed).count();
        let total_original: usize = metrics.iter().map(|m| m.response_size).sum();
        let total_compressed: usize = metrics.iter().filter_map(|m| m.compressed_size).sum();
        let savings = if total_original > 0 {
            ((total_original - total_compressed) as f64 / total_original as f64) * 100.0
        } else {
            0.0
        };

        let slow_requests = metrics
            .iter()
            .filter(|m| m.duration_ms > self.slow_threshold_ms)
            .count();

        let mut report = String::new();
        report.push_str("========================================\n");
        report.push_str("        API performance optimization report\n");
        report.push_str("========================================\n\n");

        report.push_str(&format!("total requests:        {}\n", total_requests));
        report.push_str(&format!(
            "average response time:    {:.2} ms\n",
            avg_duration
        ));
        report.push_str(&format!("minimum response time:    {} ms\n", min_duration));
        report.push_str(&format!("maximum response time:    {} ms\n", max_duration));
        report.push_str(&format!(
            "slow requests (>{}ms): {}\n\n",
            self.slow_threshold_ms, slow_requests
        ));

        report.push_str(&format!(
            "缓存命中率:      {:.1}% ({}/{})\n",
            cache_hit_rate, cache_hits, total_requests
        ));
        report.push_str(&format!("Gzip compressed requests: {}\n", gzip_count));
        report.push_str(&format!("total original data size:    {} bytes\n", total_original));
        report.push_str(&format!("compressed size:      {} bytes\n", total_compressed));
        report.push_str(&format!("bandwidth savings:        {:.1}%\n\n", savings));

        report.push_str("========================================\n");

        report
    }
}

pub struct OptimizationEngine {
    pub gzip: Mutex<GzipCompressor>,
    pub cache: Mutex<ResponseCache>,
    pub pagination: Mutex<PaginationHelper>,
    pub merger: Mutex<RequestMerger>,
    pub monitor: Arc<PerformanceMonitor>,
}

impl OptimizationEngine {
    pub fn new(config: &SharedConfig) -> Self {
        let cfg = config.read().unwrap();
        Self {
            gzip: Mutex::new(GzipCompressor::new(cfg.gzip.clone())),
            cache: Mutex::new(ResponseCache::new(cfg.cache.clone())),
            pagination: Mutex::new(PaginationHelper::new(cfg.pagination.clone())),
            merger: Mutex::new(RequestMerger::new(cfg.request_merging.clone())),
            monitor: Arc::new(PerformanceMonitor::new(config)),
        }
    }

    pub fn refresh(&self, config: &SharedConfig) {
        let cfg = config.read().unwrap();
        *self.gzip.lock().unwrap() = GzipCompressor::new(cfg.gzip.clone());
        *self.cache.lock().unwrap() = ResponseCache::new(cfg.cache.clone());
        *self.pagination.lock().unwrap() = PaginationHelper::new(cfg.pagination.clone());
        *self.merger.lock().unwrap() = RequestMerger::new(cfg.request_merging.clone());
        info!("optimization engine configuration refreshed successfully");
    }
}
