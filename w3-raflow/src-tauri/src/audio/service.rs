// src-tauri/src/audio/service.rs

//! 音频服务

use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::mpsc;
use std::time::Instant;

use cpal::traits::DeviceTrait;
use crate::core::{Result, error::AudioError};
use super::{AudioFrame, AudioDeviceInfo, capture::AudioCapture};
use super::device::enumerate_audio_devices;

/// 音频服务
pub struct AudioService {
    /// 音频捕获器
    capture: Arc<Mutex<Option<AudioCapture>>>,

    /// 当前设备 ID
    current_device: Arc<Mutex<Option<String>>>,

    /// 音频帧发送器
    sender: mpsc::Sender<AudioFrame>,

    /// 音频帧接收器
    receiver: Arc<Mutex<mpsc::Receiver<AudioFrame>>>,

    /// 录音开始时间
    recording_start: Arc<Mutex<Option<Instant>>>,
}

impl AudioService {
    /// 创建新的音频服务
    pub fn new() -> Self {
        // 增加 channel 容量以避免帧丢失
        let (sender, receiver) = mpsc::channel(1000);

        Self {
            capture: Arc::new(Mutex::new(None)),
            current_device: Arc::new(Mutex::new(None)),
            sender,
            receiver: Arc::new(Mutex::new(receiver)),
            recording_start: Arc::new(Mutex::new(None)),
        }
    }

    /// 初始化音频服务
    pub async fn initialize(&self) -> Result<()> {
        tracing::info!("Initializing audio service");

        // 检查是否有可用的音频设备
        let devices = self.enumerate_devices().await?;
        if devices.is_empty() {
            return Err(AudioError::DeviceUnavailable("未找到可用的音频输入设备".to_string()).into());
        }

        tracing::info!("Found {} audio device(s)", devices.len());

        // 打印可用设备
        for device in &devices {
            tracing::info!("  - {} (default: {})", device.name, device.is_default);
        }

        Ok(())
    }

    /// 枚举音频设备
    pub async fn enumerate_devices(&self) -> Result<Vec<AudioDeviceInfo>> {
        enumerate_audio_devices()
    }

    /// 开始录音
    pub async fn start_recording(&self, device_id: Option<String>) -> Result<()> {
        // 检查是否已经在录音
        {
            let capture = self.capture.lock().await;
            if capture.is_some() {
                let capture_ref = capture.as_ref().unwrap();
                // 注意：这里需要通过异步方式检查，暂时返回错误
                return Err(AudioError::RecordingFailed("已在录音中".to_string()).into());
            }
        }

        // 获取设备 ID（克隆以避免移动）
        let device_id_ref = device_id.as_ref();

        // 获取设备
        let device = if let Some(id) = device_id_ref {
            self.find_device_by_id(id).await?
        } else {
            self.get_default_device().await?
        };

        // 创建音频捕获器
        let mut audio_capture = AudioCapture::new(self.sender.clone());

        // 启动捕获（注意：cpal 的 start 需要在当前上下文中调用）
        audio_capture.start(&device)?;

        // 获取设备名称
        let device_name = device.name().ok();

        // 保存捕获器
        *self.capture.lock().await = Some(audio_capture);
        *self.current_device.lock().await = device_id.or(device_name.clone());
        *self.recording_start.lock().await = Some(Instant::now());

        tracing::info!("Recording started on device: {}", device_name.unwrap_or_else(|| "unknown".to_string()));

        Ok(())
    }

    /// 停止录音
    pub async fn stop_recording(&self) -> Result<()> {
        let mut capture_guard = self.capture.lock().await;

        if capture_guard.is_none() {
            return Ok(());
        }

        // 停止捕获
        if let Some(mut capture) = capture_guard.take() {
            capture.stop()?;
        }

        *self.current_device.lock().await = None;
        *self.recording_start.lock().await = None;

        tracing::info!("Recording stopped");

        Ok(())
    }

    /// 测试麦克风
    pub async fn test_microphone(&self, device_id: &str) -> Result<bool> {
        tracing::info!("=== Testing microphone: {} ===", device_id);

        // 查找设备
        let device = self.find_device_by_id(device_id).await
            .map_err(|e| {
                tracing::error!("Failed to find device {}: {}", device_id, e);
                e
            })?;

        // 获取设备名称用于日志
        let device_name = device.name().unwrap_or_else(|_| "unknown".to_string());
        tracing::info!("Found device: {}", device_name);

        // 尝试获取配置来测试设备是否可用
        let config = device
            .default_input_config()
            .map_err(|e| {
                tracing::error!("Failed to get default input config for {}: {:?}", device_name, e);
                AudioError::DeviceUnavailable(format!("设备不可用: {}", e))
            })?;

        let sample_rate = config.sample_rate();
        let channels = config.channels();
        tracing::info!("Device config: sample_rate={}, channels={}", sample_rate, channels);
        tracing::info!("=== Microphone test PASSED: {} ===", device_name);

        // 设备可用
        Ok(true)
    }

    /// 获取录音状态
    pub async fn is_recording(&self) -> bool {
        self.capture.lock().await.is_some()
    }

    /// 获取录音时长（秒）
    pub async fn recording_duration(&self) -> f64 {
        if let Some(start) = *self.recording_start.lock().await {
            start.elapsed().as_secs_f64()
        } else {
            0.0
        }
    }

    /// 尝试获取音频帧
    pub async fn try_get_frame(&self) -> Option<AudioFrame> {
        self.receiver.lock().await.try_recv().ok()
    }

    /// 等待获取音频帧
    pub async fn get_frame(&self) -> Option<AudioFrame> {
        self.receiver.lock().await.recv().await
    }

    /// 检查是否正在录音（同步版本）
    pub fn is_recording_sync(&self) -> bool {
        // 注意：这是一个阻塞调用，实际使用时需要小心
        // 由于我们不能在这里使用 async，所以使用 try_lock
        if let Ok(guard) = self.capture.try_lock() {
            guard.is_some()
        } else {
            // 如果获取锁失败，假设正在录音（保守估计）
            false
        }
    }

    /// 关闭音频服务
    pub async fn shutdown(&self) -> Result<()> {
        self.stop_recording().await?;
        Ok(())
    }

    /// 根据 ID 查找设备
    async fn find_device_by_id(&self, id: &str) -> Result<cpal::Device> {
        use cpal::traits::{DeviceTrait, HostTrait};

        let host = cpal::default_host();

        let devices = host.devices()
            .map_err(|e| AudioError::DeviceUnavailable(format!("枚举设备失败: {}", e)))?
            .filter(|d| d.default_input_config().is_ok())
            .collect::<Vec<_>>();

        for device in devices {
            if let Ok(name) = device.name() {
                if name == id {
                    return Ok(device);
                }
            }
        }

        Err(AudioError::DeviceUnavailable(format!("未找到设备: {}", id)).into())
    }

    /// 获取默认设备
    async fn get_default_device(&self) -> Result<cpal::Device> {
        use cpal::traits::{DeviceTrait, HostTrait};

        let host = cpal::default_host();
        host.default_input_device()
            .ok_or_else(|| AudioError::DeviceUnavailable("无默认音频输入设备".to_string()).into())
    }
}

impl Default for AudioService {
    fn default() -> Self {
        Self::new()
    }
}

// 为了兼容性，添加非异步的检查方法
impl AudioService {
    /// 检查是否正在录音（非异步版本）
    pub fn is_recording_non_async(&self) -> bool {
        self.is_recording_sync()
    }
}
