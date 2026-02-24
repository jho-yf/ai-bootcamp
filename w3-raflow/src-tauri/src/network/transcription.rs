// src-tauri/src/network/transcription.rs

//! 转录服务

use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::mpsc;
use std::time::Duration;

use crate::core::Result;
use crate::config::ElevenLabsConfig;
use super::{ClientMessage, TranscriptionResult};
use super::websocket::WebSocketClientV2;

/// ElevenLabs Scribe v2 WebSocket URL
const ELITE_LABS_WS_URL: &str = "wss://api.elevenlabs.io/v1/speech-to-text/stream";

/// 转录服务
pub struct TranscriptionService {
    /// WebSocket 客户端
    client: Arc<Mutex<Option<WebSocketClientV2>>>,

    /// API 配置
    api_config: ElevenLabsConfig,

    /// 结果发送器
    result_sender: Arc<Mutex<mpsc::Sender<TranscriptionResult>>>,

    /// 是否已连接
    connected: Arc<Mutex<bool>>,

    /// 部分结果累积
    partial_text: Arc<Mutex<String>>,
}

impl TranscriptionService {
    /// 创建新的转录服务
    pub fn new(api_config: ElevenLabsConfig, result_sender: mpsc::Sender<TranscriptionResult>) -> Self {
        Self {
            client: Arc::new(Mutex::new(None)),
            api_config,
            result_sender: Arc::new(Mutex::new(result_sender)),
            connected: Arc::new(Mutex::new(false)),
            partial_text: Arc::new(Mutex::new(String::new())),
        }
    }

    /// 开始会话
    pub async fn start_session(&self) -> Result<()> {
        tracing::info!("Starting transcription session");

        // 检查 API Key
        if self.api_config.api_key.is_empty() {
            return Err(crate::core::error::NetworkError::AuthFailed(
                "API Key 为空，请先在设置中配置 ElevenLabs API Key".to_string()
            ).into());
        }

        // 使用现有的结果发送器
        let sender = self.result_sender.lock().await.clone();

        // 创建 WebSocket 客户端
        let mut ws_client = WebSocketClientV2::new(
            ELITE_LABS_WS_URL.to_string(),
            sender,
        );

        // 创建初始化消息
        let init_msg = ClientMessage::Init {
            api_key: self.api_config.api_key.clone(),
            language: self.api_config.language.clone(),
            format: "pcm_s16le".to_string(),
            sample_rate: 16000,
        };

        // 连接
        ws_client.connect(init_msg).await?;

        *self.client.lock().await = Some(ws_client);
        *self.connected.lock().await = true;
        *self.partial_text.lock().await = String::new();

        tracing::info!("Transcription session started");

        Ok(())
    }

    /// 发送音频数据
    pub async fn send_audio(&self, frame_bytes: Vec<u8>) -> Result<()> {
        if !*self.connected.lock().await {
            return Err(crate::core::error::NetworkError::ConnectionFailed(
                "未连接到转录服务".to_string()
            ).into());
        }

        let client = self.client.lock().await;
        if let Some(client) = client.as_ref() {
            client.send_audio(frame_bytes).await?;
        }

        Ok(())
    }

    /// 结束会话
    pub async fn end_session(&self) -> Result<()> {
        tracing::info!("Ending transcription session");

        // 发送结束标记
        let client = self.client.lock().await;
        if let Some(ws_client) = client.as_ref() {
            ws_client.send_end().await?;
        }
        drop(client);

        // 等待一小段时间让最终结果返回
        tokio::time::sleep(Duration::from_millis(500)).await;

        // 清理
        *self.client.lock().await = None;
        *self.connected.lock().await = false;
        *self.partial_text.lock().await = String::new();

        tracing::info!("Transcription session ended");

        Ok(())
    }

    /// 获取连接状态
    pub async fn is_connected(&self) -> bool {
        *self.connected.lock().await
    }

    /// 处理转录结果（累积部分结果）
    pub async fn process_result(&self, result: &TranscriptionResult) -> String {
        if result.is_final {
            // 最终结果，清除部分累积
            let text = result.text.clone();
            *self.partial_text.lock().await = String::new();
            text
        } else {
            // 部分结果，累积
            let mut partial = self.partial_text.lock().await;
            *partial = result.text.clone();
            partial.clone()
        }
    }

    /// 获取当前部分文本
    pub async fn get_partial_text(&self) -> String {
        self.partial_text.lock().await.clone()
    }
}

impl Drop for TranscriptionService {
    fn drop(&mut self) {
        // 注意：这里是同步上下文，不能使用 async
        // 实际清理应该通过显式调用 end_session 完成
    }
}

/// 转录会话管理器
pub struct TranscriptionSession {
    /// 服务
    service: Arc<Mutex<TranscriptionService>>,

    /// 结果接收器
    result_receiver: mpsc::Receiver<TranscriptionResult>,

    /// 是否正在运行
    running: Arc<Mutex<bool>>,
}

impl TranscriptionSession {
    /// 创建新的会话
    pub async fn new(api_config: ElevenLabsConfig) -> Result<Self> {
        let (sender, receiver) = mpsc::channel(10);
        let service = TranscriptionService::new(api_config, sender);

        Ok(Self {
            service: Arc::new(Mutex::new(service)),
            result_receiver: receiver,
            running: Arc::new(Mutex::new(false)),
        })
    }

    /// 启动会话
    pub async fn start(&self) -> Result<()> {
        let service = self.service.lock().await;
        service.start_session().await?;
        *self.running.lock().await = true;
        Ok(())
    }

    /// 发送音频
    pub async fn send_audio(&self, data: Vec<u8>) -> Result<()> {
        let service = self.service.lock().await;
        service.send_audio(data).await
    }

    /// 结束会话
    pub async fn end(&self) -> Result<()> {
        *self.running.lock().await = false;
        let service = self.service.lock().await;
        service.end_session().await
    }

    /// 获取结果
    pub async fn try_get_result(&mut self) -> Option<TranscriptionResult> {
        self.result_receiver.try_recv().ok()
    }

    /// 是否正在运行
    pub async fn is_running(&self) -> bool {
        *self.running.lock().await
    }

    /// 获取部分文本
    pub async fn get_partial_text(&self) -> String {
        let service = self.service.lock().await;
        service.get_partial_text().await
    }
}

/// 转录事件
#[derive(Debug, Clone)]
pub enum TranscriptionEvent {
    /// 部分结果
    Partial(String),
    /// 最终结果
    Final(String),
    /// 错误
    Error(String),
    /// 连接状态变化
    Connected(bool),
}
