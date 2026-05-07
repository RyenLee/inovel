use lru::LruCache;
use std::num::NonZeroUsize;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use crate::settings::CacheConfig;

/// 缓存条目
#[derive(Debug, Clone)]
struct CacheEntry {
    /// 缓存的值（JSON 字符串）
    value: String,
    /// 创建时间
    created_at: Instant,
    /// 过期时间
    ttl: Duration,
}

impl CacheEntry {
    fn new(value: String, ttl: Duration) -> Self {
        Self {
            value,
            created_at: Instant::now(),
            ttl,
        }
    }

    fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.ttl
    }
}

/// 缓存统计信息
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct CacheStats {
    /// 总查询次数
    pub total_lookups: u64,
    /// 命中次数
    pub hits: u64,
    /// 未命中次数
    pub misses: u64,
    /// 当前缓存条目数
    pub current_entries: usize,
    /// 最大容量
    pub max_capacity: usize,
}

impl CacheStats {
    /// 获取缓存命中率
    pub fn hit_rate(&self) -> f64 {
        if self.total_lookups == 0 {
            return 0.0;
        }
        (self.hits as f64 / self.total_lookups as f64) * 100.0
    }
}

/// 命令响应缓存
///
/// 基于 LRU 策略的内存缓存，支持 TTL 过期和命中率统计。
/// 用于缓存频繁访问的 Tauri 命令响应数据。
pub struct ResponseCache {
    /// LRU 缓存（键为命令名+参数哈希，值为缓存条目）
    cache: Mutex<LruCache<String, CacheEntry>>,
    /// 缓存统计
    stats: Mutex<CacheStats>,
    /// 默认 TTL
    default_ttl: Duration,
    /// 是否启用
    enabled: bool,
    /// 需要缓存的命令列表
    cached_commands: Vec<String>,
}

impl ResponseCache {
    /// 创建新的响应缓存
    ///
    /// # 参数
    /// - `config`: 缓存配置
    pub fn new(config: CacheConfig) -> Self {
        let capacity = if config.max_entries == 0 {
            NonZeroUsize::new(1).unwrap()
        } else {
            NonZeroUsize::new(config.max_entries).unwrap_or(NonZeroUsize::new(1).unwrap())
        };

        Self {
            cache: Mutex::new(LruCache::new(capacity)),
            stats: Mutex::new(CacheStats {
                max_capacity: config.max_entries,
                ..Default::default()
            }),
            default_ttl: Duration::from_secs(config.ttl_seconds),
            enabled: config.enabled,
            cached_commands: config.cached_commands,
        }
    }

    /// 生成缓存键
    ///
    /// # 参数
    /// - `command`: 命令名称
    /// - `params`: 参数标识（如项目 ID 等）
    ///
    /// # 返回值
    /// 缓存键字符串
    pub fn make_key(command: &str, params: &str) -> String {
        format!("{}:{}", command, params)
    }

    /// 判断命令是否应该被缓存
    ///
    /// # 参数
    /// - `command`: 命令名称
    ///
    /// # 返回值
    /// 是否应该缓存
    pub fn should_cache(&self, command: &str) -> bool {
        if !self.enabled {
            return false;
        }
        if self.cached_commands.is_empty() {
            return true;
        }
        self.cached_commands
            .iter()
            .any(|c| c == command || (c.ends_with('*') && command.starts_with(&c[..c.len() - 1])))
    }

    /// 从缓存获取值
    ///
    /// # 参数
    /// - `key`: 缓存键
    ///
    /// # 返回值
    /// 命中返回缓存值，未命中或过期返回 None
    pub fn get(&self, key: &str) -> Option<String> {
        if !self.enabled {
            return None;
        }

        let mut stats = self.stats.lock().unwrap();
        stats.total_lookups += 1;

        let mut cache = self.cache.lock().unwrap();
        match cache.get(key) {
            Some(entry) if !entry.is_expired() => {
                stats.hits += 1;
                Some(entry.value.clone())
            }
            Some(_) => {
                stats.misses += 1;
                cache.pop(key);
                None
            }
            None => {
                stats.misses += 1;
                None
            }
        }
    }

    /// 将值存入缓存
    ///
    /// # 参数
    /// - `key`: 缓存键
    /// - `value`: 缓存值
    /// - `ttl`: 过期时间（None 则使用默认 TTL）
    pub fn put(&self, key: &str, value: String, ttl: Option<Duration>) {
        if !self.enabled {
            return;
        }

        let entry = CacheEntry::new(value, ttl.unwrap_or(self.default_ttl));
        let mut cache = self.cache.lock().unwrap();
        cache.put(key.to_string(), entry);

        let mut stats = self.stats.lock().unwrap();
        stats.current_entries = cache.len();
    }

