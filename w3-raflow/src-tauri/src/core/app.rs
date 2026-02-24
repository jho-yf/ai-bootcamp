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
        tracing::info!("Starting recording transcription flow");

        // 检查状态
        {
            let state = self.state.lock().await;
            if state.recording_state == RecordingState::Recording {
                return Err(crate::core::error::AudioError::RecordingFailed("已在录音中".to_string()).into());
            }
        }

        // 更新状态
        self.state.lock().await.set_recording_state(RecordingState::Recording);
        self.hotkey_handler.set_recording_state(true).await;

        // 开始音频捕获
        self.audio_service.start_recording(None).await?;

        // 启动转录会话
        let mut transcription_guard = self.transcription_service.lock().await;
        if let Some(transcription_service) = transcription_guard.as_mut() {
            transcription_service.start_session().await?;
        }
        drop(transcription_guard);

        // 启动音频转发任务
        let audio = self.audio_service.clone();
        let transcription = self.transcription_service.clone();

        tokio::spawn(async move {
            while audio.is_recording().await {
                if let Some(frame) = audio.try_get_frame().await {
                    let bytes = frame.to_bytes();

                    let trans_guard = transcription.lock().await;
                    if let Some(ts) = trans_guard.as_ref() {
                        let _ = ts.send_audio(bytes).await;
                    }
                }

                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        });

        Ok(())
    }

    /// 停止录音转写流程
    pub async fn stop_recording_transcription(&self) -> Result<Option<String>> {
        tracing::info!("Stopping recording transcription flow");

        // 停止音频捕获
        self.audio_service.stop_recording().await?;

        // 结束转录会话
        let mut transcription_guard = self.transcription_service.lock().await;
        if let Some(transcription_service) = transcription_guard.as_mut() {
            transcription_service.end_session().await?;
        }
        drop(transcription_guard);

        // 获取最终结果
        let mut final_text = None;
        let mut result_receiver = self.result_receiver.lock().await;
        // 尝试获取最终结果
        for _ in 0..10 {
            if let Some(result) = result_receiver.try_recv().ok() {
                if result.is_final {
                    final_text = Some(result.text);
                    break;
                }
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        drop(result_receiver);

        // 更新状态
        self.state.lock().await.set_recording_state(RecordingState::Idle);
        self.hotkey_handler.set_recording_state(false).await;

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
