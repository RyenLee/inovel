use serde::{Deserialize, Serialize};

use crate::settings::PaginationConfig;

/// 分页请求参数
#[derive(Debug, Clone, Deserialize)]
pub struct PageRequest {
    /// 页码（从 1 开始）
    #[serde(default = "default_page")]
    pub page: usize,
    /// 每页条数
    #[serde(default = "default_page_size")]
    pub page_size: usize,
}

fn default_page() -> usize { 1 }
fn default_page_size() -> usize { 20 }

/// 分页响应数据
#[derive(Debug, Clone, Serialize)]
pub struct PageResponse<T: Serialize> {
    /// 当前页数据
    pub items: Vec<T>,
    /// 总条数
    pub total: usize,
    /// 当前页码
    pub page: usize,
    /// 每页条数
    pub page_size: usize,
    /// 总页数
    pub total_pages: usize,
    /// 是否有上一页
    pub has_prev: bool,
    /// 是否有下一页
    pub has_next: bool,
}

/// 分页辅助工具
///
/// 提供标准化的分页参数校验和分页响应构建功能。
pub struct PaginationHelper {
    config: PaginationConfig,
}

impl PaginationHelper {
    /// 创建新的分页辅助工具
    ///
    /// # 参数
    /// - `config`: 分页配置
    pub fn new(config: PaginationConfig) -> Self {
        Self { config }
    }

    /// 规范化分页参数
    ///
    /// 确保 page 和 page_size 在合理范围内。
    ///
    /// # 参数
    /// - `request`: 原始分页请求
    ///
    /// # 返回值
    /// 规范化后的分页请求
    pub fn normalize(&self, request: &PageRequest) -> PageRequest {
        let page = if request.page < 1 { 1 } else { request.page };
        let page_size = if request.page_size < 1 {
            self.config.default_page_size
        } else if request.page_size > self.config.max_page_size {
            self.config.max_page_size
        } else {
            request.page_size
        };

        PageRequest { page, page_size }
    }

    /// 构建分页响应
    ///
    /// # 参数
    /// - `items`: 当前页数据
    /// - `total`: 总条数
    /// - `request`: 分页请求参数
    ///
    /// # 返回值
    /// 分页响应
    pub fn build_response<T: Serialize + Clone>(
        &self,
        items: Vec<T>,
        total: usize,
        request: &PageRequest,
    ) -> PageResponse<T> {
        let normalized = self.normalize(request);
        let total_pages = if normalized.page_size == 0 {
            0
        } else {
            (total + normalized.page_size - 1) / normalized.page_size
        };

        PageResponse {
            items,
            total,
            page: normalized.page,
            page_size: normalized.page_size,
            total_pages,
            has_prev: normalized.page > 1,
            has_next: normalized.page < total_pages,
        }
    }

    /// 计算 SQL 查询的 OFFSET 和 LIMIT
    ///
    /// # 参数
    /// - `request`: 分页请求参数
    ///
    /// # 返回值
    /// (offset, limit) 元组
    pub fn sql_offset_limit(&self, request: &PageRequest) -> (usize, usize) {
        let normalized = self.normalize(request);
        let offset = (normalized.page - 1) * normalized.page_size;
        (offset, normalized.page_size)
    }

    /// 对内存中的数据进行分页切片
    ///
    /// # 参数
    /// - `items`: 完整数据列表
    /// - `request`: 分页请求参数
    ///
    /// # 返回值
    /// 分页响应
    pub fn paginate_in_memory<T: Serialize + Clone>(
        &self,
        items: &[T],
        request: &PageRequest,
    ) -> PageResponse<T> {
        let normalized = self.normalize(request);
        let total = items.len();
        let (offset, limit) = self.sql_offset_limit(request);

        let page_items: Vec<T> = items
            .iter()
            .skip(offset)
            .take(limit)
            .cloned()
            .collect();

        self.build_response(page_items, total, &normalized)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_helper() -> PaginationHelper {
        let config = PaginationConfig {
            default_page_size: 20,
            max_page_size: 100,
        };
        PaginationHelper::new(config)
    }

    #[test]
    fn test_normalize_defaults() {
        let helper = create_helper();
        let request = PageRequest { page: 0, page_size: 0 };
        let normalized = helper.normalize(&request);
        assert_eq!(normalized.page, 1);
        assert_eq!(normalized.page_size, 20);
    }

    #[test]
    fn test_normalize_max_page_size() {
        let helper = create_helper();
        let request = PageRequest { page: 1, page_size: 500 };
        let normalized = helper.normalize(&request);
        assert_eq!(normalized.page_size, 100);
    }

    #[test]
    fn test_build_response() {
        let helper = create_helper();
        let items = vec!["a", "b", "c"];
        let request = PageRequest { page: 1, page_size: 10 };
        let response = helper.build_response(items, 25, &request);

        assert_eq!(response.items.len(), 3);
        assert_eq!(response.total, 25);
        assert_eq!(response.total_pages, 3);
        assert!(!response.has_prev);
        assert!(response.has_next);
    }

    #[test]
    fn test_sql_offset_limit() {
        let helper = create_helper();
        let request = PageRequest { page: 3, page_size: 20 };
        let (offset, limit) = helper.sql_offset_limit(&request);
        assert_eq!(offset, 40);
        assert_eq!(limit, 20);
    }

    #[test]
    fn test_paginate_in_memory() {
        let helper = create_helper();
        let items: Vec<i32> = (0..100).collect();
        let request = PageRequest { page: 2, page_size: 15 };
        let response = helper.paginate_in_memory(&items, &request);

        assert_eq!(response.items.len(), 15);
        assert_eq!(response.items[0], 15);
        assert_eq!(response.items[14], 29);
        assert_eq!(response.total, 100);
        assert_eq!(response.total_pages, 7);
        assert!(response.has_prev);
        assert!(response.has_next);
    }
}
