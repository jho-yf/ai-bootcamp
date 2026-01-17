/// 应用错误类型定义
/// 统一错误处理，所有错误都转换为 String 返回给前端
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AppError {
    /// 数据库连接错误
    DatabaseConnection(String),
    /// 查询执行错误
    QueryExecution(String),
    /// 元数据提取错误
    MetadataExtraction(String),
    /// AI 服务错误
    AIService(String),
    /// 缓存存储错误
    CacheStorage(String),
    /// 配置错误
    Configuration(String),
    /// 参数验证错误
    Validation(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            AppError::DatabaseConnection(msg) => {
                write!(f, "数据库连接失败：{}", msg)
            }
            AppError::QueryExecution(msg) => {
                write!(f, "查询执行失败：{}", msg)
            }
            AppError::MetadataExtraction(msg) => {
                write!(f, "元数据提取失败：{}", msg)
            }
            AppError::AIService(msg) => {
                write!(f, "AI 服务错误：{}", msg)
            }
            AppError::CacheStorage(msg) => {
                write!(f, "缓存存储失败：{}", msg)
            }
            AppError::Configuration(msg) => {
                write!(f, "配置错误：{}", msg)
            }
            AppError::Validation(msg) => {
                write!(f, "参数验证失败：{}", msg)
            }
        }
    }
}

impl std::error::Error for AppError {}

/// 将 AppError 转换为 String（用于 Tauri Command 返回）
impl From<AppError> for String {
    fn from(err: AppError) -> Self {
        err.to_string()
    }
}

/// 从其他错误类型转换
impl From<tokio_postgres::Error> for AppError {
    fn from(err: tokio_postgres::Error) -> Self {
        AppError::DatabaseConnection(err.to_string())
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(err: rusqlite::Error) -> Self {
        AppError::CacheStorage(err.to_string())
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::Configuration(err.to_string())
    }
}
