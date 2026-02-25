// src-tauri/src/network/optimize.rs

//! 网络通信优化模块
//!
//! 提供优化的网络通信功能，包括批量发送、优化的 Base64 编码等

use base64::Engine;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{Duration, Instant};

use super::protocol::ClientMessage;

/// 优化的 Base64 编码器
///
/// 使用预分配的缓冲区来减少内存分配
pub struct Base64Encoder {
    /// 配置
    config: Base64Config,
}

/// Base64 编码配置
#[derive(Debug, Clone, Copy)]
pub struct Base64Config {
    /// 是否使用 URL 安全格式
    pub url_safe: bool,
}

impl Default for Base64Config {
    fn default() -> Self {
        Self {
            url_safe: false,
        }
    }
}

impl Base64Encoder {
    /// 创建新的编码器
    pub fn new(config: Base64Config) -> Self {
        Self { config }
    }

    /// 编码数据
    pub fn encode(&self, data: &[u8]) -> String {
        if self.config.url_safe {
            base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(data)
        } else {
            base64::engine::general_purpose::STANDARD.encode(data)
        }
    }

    /// 批量编码多个数据块
    ///
    /// 返回一个包含所有编码结果的 Vec
    pub fn encode_batch(&self, data_chunks: &[&[u8]]) -> Vec<String> {
        data_chunks
            .iter()
            .map(|chunk| self.encode(chunk))
            .collect()
    }
}

impl Default for Base64Encoder {
    fn default() -> Self {
        Self::new(Base64Config::default())
    }
}

/// 消息批处理器
///
/// 将多个小消息批量发送以减少网络往返
#[derive(Clone)]
pub struct MessageBatcher {
    /// 待发送的消息
    messages: Arc<Mutex<Vec<ClientMessage>>>,

    /// 批次大小
    batch_size: usize,

    /// 批次超时（毫秒）
    batch_timeout_ms: u64,

    /// 最后发送时间
    last_send: Arc<Mutex<Instant>>,
}

impl MessageBatcher {
    /// 创建新的批处理器
    ///
    /// # 参数
    /// - `batch_size`: 每批最大消息数
    /// - `batch_timeout_ms`: 批次超时时间
    pub fn new(batch_size: usize, batch_timeout_ms: u64) -> Self {
        Self {
            messages: Arc::new(Mutex::new(Vec::with_capacity(batch_size))),
            batch_size,
            batch_timeout_ms,
            last_send: Arc::new(Mutex::new(Instant::now())),
        }
    }

    /// 添加消息到批次
    pub async fn add_message(&self, message: ClientMessage) -> bool {
        let mut messages = self.messages.lock().await;
        messages.push(message);

        // 检查是否达到批次大小
        messages.len() >= self.batch_size
    }

    /// 获取并清空待发送的消息
    pub async fn take_messages(&self) -> Vec<ClientMessage> {
        let mut messages = self.messages.lock().await;
        std::mem::take(&mut *messages)
    }

    /// 检查是否应该发送批次（基于超时）
    pub async fn should_flush(&self) -> bool {
        let last_send = self.last_send.lock().await;
        last_send.elapsed() > Duration::from_millis(self.batch_timeout_ms)
    }

    /// 检查是否有待发送的消息
    pub async fn has_pending(&self) -> bool {
        let messages = self.messages.lock().await;
        !messages.is_empty()
    }

    /// 获取当前待发送消息数量
    pub async fn pending_count(&self) -> usize {
        let messages = self.messages.lock().await;
        messages.len()
    }
}

impl Default for MessageBatcher {
    fn default() -> Self {
        // 默认: 10 个消息一批，50ms 超时
        Self::new(10, 50)
    }
}

/// WebSocket 连接复用器
///
/// 管理 WebSocket 连接的生命周期，支持连接复用
#[derive(Clone)]
pub struct ConnectionManager {
    /// 连接状态
    state: Arc<Mutex<OptConnectionState>>,

    /// 最后连接时间
    last_connect: Arc<Mutex<Option<Instant>>>,

    /// 重连配置
    reconnect_config: ReconnectConfig,
}

/// 连接状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Error,
}

/// 重连配置
#[derive(Debug, Clone, Copy)]
pub struct ReconnectConfig {
    /// 最大重连次数
    pub max_retries: usize,

    /// 初始重连延迟（毫秒）
    pub initial_delay_ms: u64,

    /// 最大重连延迟（毫秒）
    pub max_delay_ms: u64,

    /// 重连倍数
    pub backoff_multiplier: u32,
}

