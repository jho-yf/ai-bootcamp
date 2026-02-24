// src-tauri/src/network/websocket.rs

//! WebSocket 客户端

use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::mpsc;
use tokio_tungstenite::{
    connect_async,
    tungstenite::Message,
    MaybeTlsStream,
    WebSocketStream,
};
use futures_util::{StreamExt, SinkExt};
use base64::{Engine as _, engine::general_purpose};

use crate::core::{Result, error::NetworkError, AppError};
use super::{ClientMessage, ServerMessage, TranscriptionResult};

/// WebSocket 客户端
pub struct WebSocketClient {
    /// WebSocket URL
    url: String,

    /// WebSocket 写入端
    write: Arc<Mutex<Option<WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>>>>,

    /// 是否已连接
    connected: Arc<Mutex<bool>>,

    /// 结果发送器
    result_sender: mpsc::Sender<TranscriptionResult>,
}

impl WebSocketClient {
    /// 创建新的 WebSocket 客户端
    pub fn new(url: String, result_sender: mpsc::Sender<TranscriptionResult>) -> Self {
        Self {
            url,
            write: Arc::new(Mutex::new(None)),
            connected: Arc::new(Mutex::new(false)),
            result_sender,
        }
    }

    /// 连接到服务器
    pub async fn connect(&mut self, init_msg: ClientMessage) -> Result<()> {
        tracing::info!("Connecting to WebSocket: {}", self.url);

        // 连接到服务器
        let (ws_stream, _) = connect_async(&self.url)
            .await
            .map_err(|e| NetworkError::ConnectionFailed(format!("连接失败: {}", e)))?;

        tracing::info!("WebSocket connected, sending initialization");

        let (mut write, mut read) = ws_stream.split();

        // 发送初始化消息
        let init_json = serde_json::to_string(&init_msg)
            .map_err(|e| NetworkError::SendFailed(format!("序列化初始化消息失败: {}", e)))?;

        write
            .send(Message::Text(init_json.into()))
            .await
            .map_err(|e| NetworkError::SendFailed(format!("发送初始化消息失败: {}", e)))?;

        tracing::info!("Initialization message sent");

        // 标记为已连接（不再保存 ws_stream）
        *self.connected.lock().await = true;

        // 启动接收循环
        let sender = self.result_sender.clone();
        let connected = self.connected.clone();

        tokio::spawn(async move {
            tracing::info!("Starting WebSocket receive loop");

            while let Some(msg_result) = read.next().await {
                match msg_result {
                    Ok(Message::Text(text)) => {
                        tracing::debug!("Received text message: {}", text);

                        // 解析服务器消息
                        if let Ok(server_msg) = serde_json::from_str::<ServerMessage>(&text) {
                            match &server_msg {
                                ServerMessage::Result { text, is_final, confidence, language } => {
                                    let mut result = TranscriptionResult::new(text.clone(), *is_final, *confidence);
                                    result.language = language.clone();
                                    let _ = sender.send(result).await;
                                }
                                ServerMessage::Status { state } => {
                                    tracing::info!("Server status: {}", state);
                                }
                                ServerMessage::Error { code, message } => {
                                    tracing::error!("Server error: {} - {}", code, message);
                                }
                            }
                        }
                    }
                    Ok(Message::Close(close_frame)) => {
                        tracing::info!("WebSocket closed: {:?}", close_frame);
                        *connected.lock().await = false;
                        break;
                    }
                    Ok(Message::Ping(data)) => {
                        // 响应 ping
                        // tungstenite 会自动处理 pong
                        tracing::trace!("Received ping: {:?}", data);
                    }
                    Ok(Message::Pong(data)) => {
                        tracing::trace!("Received pong: {:?}", data);
                    }
                    Err(e) => {
                        tracing::error!("WebSocket error: {}", e);
                        *connected.lock().await = false;
                        break;
                    }
                    _ => {}
                }
            }

            tracing::info!("WebSocket receive loop ended");
        });

        Ok(())
    }

