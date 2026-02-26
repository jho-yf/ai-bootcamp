// src-tauri/src/network/websocket.rs

//! WebSocket 客户端

use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::mpsc;
use tokio_tungstenite::{
    connect_async_tls_with_config,
    tungstenite::{Message, protocol::WebSocketConfig, client::IntoClientRequest, http::header::HeaderName},
    MaybeTlsStream,
    WebSocketStream,
};
use futures_util::{StreamExt, SinkExt};
use base64::{Engine as _, engine::general_purpose};

use crate::core::{Result, error::NetworkError, AppError};
use super::{ClientMessage, ServerMessage, TranscriptionResult};

/// WebSocket 客户端（支持并发写入）
pub struct WebSocketClient {
    /// WebSocket URL
    url: String,

    /// 消息发送队列
    message_queue: Arc<Mutex<mpsc::Sender<Message>>>,

    /// 是否已连接
    connected: Arc<Mutex<bool>>,

    /// 结果发送器
    result_sender: mpsc::Sender<TranscriptionResult>,
}

impl WebSocketClient {
    /// 创建新的 WebSocket 客户端
    pub fn new(url: String, result_sender: mpsc::Sender<TranscriptionResult>) -> Self {
        let (sender, _receiver) = mpsc::channel(100);

        Self {
            url,
            message_queue: Arc::new(Mutex::new(sender)),
            connected: Arc::new(Mutex::new(false)),
            result_sender,
        }
    }

    /// 连接到服务器
    /// config: 配置参数（model_id, language_code, audio_format 等）
    pub async fn connect(
        &mut self,
        api_key: String,
        model_id: String,
        language_code: String,
        audio_format: String,
        sample_rate: u32,
    ) -> Result<()> {
        tracing::info!("Connecting to WebSocket (V2): {}", self.url);

        // 打印 API Key 信息（部分隐藏）
        let key_preview = if api_key.len() > 8 {
            format!("{}...{}", &api_key[..4], &api_key[api_key.len()-4..])
        } else if api_key.is_empty() {
            "(empty)".to_string()
        } else {
            "(too short)".to_string()
        };
        tracing::info!("API Key preview: {}, length: {}", key_preview, api_key.len());

        // ElevenLabs Speech to Text Realtime API:
        // 配置通过 URL 查询参数传递，使用 xi-api-key header 认证
        // 参考：https://elevenlabs.io/docs/api-reference/speech-to-text/v-1-speech-to-text-realtime

        // 构建 URL 查询参数
        let mut params = vec![
            format!("model_id={}", model_id),
            format!("audio_format={}", audio_format),
            "commit_strategy=manual".to_string(),
        ];

        // 只有当语言代码不为空时才添加
        if !language_code.is_empty() {
            params.push(format!("language_code={}", language_code));
        }

        let url_with_params = format!("{}?{}", self.url, params.join("&"));

        tracing::info!("[连接中] 语言: {}", if language_code.is_empty() { "自动检测" } else { &language_code });

        // 创建带有自定义 headers 的 WebSocket 请求
        let mut request = url_with_params.clone().into_client_request()
            .map_err(|e| NetworkError::ConnectionFailed(format!("创建请求失败: {}", e)))?;

        // 添加 xi-api-key header
        request.headers_mut().insert(
            HeaderName::from_static("xi-api-key"),
            api_key.parse()
                .map_err(|e| NetworkError::AuthFailed(format!("无效的 API Key: {}", e)))?
        );

        // 使用 TLS 连接（wss://）
        let (ws_stream, _) = connect_async_tls_with_config(
            request,
            None::<WebSocketConfig>,
            false,
            None,
        )
        .await
        .map_err(|e| NetworkError::ConnectionFailed(format!("连接失败: {}", e)))?;

        let (mut write, mut read) = ws_stream.split();

        tracing::info!("[已连接] 等待会话开始...");

        // 标记为已连接
        *self.connected.lock().await = true;

        // 创建新的消息队列用于写入
        let (msg_sender, mut msg_receiver) = mpsc::channel(100);
        *self.message_queue.lock().await = msg_sender;

        // 启动写入任务
        let connected_write = self.connected.clone();
        tokio::spawn(async move {
            while let Some(msg) = msg_receiver.recv().await {
                if !*connected_write.lock().await {
                    break;
                }

                if let Err(e) = write.send(msg).await {
                    tracing::error!("Failed to send message: {}", e);
                    *connected_write.lock().await = false;
                    break;
                }
            }
        });

        // 启动接收循环
        let sender = self.result_sender.clone();
        let connected_read = self.connected.clone();

        tokio::spawn(async move {
            while let Some(msg_result) = read.next().await {
                match msg_result {
                    Ok(Message::Text(text)) => {
                        tracing::debug!("Received text message: {}", text);

                        if let Ok(server_msg) = serde_json::from_str::<ServerMessage>(&text) {
                            match &server_msg {
                                ServerMessage::SessionStarted { session_id, config } => {
                                    tracing::info!(
                                        "[会话开始] Session: {}, Model: {}, 语言: {:?}",
                                        session_id,
                                        config.model_id,
                                        config.language_code
                                    );
                                }
                                ServerMessage::PartialTranscript { text } => {
                                    tracing::info!("[转录中] {}", text);

                                    let result = TranscriptionResult::new(text.clone(), false);
                                    let _ = sender.send(result).await;
                                }
                                ServerMessage::CommittedTranscript { text } => {
                                    tracing::info!("[转录完成] {}", text);

                                    let result = TranscriptionResult::new(text.clone(), true);
                                    let _ = sender.send(result).await;
                                }
                                ServerMessage::CommittedTranscriptWithTimestamps { text, language_code, words } => {
                                    tracing::info!("[转录完成] {} (语言: {:?}, 词数: {})", text, language_code, words.len());

                                    let mut result = TranscriptionResult::new(text.clone(), true);
                                    result.language = language_code.clone();
                                    let _ = sender.send(result).await;
                                }
                                ServerMessage::Error { code, message } => {
                                    tracing::error!("[API错误] {}: {}", code, message);
                                }
                                ServerMessage::AuthError { message } => {
                                    tracing::error!("[认证错误] {}", message);
                                    *connected_read.lock().await = false;
                                    break;
                                }
                                ServerMessage::QuotaExceededError { message } => {
                                    tracing::error!("[配额超限] {}", message);
                                }
                                ServerMessage::ThrottledError { message } => {
                                    tracing::warn!("[限流] {}", message);
                                }
                                ServerMessage::RateLimitedError { message } => {
                                    tracing::warn!("[频率限制] {}", message);
                                }
                                ServerMessage::CommitThrottled { error } => {
                                    tracing::debug!("[提交忽略] {}", error);
                                }
                            }
                        } else {
                            tracing::warn!("Failed to parse server message: {}", text);
                        }
                    }
                    Ok(Message::Close(_)) => {
                        tracing::info!("[连接关闭]");
                        *connected_read.lock().await = false;
                        break;
                    }
                    Err(e) => {
                        tracing::error!("[连接错误] {}", e);
                        *connected_read.lock().await = false;
                        break;
                    }
                    _ => {}
                }
            }
        });

        Ok(())
    }

