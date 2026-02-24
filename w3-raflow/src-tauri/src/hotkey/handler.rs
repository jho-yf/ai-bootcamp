// src-tauri/src/hotkey/handler.rs

//! 热键处理器

use std::sync::Arc;
use tokio::sync::Mutex;

use crate::core::Result;

/// 热键处理器
pub struct HotkeyHandler {
    /// 是否正在录音
    is_recording: Arc<Mutex<bool>>,

    /// 触发计数
    trigger_count: Arc<Mutex<u64>>,
}

impl HotkeyHandler {
    /// 创建新的处理器
    pub fn new() -> Self {
        Self {
            is_recording: Arc::new(Mutex::new(false)),
            trigger_count: Arc::new(Mutex::new(0)),
        }
    }

    /// 处理热键触发
    pub async fn handle_trigger(&self) -> Result<HotkeyAction> {
        let mut count = self.trigger_count.lock().await;
        *count += 1;
        let current_count = *count;
        drop(count);

        let is_recording = *self.is_recording.lock().await;

        let action = if is_recording {
            HotkeyAction::StopRecording
        } else {
            HotkeyAction::StartRecording
        };

        tracing::info!("Hotkey triggered (#{}): {:?}", current_count, action);

        Ok(action)
    }

    /// 设置录音状态
    pub async fn set_recording_state(&self, recording: bool) {
        *self.is_recording.lock().await = recording;
    }

    /// 获取录音状态
    pub async fn is_recording(&self) -> bool {
        *self.is_recording.lock().await
    }

    /// 获取触发计数
    pub async fn trigger_count(&self) -> u64 {
        *self.trigger_count.lock().await
    }
}

impl Default for HotkeyHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// 热键动作
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotkeyAction {
    /// 开始录音
    StartRecording,
    /// 停止录音
    StopRecording,
    /// 取消操作
    Cancel,
}
