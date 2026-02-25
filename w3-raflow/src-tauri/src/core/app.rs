// src-tauri/src/core/app.rs

//! 应用主结构

use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::mpsc;
use std::time::Duration;

use crate::core::{Result, AppState, RecordingState, ConnectionState};
use crate::config::AppConfig;
use crate::audio::AudioService;
use crate::network::{TranscriptionService, TranscriptionResult};
use crate::input::{TextService, TextInsertionResult};
use crate::hotkey::HotkeyHandler;

/// RaFlow 应用主结构
pub struct RaFlowApp {
    /// 音频服务
    audio_service: Arc<AudioService>,

    /// 转录服务
    transcription_service: Arc<Mutex<Option<TranscriptionService>>>,

    /// 文本服务
    text_service: Arc<TextService>,

    /// 配置
    config: Arc<Mutex<AppConfig>>,

    /// 应用状态
    state: Arc<Mutex<AppState>>,

    /// 热键处理器
    hotkey_handler: Arc<HotkeyHandler>,

    /// 转录结果接收器
    result_receiver: Arc<Mutex<mpsc::Receiver<TranscriptionResult>>>,
}

impl RaFlowApp {
    /// 创建新的应用实例
    pub async fn new(config: AppConfig) -> Result<Self> {
        // 创建音频服务
        let audio_service = Arc::new(AudioService::new());

        // 创建转录服务通道
        let (result_tx, result_rx) = mpsc::channel(10);
        let transcription_svc = TranscriptionService::new(config.elevenlabs.clone(), result_tx);

        // 创建文本服务
        let text_service = Arc::new(TextService::new(config.text.strategy.clone())?);

        // 创建热键处理器
        let hotkey_handler = Arc::new(HotkeyHandler::new());

        Ok(Self {
            audio_service,
            transcription_service: Arc::new(Mutex::new(Some(transcription_svc))),
            text_service,
            config: Arc::new(Mutex::new(config)),
            state: Arc::new(Mutex::new(AppState::new())),
            hotkey_handler,
            result_receiver: Arc::new(Mutex::new(result_rx)),
        })
    }