    /// 发送音频数据
    pub async fn send_audio(&self, data: Vec<u8>, sample_rate: u32) -> Result<()> {
        if !*self.connected.lock().await {
            return Err(NetworkError::ConnectionFailed("未连接到服务器".to_string()).into());
        }

        let base64_data = general_purpose::STANDARD.encode(&data);
        let msg = ClientMessage::InputAudioChunk {
            audio_base_64: base64_data,
            commit: None,  // 使用手动提交模式，不自动提交
            sample_rate,
            previous_text: None,
        };

        let json = serde_json::to_string(&msg)
            .map_err(|e| NetworkError::SendFailed(format!("序列化音频消息失败: {}", e)))?;

        self.message_queue
            .lock()
            .await
            .send(Message::Text(json.into()))
            .await
            .map_err(|_| NetworkError::SendFailed("发送队列已满".to_string()))
            .map_err(|e| AppError::Network(e))?;

        Ok(())
    }

    /// 发送提交标记（手动提交模式）
    pub async fn send_commit(&self) -> Result<()> {
        if !*self.connected.lock().await {
            return Ok(());
        }

        // 发送一个空的音频块并设置 commit=true 来提交当前的转录
        let msg = ClientMessage::InputAudioChunk {
            audio_base_64: String::new(),
            commit: Some(true),
            sample_rate: 16000,  // 默认采样率
            previous_text: None,
        };

        let json = serde_json::to_string(&msg)
            .map_err(|e| NetworkError::SendFailed(format!("序列化提交消息失败: {}", e)))?;

        self.message_queue
            .lock()
            .await
            .send(Message::Text(json.into()))
            .await
            .map_err(|_| NetworkError::SendFailed("发送队列已满".to_string()))
            .map_err(|e| AppError::Network(e))?;

        Ok(())
    }

    /// 检查是否已连接
    pub async fn is_connected(&self) -> bool {
        *self.connected.lock().await
    }

    /// 断开连接
    pub async fn disconnect(&self) -> Result<()> {
        *self.connected.lock().await = false;
        Ok(())
    }
}
