// src-tauri/src/commands/recording.rs

//! 录音相关命令（包含完整验证）

use tauri::{AppHandle, Emitter, State};
use std::sync::Arc;

use crate::core::RaFlowApp;
use crate::config::ConfigStorage;

/// 热键触发的录音切换（内部实现）
///
/// 可以直接从热键回调中调用，不需要通过 Tauri 命令系统
pub async fn toggle_recording_impl(
    app: AppHandle,
    raflow_app: Arc<RaFlowApp>,
    storage: Arc<ConfigStorage>,
) -> Result<(), String> {
    tracing::info!("=== toggle_recording_impl called ===");

    // 1. 加载配置
    let config = storage.load()
        .map_err(|e| format!("加载配置失败: {}", e))?;

    // 2. 检查当前录音状态
    let is_recording = raflow_app.state().lock().await.recording_state == crate::core::RecordingState::Recording;

    if is_recording {
        // 停止录音和转录
        tracing::info!("Stopping recording transcription via hotkey");
        let result = raflow_app.stop_recording_transcription().await
            .map_err(|e| {
                let error_msg = format!("停止录音失败: {}", e);
                tracing::error!("Failed to stop recording: {}", e);
                error_msg
            })?;

        // 发送停止事件
        app.emit("recording-stopped", result)
            .map_err(|e| format!("Failed to emit event: {}", e))?;

        Ok(())
    } else {
        // 开始录音前进行验证

        // 3. 验证 ElevenLabs API Key
        if config.elevenlabs.api_key.is_empty() || config.elevenlabs.api_key.len() < 10 {
            let error_msg = "请先在设置中配置 ElevenLabs API Key。\n\n获取方式：\n1. 访问 https://elevenlabs.io\n2. 注册账号并获取 API Key\n3. 在设置中粘贴 API Key";
            tracing::error!("API key not configured");
            app.emit("show-error", error_msg)
                .map_err(|e| format!("Failed to emit error event: {}", e))?;
            return Err(error_msg.to_string());
        }

        // 4. 验证音频设备
        let devices = raflow_app.audio_service().enumerate_devices().await
            .map_err(|e| format!("获取音频设备失败: {}", e))?;

        if devices.is_empty() {
            let error_msg = "未检测到可用的音频输入设备。\n\n请检查：\n1. 麦克风是否已连接\n2. 系统麦克风权限是否已开启\n3. 麦克风是否被其他应用占用";
            tracing::error!("No audio devices available");
            app.emit("show-error", error_msg)
                .map_err(|e| format!("Failed to emit error event: {}", e))?;
            return Err(error_msg.to_string());
        }

        tracing::info!("Pre-checks passed, starting recording transcription");

        // 5. 启动完整的录音转录流程
        raflow_app.start_recording_transcription().await
            .map_err(|e| {
                let error_msg = format!("启动录音失败: {}", e);
                tracing::error!("Failed to start recording transcription: {}", e);
                app.emit("show-error", error_msg.clone()).ok();
                error_msg
            })?;

        // 发送录音开始事件
        app.emit("recording-started", ())
            .map_err(|e| format!("Failed to emit event: {}", e))?;

        tracing::info!("Recording transcription started successfully");
        Ok(())
    }
}

/// 热键触发的录音切换
///
/// 这是热键触发时应该调用的命令，它会进行完整的前置检查
#[tauri::command]
pub async fn toggle_recording(
    app: AppHandle,
    raflow_app: State<'_, Arc<RaFlowApp>>,
    storage: State<'_, Arc<ConfigStorage>>,
) -> Result<(), String> {
    toggle_recording_impl(app, raflow_app.inner().clone(), storage.inner().clone()).await
}

/// 检查录音功能是否可用
///
/// 返回详细的可用性信息和错误提示
#[tauri::command]
pub async fn check_recording_availability(
    storage: State<'_, Arc<ConfigStorage>>,
    raflow_app: State<'_, Arc<RaFlowApp>>,
) -> Result<RecordingAvailability, String> {
    let mut availability = RecordingAvailability {
        available: true,
        issues: Vec::new(),
    };

    // 检查 API Key
    let config = storage.load()
        .map_err(|e| format!("加载配置失败: {}", e))?;

    if config.elevenlabs.api_key.is_empty() || config.elevenlabs.api_key.len() < 10 {
        availability.available = false;
        availability.issues.push("未配置 ElevenLabs API Key".to_string());
    }

    // 检查音频设备
    let devices = raflow_app.audio_service().enumerate_devices().await
        .map_err(|e| format!("获取音频设备失败: {}", e))?;

    if devices.is_empty() {
        availability.available = false;
        availability.issues.push("未检测到可用的音频输入设备".to_string());
    }

    Ok(availability)
}

/// 录音可用性信息
#[derive(serde::Serialize, Clone, Debug)]
pub struct RecordingAvailability {
    /// 是否可用
    pub available: bool,

    /// 问题列表（如果不可用）
    pub issues: Vec<String>,
}
