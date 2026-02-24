// src-tauri/src/core/state.rs

//! 应用状态定义

use serde::{Deserialize, Serialize};

/// 录音状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum RecordingState {
    /// 空闲
    #[default]
    Idle,
    /// 录音中
    Recording,
    /// 处理中
    Processing,
}

/// WebSocket 连接状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ConnectionState {
    /// 未连接
    #[default]
    Disconnected,
    /// 连接中
    Connecting,
    /// 已连接
    Connected,
    /// 认证中
    Authenticating,
    /// 就绪
    Ready,
    /// 流式传输中
    Streaming,
    /// 错误
    Error,
}

/// 应用状态
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppState {
    /// 当前录音状态
    pub recording_state: RecordingState,

    /// WebSocket 连接状态
    pub connection_state: ConnectionState,

    /// 当前音频设备 ID
    pub current_device: Option<String>,

    /// 最后的识别结果
    pub last_result: Option<String>,

    /// 当前录音时长（秒）
    pub recording_duration: u64,

    /// 部分识别文本（实时预览）
    pub partial_text: Option<String>,
}

impl AppState {
    /// 创建新的默认状态
    pub fn new() -> Self {
        Self::default()
    }

    /// 重置为初始状态
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// 更新录音状态
    pub fn set_recording_state(&mut self, state: RecordingState) {
        self.recording_state = state;
    }

    /// 更新连接状态
    pub fn set_connection_state(&mut self, state: ConnectionState) {
        self.connection_state = state;
    }

    /// 更新部分文本
    pub fn update_partial_text(&mut self, text: String) {
        self.partial_text = Some(text);
    }

    /// 设置最终结果
    pub fn set_final_result(&mut self, text: String) {
        self.last_result = Some(text);
        self.partial_text = None;
    }
}
