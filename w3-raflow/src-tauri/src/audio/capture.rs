// src-tauri/src/audio/capture.rs

//! 音频捕获

use serde::{Deserialize, Serialize};
use cpal::{traits::{StreamTrait, DeviceTrait}, Stream, SampleFormat, Device};
use tokio::sync::mpsc;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::core::{Result, error::AudioError};

/// 音频格式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AudioFormat {
    /// 16位 PCM 小端序
    PcmS16Le,
}

/// 音频配置
#[derive(Debug, Clone, Copy)]
pub struct AudioCaptureConfig {
    /// 采样率 (固定 16000 Hz)
    pub sample_rate: u32,
    /// 声道数 (固定 1 - 单声道)
    pub channels: u16,
    /// 缓冲区大小
    pub buffer_size: u32,
}

impl Default for AudioCaptureConfig {
    fn default() -> Self {
        Self {
            sample_rate: 16000,
            channels: 1,
            buffer_size: 512,
        }
    }
}

/// 音频帧
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioFrame {
    /// 时间戳
    pub timestamp: u64,

    /// PCM数据
    pub data: Vec<i16>,
}

impl AudioFrame {
    /// 创建新的音频帧
    pub fn new(timestamp: u64, data: Vec<i16>) -> Self {
        Self { timestamp, data }
    }

    /// 获取当前时间戳
    pub fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }

    /// 获取帧大小（样本数）
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// 获取持续时间（毫秒），假设采样率为 16kHz
    pub fn duration_ms(&self) -> u64 {
        (self.data.len() as u64 * 1000) / 16000
    }

    /// 转换为字节数组 (小端序)
    pub fn to_bytes(&self) -> Vec<u8> {
        self.data
            .iter()
            .flat_map(|&sample| {
                [(sample & 0xff) as u8, ((sample >> 8) & 0xff) as u8]
            })
            .collect()
    }

    /// 从字节数组创建 (小端序)
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let data: Vec<i16> = bytes
            .chunks_exact(2)
            .map(|chunk| {
                (chunk[0] as i16) | ((chunk[1] as i16) << 8)
            })
            .collect();

        Self {
            timestamp: Self::current_timestamp(),
            data,
        }
    }
}

/// 音频捕获器
pub struct AudioCapture {
    /// cpal 音频流
    stream: Option<Stream>,

    /// 音频发送器
    sender: mpsc::Sender<AudioFrame>,

    /// 是否正在运行
    is_running: Arc<std::sync::atomic::AtomicBool>,
}

