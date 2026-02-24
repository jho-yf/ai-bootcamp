// src-tauri/src/audio/mod.rs

//! 音频模块

pub mod capture;
pub mod device;
pub mod service;

pub use capture::{AudioFrame, AudioFormat};
pub use device::AudioDeviceInfo;
pub use service::AudioService;
