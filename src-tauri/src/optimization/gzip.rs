use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use flate2::Compression;
use flate2::read::GzEncoder;
use std::io::Read;

use crate::settings::GzipConfig;

/// Gzip 压缩结果
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CompressedData {
    /// 压缩后的数据（Base64 编码）
    pub data: String,
    /// 原始大小（字节）
    pub original_size: usize,
    /// 压缩后大小（字节）
    pub compressed_size: usize,
    /// 压缩率（百分比，如 75.3 表示压缩到原来的 75.3%）
    pub compression_ratio: f64,
    /// 是否实际进行了压缩
    pub compressed: bool,
}

/// Gzip 压缩器
///
/// 根据配置对数据进行 gzip 压缩，支持压缩级别和最小压缩阈值控制。
pub struct GzipCompressor {
    config: GzipConfig,
}

impl GzipCompressor {
    /// 创建新的 Gzip 压缩器
    ///
    /// # 参数
    /// - `config`: Gzip 压缩配置
    pub fn new(config: GzipConfig) -> Self {
        Self { config }
    }

    /// 压缩 JSON 字符串数据
    ///
    /// 如果数据大小小于 min_size 阈值或压缩未启用，则返回原始数据。
    ///
    /// # 参数
    /// - `data`: 待压缩的 JSON 字符串
    ///
    /// # 返回值
    /// 压缩结果，包含压缩后的 Base64 数据和统计信息
    pub fn compress_json(&self, data: &str) -> CompressedData {
        self.compress(data.as_bytes(), "application/json")
    }

    /// 压缩字节数据
    ///
    /// # 参数
    /// - `data`: 待压缩的字节数据
    /// - `content_type`: 内容类型（用于判断是否需要压缩）
    ///
    /// # 返回值
    /// 压缩结果
    pub fn compress(&self, data: &[u8], content_type: &str) -> CompressedData {
        let original_size = data.len();

        if !self.config.enabled {
            return CompressedData {
                data: String::from_utf8_lossy(data).to_string(),
                original_size,
                compressed_size: original_size,
                compression_ratio: 100.0,
                compressed: false,
            };
        }

        if original_size < self.config.min_size as usize {
            return CompressedData {
                data: String::from_utf8_lossy(data).to_string(),
                original_size,
                compressed_size: original_size,
                compression_ratio: 100.0,
                compressed: false,
            };
        }

        if !self.should_compress_type(content_type) {
            return CompressedData {
                data: String::from_utf8_lossy(data).to_string(),
                original_size,
                compressed_size: original_size,
                compression_ratio: 100.0,
                compressed: false,
            };
        }

        let level = Compression::new(self.config.level);
        let mut encoder = GzEncoder::new(data, level);
        let mut compressed = Vec::new();
        match encoder.read_to_end(&mut compressed) {
            Ok(_) => {
                let compressed_size = compressed.len();
                let encoded = BASE64.encode(&compressed);
                CompressedData {
                    data: encoded,
                    original_size,
                    compressed_size,
                    compression_ratio: (compressed_size as f64 / original_size as f64) * 100.0,
                    compressed: true,
                }
            }
            Err(_) => CompressedData {
                data: String::from_utf8_lossy(data).to_string(),
                original_size,
                compressed_size: original_size,
                compression_ratio: 100.0,
                compressed: false,
            },
        }
    }

    /// 解压 Base64 编码的 gzip 数据
    ///
    /// # 参数
    /// - `encoded`: Base64 编码的压缩数据
    ///
    /// # 返回值
    /// 成功返回解压后的字符串，失败返回错误信息
    pub fn decompress(&self, encoded: &str) -> Result<String, String> {
        let compressed = BASE64
            .decode(encoded)
            .map_err(|e| format!("Base64 解码失败: {}", e))?;

        let mut decoder = flate2::read::GzDecoder::new(&compressed[..]);
        let mut decompressed = String::new();
        decoder
            .read_to_string(&mut decompressed)
            .map_err(|e| format!("Gzip 解压失败: {}", e))?;

        Ok(decompressed)
    }

    /// 判断内容类型是否需要压缩
    fn should_compress_type(&self, content_type: &str) -> bool {
        if self.config.compress_types.is_empty() {
            return true;
        }
        self.config
            .compress_types
            .iter()
            .any(|t| content_type.to_lowercase().contains(&t.to_lowercase()))
    }

    /// 获取压缩统计信息的摘要
    ///
    /// # 参数
    /// - `result`: 压缩结果
    ///
    /// # 返回值
    /// 人类可读的统计摘要
    pub fn format_stats(result: &CompressedData) -> String {
        if result.compressed {
            format!(
                "原始: {} bytes → 压缩: {} bytes (节省 {:.1}%)",
                result.original_size,
                result.compressed_size,
                100.0 - result.compression_ratio
            )
        } else {
            format!("未压缩: {} bytes (低于阈值或未启用)", result.original_size)
        }
    }
}
