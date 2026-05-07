use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use crate::settings::RequestMergingConfig;

/// 请求合并条目
struct MergeEntry {
    /// 首次请求时间
    first_request_at: Instant,
    /// 合并窗口
    window: Duration,
    /// 请求参数列表
    params: Vec<String>,
}

/// 请求合并器
///
/// 在指定时间窗口内合并相同命令的多次调用，减少重复处理。
/// 适用于高频操作如自动保存、实时搜索等场景。
pub struct RequestMerger {
    /// 待合并的请求队列
    pending: Mutex<HashMap<String, MergeEntry>>,
    /// 合并窗口时间
    window: Duration,
    /// 最大批量大小
    max_batch_size: usize,
    /// 是否启用
    enabled: bool,
}

impl RequestMerger {
    /// 创建新的请求合并器
    ///
    /// # 参数
    /// - `config`: 请求合并配置
    pub fn new(config: RequestMergingConfig) -> Self {
        Self {
            pending: Mutex::new(HashMap::new()),
            window: Duration::from_millis(config.window_ms),
            max_batch_size: config.max_batch_size,
            enabled: config.enabled,
        }
    }

    /// 尝试合并请求
    ///
    /// 如果请求在合并窗口内且未达到批量上限，则合并并返回 None。
    /// 如果窗口已过期或达到批量上限，则返回所有合并的参数。
    ///
    /// # 参数
    /// - `command`: 命令名称
    /// - `param`: 请求参数标识
    ///
    /// # 返回值
    /// - `None`: 请求已合并，等待更多请求
    /// - `Some(params)`: 应执行请求，返回所有合并的参数
    pub fn try_merge(&self, command: &str, param: String) -> Option<Vec<String>> {
        if !self.enabled {
            return Some(vec![param]);
        }

        let mut pending = self.pending.lock().unwrap();

        if let Some(entry) = pending.get_mut(command) {
            if entry.first_request_at.elapsed() >= entry.window {
                let params = std::mem::take(&mut entry.params);
                pending.remove(command);
                
                let mut result = params;
                if !result.contains(&param) {
                    result.push(param);
                }
                return Some(result);
            }
            
            if !entry.params.contains(&param) {
                entry.params.push(param);
            }
            
            if entry.params.len() >= self.max_batch_size {
                let params = std::mem::take(&mut entry.params);
                pending.remove(command);
                return Some(params);
            }
            
            return None;
        }

        pending.insert(
            command.to_string(),
            MergeEntry {
                first_request_at: Instant::now(),
                window: self.window,
                params: vec![param],
            },
        );

        None
    }

    /// 强制刷新指定命令的合并队列
    ///
    /// # 参数
    /// - `command`: 命令名称
    ///
    /// # 返回值
    /// 所有合并的参数
    pub fn flush(&self, command: &str) -> Option<Vec<String>> {
        let mut pending = self.pending.lock().unwrap();
        pending.remove(command).map(|entry| entry.params)
    }

    /// 强制刷新所有合并队列
    ///
    /// # 返回值
    /// 所有命令及其合并的参数
    pub fn flush_all(&self) -> HashMap<String, Vec<String>> {
        let mut pending = self.pending.lock().unwrap();
        let result: HashMap<String, Vec<String>> = pending
            .drain()
            .map(|(k, v)| (k, v.params))
            .collect();
        result
    }

    /// 获取当前待合并的请求数
    pub fn pending_count(&self) -> usize {
        let pending = self.pending.lock().unwrap();
        pending.values().map(|e| e.params.len()).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_merger() -> RequestMerger {
        let config = RequestMergingConfig {
            enabled: true,
            window_ms: 100,
            max_batch_size: 5,
        };
        RequestMerger::new(config)
    }

    #[test]
    fn test_first_request_merged() {
        let merger = create_merger();
        let result = merger.try_merge("save_chapter", "chapter_1".to_string());
        assert_eq!(result, None);
    }

    #[test]
    fn test_second_request_merged() {
        let merger = create_merger();
        assert_eq!(merger.try_merge("save_chapter", "chapter_1".to_string()), None);
        assert_eq!(merger.try_merge("save_chapter", "chapter_2".to_string()), None);
    }

    #[test]
    fn test_window_expired_returns_batch() {
        let merger = create_merger();
        assert_eq!(merger.try_merge("save_chapter", "chapter_1".to_string()), None);

        std::thread::sleep(Duration::from_millis(150));

        let result = merger.try_merge("save_chapter", "chapter_2".to_string());
        assert!(result.is_some());
        let params = result.unwrap();
        assert_eq!(params, vec!["chapter_1", "chapter_2"]);
    }

    #[test]
    fn test_max_batch_size_triggers_flush() {
        let config = RequestMergingConfig {
            enabled: true,
            window_ms: 10000,
            max_batch_size: 3,
        };
        let merger = RequestMerger::new(config);

        assert_eq!(merger.try_merge("save", "1".to_string()), None);
        assert_eq!(merger.try_merge("save", "2".to_string()), None);
        let result = merger.try_merge("save", "3".to_string());
        assert!(result.is_some());
        assert_eq!(result.unwrap().len(), 3);
    }

    #[test]
    fn test_disabled_merger() {
        let config = RequestMergingConfig {
            enabled: false,
            window_ms: 100,
            max_batch_size: 5,
        };
        let merger = RequestMerger::new(config);

        let result = merger.try_merge("save", "1".to_string());
        assert_eq!(result, Some(vec!["1".to_string()]));
    }

    #[test]
    fn test_flush_all() {
        let merger = create_merger();
        merger.try_merge("cmd_a", "a1".to_string());
        merger.try_merge("cmd_b", "b1".to_string());

        let all = merger.flush_all();
        assert_eq!(all.len(), 2);
        assert!(all.contains_key("cmd_a"));
        assert!(all.contains_key("cmd_b"));
    }
}
