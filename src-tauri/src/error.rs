use thiserror::Error;

/// 应用程序错误类型枚举
///
/// 统一管理所有业务错误，便于错误处理和日志记录。
/// 使用 `thiserror` 宏自动实现 `std::error::Error` trait。
#[derive(Debug, Error)]
pub enum AppError {
    /// 数据库操作相关错误（如连接失败、查询失败、插入失败等）
    /// 通过 `#[from]` 自动将 `rusqlite::Error` 转换为此类型
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),

    /// 文件系统操作相关错误（如文件读写失败、目录创建失败等）
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON 序列化/反序列化错误
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// 配置文件读取或解析错误
    #[error("configuration error: {0}")]
    Config(String),

    /// 用户输入验证失败（如密码太短、参数格式错误等）
    #[error("validation error: {0}")]
    Validation(String),

    /// 请求的资源未找到（如项目不存在、章节不存在等）
    #[error("not found error: {0}")]
    NotFound(String),

    /// 权限不足（如尝试访问未授权资源）
    #[error("permission error: {0}")]
    Permission(String),

    /// 加密/解密操作失败（如密码错误、密钥派生失败等）
    #[error("encryption error: {0}")]
    Encryption(String),

    /// Git 操作失败（如仓库初始化失败、提交失败等）
    #[error("git error: {0}")]
    Git(String),

    /// 导出操作失败（如生成 EPUB 失败、写入文件失败等）
    #[error("export error: {0}")]
    Export(String),

    /// 备份操作失败（如压缩失败、保存失败等）
    #[error("backup error: {0}")]
    Backup(String),

    /// 未分类的内部错误
    #[error("internal error: {0}")]
    Internal(String),
}

impl AppError {
    /// 创建配置错误
    pub fn config(msg: impl Into<String>) -> Self {
        Self::Config(msg.into())
    }

    /// 创建验证错误
    pub fn validation(msg: impl Into<String>) -> Self {
        Self::Validation(msg.into())
    }

    /// 创建资源未找到错误
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound(msg.into())
    }

    /// 创建权限不足错误
    pub fn permission(msg: impl Into<String>) -> Self {
        Self::Permission(msg.into())
    }

    /// 创建加密错误
    pub fn encryption(msg: impl Into<String>) -> Self {
        Self::Encryption(msg.into())
    }

    /// 创建 Git 操作错误
    pub fn git(msg: impl Into<String>) -> Self {
        Self::Git(msg.into())
    }

    /// 创建导出错误
    pub fn export(msg: impl Into<String>) -> Self {
        Self::Export(msg.into())
    }

    /// 创建备份错误
    pub fn backup(msg: impl Into<String>) -> Self {
        Self::Backup(msg.into())
    }

    /// 创建内部错误
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }
}

/// 实现 `serde::Serialize` trait，使错误可以序列化为 JSON
///
/// 序列化格式：{"type": "error_type", "message": "error_message"}
/// 用于向前端传递错误信息
impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;

        let mut state = serializer.serialize_struct("AppError", 2)?;
        state.serialize_field("type", self.error_type())?;
        state.serialize_field("message", &self.to_string())?;
        state.end()
    }
}

impl AppError {
    /// 获取错误类型的字符串标识（用于 JSON 序列化）
    fn error_type(&self) -> &'static str {
        match self {
            Self::Database(_) => "database",
            Self::Io(_) => "io",
            Self::Serialization(_) => "serialization",
            Self::Config(_) => "config",
            Self::Validation(_) => "validation",
            Self::NotFound(_) => "not_found",
            Self::Permission(_) => "permission",
            Self::Encryption(_) => "encryption",
            Self::Git(_) => "git",
            Self::Export(_) => "export",
            Self::Backup(_) => "backup",
            Self::Internal(_) => "internal",
        }
    }
}

/// 应用程序结果类型别名
///
/// 统一使用 `Result<T, AppError>` 作为所有函数返回类型
pub type Result<T> = std::result::Result<T, AppError>;