impl AudioCapture {
    /// 创建新的音频捕获器
    pub fn new(sender: mpsc::Sender<AudioFrame>) -> Self {
        Self {
            stream: None,
            sender,
            is_running: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    /// 启动音频捕获
    pub fn start(&mut self, device: &Device) -> Result<()> {
        // 检查是否已经在运行
        if self.is_running.load(std::sync::atomic::Ordering::Acquire) {
            return Err(AudioError::RecordingFailed("录音已在进行中".to_string()).into());
        }

        let config = AudioCaptureConfig::default();

        // 获取默认配置
        let default_config = device.default_input_config()
            .map_err(|e| AudioError::DeviceUnavailable(format!("获取设备配置失败: {}", e)))?;

        // 获取格式信息
        let sample_format = default_config.sample_format();
        let sample_rate = default_config.sample_rate();

        tracing::info!("设备音频格式: {:?}, 采样率: {}Hz", sample_format, sample_rate);

        // 使用默认配置
        let stream_config: cpal::StreamConfig = default_config.clone().into();

        let sender = self.sender.clone();
        let is_running = self.is_running.clone();
        let target_sample_rate = config.sample_rate;
        let need_resample = sample_rate != target_sample_rate;

        // 重要：在启动流之前设置 is_running = true
        // 否则回调函数会在检查时发现 is_running = false 并直接返回
        self.is_running.store(true, std::sync::atomic::Ordering::Release);

        // 根据格式创建不同的数据回调
        match sample_format {
            SampleFormat::I16 => {
                let data_callback = move |data: &cpal::Data, _: &cpal::InputCallbackInfo| {
                    if !is_running.load(std::sync::atomic::Ordering::Acquire) {
                        return;
                    }

                    if let Some(i16_samples) = data.as_slice::<i16>() {
                        let final_samples = if need_resample {
                            convert::resample(i16_samples, sample_rate, target_sample_rate)
                        } else {
                            i16_samples.to_vec()
                        };

                        let frame = AudioFrame::new(AudioFrame::current_timestamp(), final_samples);

                        if let Err(e) = sender.try_send(frame) {
                            eprintln!("[Audio Capture I16] Failed to send frame: {:?}", e);
                        }
                    }
                };

                self.build_and_start_stream(device, &stream_config, SampleFormat::I16, data_callback)?;
            }
            SampleFormat::F32 => {
                let data_callback = move |data: &cpal::Data, _info: &cpal::InputCallbackInfo| {
                    if !is_running.load(std::sync::atomic::Ordering::Acquire) {
                        return;
                    }

                    if let Some(f32_samples) = data.as_slice::<f32>() {
                        let samples = convert::f32_to_i16(f32_samples);

                        let final_samples = if need_resample {
                            convert::resample(&samples, sample_rate, target_sample_rate)
                        } else {
                            samples
                        };

                        let frame = AudioFrame::new(AudioFrame::current_timestamp(), final_samples);
                        let _ = sender.try_send(frame);
                    }
                };

                self.build_and_start_stream(device, &stream_config, SampleFormat::F32, data_callback)?;
            }
            SampleFormat::I32 => {
                let data_callback = move |data: &cpal::Data, _: &cpal::InputCallbackInfo| {
                    if !is_running.load(std::sync::atomic::Ordering::Acquire) {
                        return;
                    }

                    if let Some(i32_samples) = data.as_slice::<i32>() {
                        let samples = convert::i32_to_i16(i32_samples);

                        let final_samples = if need_resample {
                            convert::resample(&samples, sample_rate, target_sample_rate)
                        } else {
                            samples
                        };

                        let frame = AudioFrame::new(AudioFrame::current_timestamp(), final_samples);
                        let _ = sender.try_send(frame);
                    }
                };

                self.build_and_start_stream(device, &stream_config, SampleFormat::I32, data_callback)?;
            }
            SampleFormat::U16 => {
                let data_callback = move |data: &cpal::Data, _: &cpal::InputCallbackInfo| {
                    if !is_running.load(std::sync::atomic::Ordering::Acquire) {
                        return;
                    }

                    if let Some(u16_samples) = data.as_slice::<u16>() {
                        let samples = convert::u16_to_i16(u16_samples);

                        let final_samples = if need_resample {
                            convert::resample(&samples, sample_rate, target_sample_rate)
                        } else {
                            samples
                        };

                        let frame = AudioFrame::new(AudioFrame::current_timestamp(), final_samples);
                        let _ = sender.try_send(frame);
                    }
                };

                self.build_and_start_stream(device, &stream_config, SampleFormat::U16, data_callback)?;
            }
            _ => {
                return Err(AudioError::UnsupportedFormat(
                    format!("不支持的音频格式: {:?}", sample_format)
                ).into());
            }
        }

        tracing::info!("音频捕获已启动 (格式: {:?}, 原始采样率: {}Hz)", sample_format, sample_rate);
        Ok(())
    }

    /// 构建并启动音频流
    fn build_and_start_stream<F>(
        &mut self,
        device: &Device,
        stream_config: &cpal::StreamConfig,
        sample_format: SampleFormat,
        data_callback: F,
    ) -> Result<()>
    where
        F: FnMut(&cpal::Data, &cpal::InputCallbackInfo) + Send + 'static,
    {
        let error_callback = move |err| {
            tracing::error!("音频流错误: {}", err);
        };

        let stream = device
            .build_input_stream_raw(
                stream_config,
                sample_format,
                data_callback,
                error_callback,
                None,
            )
            .map_err(|e| AudioError::StreamError(format!("创建音频流失败: {}", e)))?;

        self.stream = Some(stream);

        if let Some(stream) = &self.stream {
            stream
                .play()
                .map_err(|e| AudioError::StreamError(format!("启动音频流失败: {}", e)))?;
        }

        Ok(())
    }

    /// 停止音频捕获
    pub fn stop(&mut self) -> Result<()> {
        if !self.is_running.load(std::sync::atomic::Ordering::Acquire) {
            return Ok(());
        }

        self.is_running.store(false, std::sync::atomic::Ordering::Release);

        // Drop stream 会自动停止
        if let Some(stream) = self.stream.take() {
            drop(stream);
        }

        tracing::info!("音频捕获已停止");
        Ok(())
    }

    /// 检查是否正在运行
    pub fn is_running(&self) -> bool {
        self.is_running.load(std::sync::atomic::Ordering::Acquire)
    }
}

impl Drop for AudioCapture {
    fn drop(&mut self) {
        // 确保停止录音
        let _ = self.stop();
    }
}

/// 音频数据转换工具
pub mod convert {
    use super::*;

    /// 将 f32 样本转换为 i16
    pub fn f32_to_i16(samples: &[f32]) -> Vec<i16> {
        samples
            .iter()
            .map(|&s| {
                if s >= 1.0 {
                    i16::MAX
                } else if s <= -1.0 {
                    i16::MIN
                } else {
                    (s * i16::MAX as f32) as i16
                }
            })
            .collect()
    }

    /// 将 i32 样本转换为 i16
    pub fn i32_to_i16(samples: &[i32]) -> Vec<i16> {
        samples
            .iter()
            .map(|&s| {
                if s >= i32::MAX {
                    i16::MAX
                } else if s <= i32::MIN {
                    i16::MIN
                } else {
                    (s >> 16) as i16
                }
            })
            .collect()
    }

    /// 将 u16 样本转换为 i16
    pub fn u16_to_i16(samples: &[u16]) -> Vec<i16> {
        samples
            .iter()
            .map(|&s| (s as i16).wrapping_sub(i16::MIN as i16))
            .collect()
    }

    /// 重采样音频数据 (简单线性插值)
    pub fn resample(data: &[i16], from_rate: u32, to_rate: u32) -> Vec<i16> {
        if from_rate == to_rate {
            return data.to_vec();
        }

        let ratio = from_rate as f64 / to_rate as f64;
        let output_len = ((data.len() as f64) / ratio).ceil() as usize;

        (0..output_len)
            .map(|i| {
                let src_pos = (i as f64) * ratio;
                let src_idx = src_pos.floor() as usize;
                let frac = src_pos - src_pos.floor();

                if src_idx + 1 < data.len() {
                    let a = data[src_idx] as f64;
                    let b = data[src_idx + 1] as f64;
                    (a + (b - a) * frac) as i16
                } else {
                    data.get(src_idx).copied().unwrap_or(0)
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_frame_creation() {
        let frame = AudioFrame::new(1000, vec![100, 200, 300]);
        assert_eq!(frame.timestamp, 1000);
        assert_eq!(frame.len(), 3);
        assert!(!frame.is_empty());
    }

    #[test]
    fn test_audio_frame_bytes_conversion() {
        let original = AudioFrame::new(1000, vec![1000, -1000, 0, 500, -500]);
        let bytes = original.to_bytes();
        let restored = AudioFrame::from_bytes(&bytes);

        assert_eq!(restored.data, original.data);
    }

    #[test]
    fn test_f32_to_i16_conversion() {
        let input = vec![1.0, 0.5, 0.0, -0.5, -1.0];
        let output = convert::f32_to_i16(&input);

        assert_eq!(output[0], i16::MAX);
        assert_eq!(output[4], i16::MIN);
    }

    #[test]
    fn test_resample() {
        let input = vec![100i16; 160]; // 10ms at 16kHz
        let output = convert::resample(&input, 16000, 8000); // Downsample to 8kHz

        // 输出应该大约是输入的一半
        assert!(output.len() < 160);
        assert!(output.len() > 60);
    }
}