    /// 发送音频数据
    pub async fn send_audio(&self, data: Vec<u8>) -> Result<()> {
        if !*self.connected.lock().await {
            return Err(NetworkError::ConnectionFailed("未连接到服务器".to_string()).into());
        }

        // 编码为 Base64
        let base64_data = general_purpose::STANDARD.encode(&data);

        let msg = ClientMessage::Audio { data: base64_data };
        let json = serde_json::to_string(&msg)
            .map_err(|e| NetworkError::SendFailed(format!("序列化音频消息失败: {}", e)))?;

        // 注意：这里需要重构以支持并发写入
        // 当前简化版本会在每次发送时重新获取流
        // TODO: 改进以支持高效并发写入
        tracing::trace!("Sending audio frame, size: {} bytes", data.len());

        Ok(())
    }

    /// 发送结束标记
    pub async fn send_end(&self) -> Result<()> {
        if !*self.connected.lock().await {
            return Ok(());
        }

        let msg = ClientMessage::End;
        let json = serde_json::to_string(&msg)
            .map_err(|e| NetworkError::SendFailed(format!("序列化结束消息失败: {}", e)))?;

        tracing::info!("Sending end message");

        // TODO: 实际发送消息
        // 这需要重构 write 端的处理方式

        Ok(())
    }

    /// 断开连接
    pub async fn disconnect(&mut self) -> Result<()> {
        tracing::info!("Disconnecting WebSocket");

        *self.connected.lock().await = false;

        if let Some(mut ws_stream) = self.write.lock().await.take() {
            let _ = ws_stream.close(None).await;
        }

        Ok(())
    }

    /// 检查是否已连接
    pub async fn is_connected(&self) -> bool {
        *self.connected.lock().await
    }
}

/// 改进的 WebSocket 客户端，支持并发写入
pub struct WebSocketClientV2 {
    /// WebSocket URL
    url: String,

    /// 消息发送队列
    message_queue: Arc<Mutex<mpsc::Sender<Message>>>,

    /// 是否已连接
    connected: Arc<Mutex<bool>>,

    /// 结果发送器
    result_sender: mpsc::Sender<TranscriptionResult>,
}

impl WebSocketClientV2 {
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
    pub async fn connect(&mut self, init_msg: ClientMessage) -> Result<()> {
        tracing::info!("Connecting to WebSocket: {}", self.url);

        // 连接到服务器
        let (ws_stream, _) = connect_async(&self.url)
            .await
            .map_err(|e| NetworkError::ConnectionFailed(format!("连接失败: {}", e)))?;

        let (mut write, mut read) = ws_stream.split();

        // 发送初始化消息
        let init_json = serde_json::to_string(&init_msg)
            .map_err(|e| NetworkError::SendFailed(format!("序列化初始化消息失败: {}", e)))?;

        write
            .send(Message::Text(init_json.into()))
            .await
            .map_err(|e| NetworkError::SendFailed(format!("发送初始化消息失败: {}", e)))?;

        tracing::info!("Initialization message sent");

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
                                ServerMessage::Result { text, is_final, confidence, language } => {
                                    let mut result = TranscriptionResult::new(text.clone(), *is_final, *confidence);
                                    result.language = language.clone();
                                    let _ = sender.send(result).await;
                                }
                                ServerMessage::Status { state } => {
                                    tracing::info!("Server status: {}", state);
                                }
                                ServerMessage::Error { code, message } => {
                                    tracing::error!("Server error: {} - {}", code, message);
                                }
                            }
                        }
                    }
                    Ok(Message::Close(_)) => {
                        tracing::info!("WebSocket closed");
                        *connected_read.lock().await = false;
                        break;
                    }
                    Err(e) => {
                        tracing::error!("WebSocket error: {}", e);
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
    pub async fn send_audio(&self, data: Vec<u8>) -> Result<()> {
        if !*self.connected.lock().await {
            return Err(NetworkError::ConnectionFailed("未连接到服务器".to_string()).into());
        }

        let base64_data = general_purpose::STANDARD.encode(&data);
        let msg = ClientMessage::Audio { data: base64_data };
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

    /// 发送结束标记
    pub async fn send_end(&self) -> Result<()> {
        if !*self.connected.lock().await {
            return Ok(());
        }

        let msg = ClientMessage::End;
        let json = serde_json::to_string(&msg)
            .map_err(|e| NetworkError::SendFailed(format!("序列化结束消息失败: {}", e)))?;

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
