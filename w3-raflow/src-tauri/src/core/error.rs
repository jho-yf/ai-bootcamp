// src-tauri/src/core/error.rs

//! 错误类型定义
//!
//! 统一的错误类型和错误处理机制。

/// 应用错误类型
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    /// 音频错误
    #[error("音频错误: {0}")]
    Audio(#[from] AudioError),

    /// 网络错误
    #[error("网络错误: {0}")]
    Network(#[from] NetworkError),

    /// 配置错误
    #[error("配置错误: {0}")]
    Config(#[from] ConfigError),

    /// 输入错误
    #[error("输入错误: {0}")]
    Input(#[from] InputError),

    /// IO 错误
    #[error("IO错误: {0}")]
    Io(#[from] std::io::Error),

    /// JSON 序列化错误
    #[error("JSON错误: {0}")]
    Json(#[from] serde_json::Error),

    /// TOML 解析错误
    #[error("TOML错误: {0}")]
    Toml(#[from] toml::de::Error),

    /// Tauri 错误
    #[error("Tauri错误: {0}")]
    Tauri(#[from] tauri::Error),

    /// 其他错误
    #[error("{0}")]
    Other(String),
}

/// 类型别名：Result<T> = std::result::Result<T, AppError>
pub type Result<T> = std::result::Result<T, AppError>;

/// 音频错误
#[derive(Debug, thiserror::Error)]
pub enum AudioError {
    /// 设备不可用
    #[error("音频设备不可用: {0}")]
    DeviceUnavailable(String),

    /// 设备初始化失败
    #[error("设备初始化失败: {0}")]
    InitFailed(String),

    /// 录音失败
    #[error("录音失败: {0}")]
    RecordingFailed(String),

    /// 格式不支持
    #[error("音频格式不支持: {0}")]
    UnsupportedFormat(String),

    /// 流创建失败
    #[error("音频流创建失败: {0}")]
    StreamError(String),
}

/// 网络错误
#[derive(Debug, thiserror::Error)]
pub enum NetworkError {
    /// 连接失败
    #[error("连接失败: {0}")]
    ConnectionFailed(String),

    /// 认证失败
    #[error("认证失败: {0}")]
    AuthFailed(String),

    /// 发送失败
    #[error("发送失败: {0}")]
    SendFailed(String),

    /// 接收失败
    #[error("接收失败: {0}")]
    ReceiveFailed(String),

    /// 超时
    #[error("请求超时")]
    Timeout,

    /// API 错误
    #[error("API错误: {code} - {message}")]
    ApiError { code: String, message: String },
}

/// 配置错误
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// 配置文件损坏
    #[error("配置文件损坏: {0}")]
    Corrupted(String),

    /// 配置目录不可访问
    #[error("配置目录不可访问: {0}")]
    DirectoryInaccessible(String),

    /// 配置验证失败
    #[error("配置验证失败: {0}")]
    ValidationFailed(String),

    /// 配置保存失败
    #[error("配置保存失败: {0}")]
    SaveFailed(String),

    /// 配置加载失败
    #[error("配置加载失败: {0}")]
    LoadFailed(String),
}

/// 输入错误
#[derive(Debug, thiserror::Error)]
pub enum InputError {
    /// 键盘输入失败
    #[error("键盘输入失败: {0}")]
    KeyboardFailed(String),

    /// 剪贴板错误
    #[error("剪贴板错误: {0}")]
    ClipboardError(String),

    /// 热键注册失败
    #[error("热键注册失败: {0}")]
    HotkeyRegisterFailed(String),

    /// 无窗口焦点
    #[error("无窗口焦点")]
    NoFocus,

    /// 其他错误
    #[error("{0}")]
    Other(String),
}

impl From<String> for AppError {
    fn from(s: String) -> Self {
        AppError::Other(s)
    }
}

impl From<&str> for AppError {
    fn from(s: &str) -> Self {
        AppError::Other(s.to_string())
    }
}
