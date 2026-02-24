// src-tauri/src/commands/audio.rs

//! 音频相关 Tauri 命令

use tauri::State;
use std::sync::Arc;

use crate::audio::{AudioService, AudioDeviceInfo};

/// 获取音频设备列表
#[tauri::command]
pub async fn get_audio_devices(
    service: State<'_, Arc<AudioService>>,
) -> std::result::Result<Vec<AudioDeviceInfo>, String> {
    service.enumerate_devices().await.map_err(|e| e.to_string())
}

/// 开始录音
#[tauri::command]
pub async fn start_recording(
    device_id: Option<String>,
    service: State<'_, Arc<AudioService>>,
) -> std::result::Result<(), String> {
    service.start_recording(device_id).await.map_err(|e| e.to_string())
}

/// 停止录音
#[tauri::command]
pub async fn stop_recording(
    service: State<'_, Arc<AudioService>>,
) -> std::result::Result<(), String> {
    service.stop_recording().await.map_err(|e| e.to_string())
}

/// 测试麦克风
#[tauri::command]
pub async fn test_microphone(
    device_id: String,
    service: State<'_, Arc<AudioService>>,
) -> std::result::Result<bool, String> {
    service.test_microphone(&device_id).await.map_err(|e| e.to_string())
}

/// 获取当前录音状态
#[tauri::command]
pub async fn get_recording_state(
    service: State<'_, Arc<AudioService>>,
) -> std::result::Result<bool, String> {
    Ok(service.is_recording().await)
}