    /// 使指定缓存条目失效
    ///
    /// # 参数
    /// - `key`: 缓存键
    pub fn invalidate(&self, key: &str) {
        let mut cache = self.cache.lock().unwrap();
        cache.pop(key);
        let mut stats = self.stats.lock().unwrap();
        stats.current_entries = cache.len();
    }

    /// 按前缀批量失效缓存
    ///
    /// # 参数
    /// - `prefix`: 缓存键前缀
    pub fn invalidate_prefix(&self, prefix: &str) {
        let mut cache = self.cache.lock().unwrap();
        let keys_to_remove: Vec<String> = cache
            .iter()
            .filter(|(k, _)| k.starts_with(prefix))
            .map(|(k, _)| k.clone())
            .collect();
        for key in keys_to_remove {
            cache.pop(&key);
        }
        let mut stats = self.stats.lock().unwrap();
        stats.current_entries = cache.len();
    }

    /// 清空所有缓存
    pub fn clear(&self) {
        let mut cache = self.cache.lock().unwrap();
        cache.clear();
        let mut stats = self.stats.lock().unwrap();
        stats.current_entries = 0;
    }

    /// 获取缓存统计信息
    pub fn get_stats(&self) -> CacheStats {
        self.stats.lock().unwrap().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_cache() -> ResponseCache {
        let config = CacheConfig {
            enabled: true,
            max_entries: 100,
            ttl_seconds: 60,
            cached_commands: vec![],
        };
        ResponseCache::new(config)
    }

    #[test]
    fn test_cache_put_and_get() {
        let cache = create_test_cache();
        let key = ResponseCache::make_key("test_cmd", "123");
        cache.put(&key, r#"{"result":"ok"}"#.to_string(), None);

        let result = cache.get(&key);
        assert_eq!(result, Some(r#"{"result":"ok"}"#.to_string()));
    }

    #[test]
    fn test_cache_miss() {
        let cache = create_test_cache();
        let result = cache.get("nonexistent");
        assert_eq!(result, None);
    }

    #[test]
    fn test_cache_expiry() {
        let cache = create_test_cache();
        let key = ResponseCache::make_key("test_cmd", "123");
        cache.put(
            &key,
            r#"{"result":"ok"}"#.to_string(),
            Some(Duration::from_millis(1)),
        );

        std::thread::sleep(Duration::from_millis(10));
        let result = cache.get(&key);
        assert_eq!(result, None);
    }

    #[test]
    fn test_cache_invalidate() {
        let cache = create_test_cache();
        let key = ResponseCache::make_key("test_cmd", "123");
        cache.put(&key, r#"{"result":"ok"}"#.to_string(), None);
        cache.invalidate(&key);

        let result = cache.get(&key);
        assert_eq!(result, None);
    }

    #[test]
    fn test_cache_invalidate_prefix() {
        let cache = create_test_cache();
        cache.put("project:1:data", r#"{"a":1}"#.to_string(), None);
        cache.put("project:2:data", r#"{"b":2}"#.to_string(), None);
        cache.put("chapter:1:data", r#"{"c":3}"#.to_string(), None);

        cache.invalidate_prefix("project:");

        assert_eq!(cache.get("project:1:data"), None);
        assert_eq!(cache.get("project:2:data"), None);
        assert_eq!(cache.get("chapter:1:data"), Some(r#"{"c":3}"#.to_string()));
    }

    #[test]
    fn test_cache_stats() {
        let cache = create_test_cache();
        let key = ResponseCache::make_key("test_cmd", "123");

        cache.get(&key); // miss
        cache.put(&key, r#"{"result":"ok"}"#.to_string(), None);
        cache.get(&key); // hit
        cache.get(&key); // hit

        let stats = cache.get_stats();
        assert_eq!(stats.total_lookups, 3);
        assert_eq!(stats.hits, 2);
        assert_eq!(stats.misses, 1);
    }

    #[test]
    fn test_should_cache_with_filter() {
        let config = CacheConfig {
            enabled: true,
            max_entries: 100,
            ttl_seconds: 60,
            cached_commands: vec!["get_chapter_tree".to_string(), "list_*".to_string()],
        };
        let cache = ResponseCache::new(config);

        assert!(cache.should_cache("get_chapter_tree"));
        assert!(cache.should_cache("list_characters"));
        assert!(cache.should_cache("list_locations"));
        assert!(!cache.should_cache("save_chapter_content"));
    }

    #[test]
    fn test_disabled_cache() {
        let config = CacheConfig {
            enabled: false,
            max_entries: 100,
            ttl_seconds: 60,
            cached_commands: vec![],
        };
        let cache = ResponseCache::new(config);
        let key = ResponseCache::make_key("test_cmd", "123");
        cache.put(&key, r#"{"result":"ok"}"#.to_string(), None);

        assert_eq!(cache.get(&key), None);
    }
}
