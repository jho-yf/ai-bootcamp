use chrono::{DateTime, Utc};
/// 数据库连接模型
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseConnection {
    /// 唯一标识符（UUID）
    pub id: String,

    /// 用户自定义的连接名称（如 "生产数据库"、"测试环境"）
    pub name: String,

    /// 数据库主机地址
    pub host: String,

    /// 端口号（默认 5432）
    pub port: u16,

    /// 数据库名称
    pub database_name: String,

    /// 用户名
    pub user: String,

    /// 密码（存储时加密）
    #[serde(skip_serializing)]
    pub password: String,

    /// 连接状态
    pub status: ConnectionStatus,

    /// 创建时间
    pub created_at: DateTime<Utc>,

    /// 最后更新时间
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConnectionStatus {
    /// 未连接
    Disconnected,
    /// 连接中
    Connecting,
    /// 已连接
    Connected,
    /// 连接失败
    Failed,
}

/// 添加数据库请求
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddDatabaseRequest {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub database_name: String,
    pub user: String,
    pub password: String,
}

/// 更新数据库请求
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDatabaseRequest {
    pub id: String,
    pub name: Option<String>,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub database_name: Option<String>,
    pub user: Option<String>,
    pub password: Option<String>,
}

/// 测试连接请求
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestConnectionRequest {
    pub host: String,
    pub port: u16,
    pub database_name: String,
    pub user: String,
    pub password: String,
}