impl Default for ReconnectConfig {
    fn default() -> Self {
        Self {
            max_retries: 5,
            initial_delay_ms: 1000,
            max_delay_ms: 30000,
            backoff_multiplier: 2,
        }
    }
}

impl ConnectionManager {
    /// 创建新的连接管理器
    pub fn new(reconnect_config: ReconnectConfig) -> Self {
        Self {
            state: Arc::new(Mutex::new(OptConnectionState::Disconnected)),
            last_connect: Arc::new(Mutex::new(None)),
            reconnect_config,
        }
    }

    /// 标记连接已建立
    pub async fn mark_connected(&self) {
        let mut state = self.state.lock().await;
        *state = OptConnectionState::Connected;
        *self.last_connect.lock().await = Some(Instant::now());
    }

    /// 标记连接已断开
    pub async fn mark_disconnected(&self) {
        let mut state = self.state.lock().await;
        *state = OptConnectionState::Disconnected;
    }

    /// 标记连接错误
    pub async fn mark_error(&self) {
        let mut state = self.state.lock().await;
        *state = OptConnectionState::Error;
    }

    /// 获取当前连接状态
    pub async fn state(&self) -> OptConnectionState {
        *self.state.lock().await
    }

    /// 检查是否已连接
    pub async fn is_connected(&self) -> bool {
        matches!(self.state().await, OptConnectionState::Connected)
    }

    /// 计算下次重连延迟
    pub fn next_reconnect_delay(&self, attempt: usize) -> Duration {
        let config = self.reconnect_config;

        if attempt >= config.max_retries {
            return Duration::from_millis(config.max_delay_ms);
        }

        let delay_ms = config.initial_delay_ms
            * u64::pow(config.backoff_multiplier as u64, attempt as u32);

        Duration::from_millis(delay_ms.min(config.max_delay_ms))
    }

    /// 获取连接时长
    pub async fn connection_duration(&self) -> Option<Duration> {
        let last_connect = self.last_connect.lock().await;
        last_connect.map(|t| t.elapsed())
    }
}

impl Default for ConnectionManager {
    fn default() -> Self {
        Self::new(ReconnectConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base64_encoder() {
        let encoder = Base64Encoder::new(Base64Config::default());

        let data = b"Hello, World!";
        let encoded = encoder.encode(data);

        assert_eq!(encoded, base64::engine::general_purpose::STANDARD.encode(data));
    }

    #[test]
    fn test_base64_batch_encoding() {
        let encoder = Base64Encoder::new(Base64Config::default());

        let chunks: Vec<&[u8]> = vec![b"Hello", b"World", b"Test!"];
        let results = encoder.encode_batch(&chunks);

        assert_eq!(results.len(), 3);
        assert_eq!(results[0], base64::engine::general_purpose::STANDARD.encode(b"Hello"));
    }

    #[tokio::test]
    async fn test_message_batcher() {
        let batcher = MessageBatcher::new(3, 100);

        // 添加消息
        assert!(!batcher.add_message(ClientMessage::End).await);
        assert!(!batcher.add_message(ClientMessage::End).await);

        // 第三条消息达到批次大小
        assert!(batcher.add_message(ClientMessage::End).await);

        // 获取消息
        let messages = batcher.take_messages().await;
        assert_eq!(messages.len(), 3);
    }

    #[tokio::test]
    async fn test_connection_manager() {
        let manager = ConnectionManager::new(ReconnectConfig::default());

        assert_eq!(manager.state().await, OptConnectionState::Disconnected);
        assert!(!manager.is_connected().await);

        manager.mark_connected().await;
        assert!(manager.is_connected().await);

        let duration = manager.connection_duration().await;
        assert!(duration.is_some());
        assert!(duration.unwrap().as_millis() < 100);
    }

    #[test]
    fn test_reconnect_delay_calculation() {
        let manager = ConnectionManager::new(ReconnectConfig {
            max_retries: 5,
            initial_delay_ms: 1000,
            max_delay_ms: 10000,
            backoff_multiplier: 2,
        });

        // 测试指数退避
        assert_eq!(manager.next_reconnect_delay(0).as_millis(), 1000);
        assert_eq!(manager.next_reconnect_delay(1).as_millis(), 2000);
        assert_eq!(manager.next_reconnect_delay(2).as_millis(), 4000);
        assert_eq!(manager.next_reconnect_delay(3).as_millis(), 8000);

        // 达到最大值
        assert_eq!(manager.next_reconnect_delay(4).as_millis(), 10000);

        // 超过最大重连次数
        assert_eq!(manager.next_reconnect_delay(10).as_millis(), 10000);
    }
}
