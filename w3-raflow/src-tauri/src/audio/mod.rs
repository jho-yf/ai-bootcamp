// src-tauri/src/audio/mod.rs

//! 音频模块

pub mod capture;
pub mod device;
pub mod service;
pub mod pool;

pub use capture::{AudioCapture, AudioCaptureConfig, AudioFrame, AudioFormat};
pub use device::{AudioDeviceInfo, enumerate_audio_devices};
pub use service::AudioService;
pub use pool::{AudioBufferPool, PoolStats};