    /// 启动应用
    pub async fn start(&self) -> Result<()> {
        tracing::info!("Starting RaFlow application");

        // 初始化各服务
        self.audio_service.initialize().await?;

        // 启动状态更新任务
        let state_clone = self.state.clone();
        let audio_clone = self.audio_service.clone();
        let transcription_clone = self.transcription_service.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(100));
            loop {
                interval.tick().await;

                let mut state = state_clone.lock().await;

                // 更新录音状态
                if audio_clone.is_recording().await {
                    state.set_recording_state(RecordingState::Recording);
                } else {
                    state.set_recording_state(RecordingState::Idle);
                }

                // 更新连接状态
                let trans = transcription_clone.lock().await;
                if let Some(ts) = trans.as_ref() {
                    if ts.is_connected().await {
                        state.set_connection_state(ConnectionState::Streaming);
                    } else {
                        state.set_connection_state(ConnectionState::Disconnected);
                    }
                }
            }
        });

        Ok(())
    }

    /// 关闭应用
    pub async fn shutdown(&self) -> Result<()> {
        tracing::info!("Shutting down RaFlow application");

        // 停止各服务
        self.audio_service.shutdown().await?;

        Ok(())
    }

    /// 开始录音转写流程
    pub async fn start_recording_transcription(&self) -> Result<()> {
        tracing::info!("==================================================");
        tracing::info!("=== STARTING RECORDING TRANSCRIPTION FLOW ===");
        tracing::info!("==================================================");

        // 检查状态
        {
            let state = self.state.lock().await;
            if state.recording_state == RecordingState::Recording {
                tracing::warn!("Already recording, ignoring request");
                return Err(crate::core::error::AudioError::RecordingFailed("已在录音中".to_string()).into());
            }
        }

        // 更新状态
        tracing::info!("Step 1: Updating state to Recording");
        self.state.lock().await.set_recording_state(RecordingState::Recording);
        self.hotkey_handler.set_recording_state(true).await;

        // 开始音频捕获
        tracing::info!("Step 2: Starting audio capture");
        self.audio_service.start_recording(None).await?;
        tracing::info!("Audio capture started");

        // 启动转录会话
        tracing::info!("Step 3: Starting transcription session");
        let mut transcription_guard = self.transcription_service.lock().await;
        if let Some(transcription_service) = transcription_guard.as_mut() {
            transcription_service.start_session().await?;
            tracing::info!("Transcription session started");
        } else {
            tracing::warn!("Transcription service is None");
        }
        drop(transcription_guard);

        // 启动音频转发任务
        tracing::info!("Step 4: Starting audio forwarding task");
        let audio = self.audio_service.clone();
        let transcription = self.transcription_service.clone();

        tokio::spawn(async move {
            tracing::info!("Audio forwarding task started");
            let mut frame_count = 0u64;
            while audio.is_recording().await {
                if let Some(frame) = audio.try_get_frame().await {
                    frame_count += 1;
                    let bytes = frame.to_bytes();

                    let trans_guard = transcription.lock().await;
                    if let Some(ts) = trans_guard.as_ref() {
                        match ts.send_audio(bytes).await {
                            Ok(_) => {
                                if frame_count % 100 == 0 {
                                    tracing::debug!("Sent {} audio frames", frame_count);
                                }
                            }
                            Err(e) => {
                                tracing::error!("Failed to send audio: {}", e);
                            }
                        }
                    }
                }

                tokio::time::sleep(Duration::from_millis(10)).await;
            }
            tracing::info!("Audio forwarding task stopped (sent {} frames total)", frame_count);
        });

        tracing::info!("==================================================");
        tracing::info!("=== RECORDING TRANSCRIPTION STARTED SUCCESSFULLY ===");
        tracing::info!("==================================================");

        Ok(())
    }

    /// 停止录音转写流程
    pub async fn stop_recording_transcription(&self) -> Result<Option<String>> {
        tracing::info!("==================================================");
        tracing::info!("=== STOPPING RECORDING TRANSCRIPTION FLOW ===");
        tracing::info!("==================================================");

        // 停止音频捕获
        tracing::info!("Step 1: Stopping audio capture");
        self.audio_service.stop_recording().await?;
        tracing::info!("Audio capture stopped");

        // 结束转录会话
        tracing::info!("Step 2: Ending transcription session");
        let mut transcription_guard = self.transcription_service.lock().await;
        if let Some(transcription_service) = transcription_guard.as_mut() {
            transcription_service.end_session().await?;
            tracing::info!("Transcription session ended");
        }
        drop(transcription_guard);

        // 获取最终结果
        tracing::info!("Step 3: Waiting for final transcription result");
        let mut final_text = None;
        let mut result_receiver = self.result_receiver.lock().await;
        // 尝试获取最终结果
        for i in 0..10 {
            if let Some(result) = result_receiver.try_recv().ok() {
                if result.is_final {
                    final_text = Some(result.text.clone());
                    tracing::info!("Got final result ({} chars): {}", result.text.len(), result.text);
                    break;
                }
            }
            if i < 9 {
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }
        drop(result_receiver);

        // 更新状态
        tracing::info!("Step 4: Updating state to Idle");
        self.state.lock().await.set_recording_state(RecordingState::Idle);
        self.hotkey_handler.set_recording_state(false).await;

        tracing::info!("==================================================");
        if let Some(ref text) = final_text {
            tracing::info!("=== RECORDING STOPPED WITH RESULT: {} ===", text);
        } else {
            tracing::info!("=== RECORDING STOPPED (no result) ===");
        }
        tracing::info!("==================================================");

        Ok(final_text)
    }

    /// 获取状态
    pub fn state(&self) -> &Arc<Mutex<AppState>> {
        &self.state
    }

    /// 获取配置
    pub fn config(&self) -> &Arc<Mutex<AppConfig>> {
        &self.config
    }

    /// 获取音频服务
    pub fn audio_service(&self) -> &Arc<AudioService> {
        &self.audio_service
    }

    /// 获取文本服务
    pub fn text_service(&self) -> &Arc<TextService> {
        &self.text_service
    }

    /// 获取热键处理器
    pub fn hotkey_handler(&self) -> &Arc<HotkeyHandler> {
        &self.hotkey_handler
    }

    /// 插入文本
    pub async fn insert_text(&self, text: &str) -> Result<TextInsertionResult> {
        self.text_service.insert_text(text).await
    }
}
