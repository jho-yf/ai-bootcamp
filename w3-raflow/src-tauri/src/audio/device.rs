// src-tauri/src/audio/device.rs

//! 音频设备管理

use serde::{Deserialize, Serialize};

use crate::core::{Result, error::AudioError};

/// 音频设备信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioDeviceInfo {
    /// 设备名称
    pub name: String,

    /// 设备ID
    pub id: String,

    /// 是否为默认设备
    pub is_default: bool,
}

/// 枚举所有音频输入设备
pub fn enumerate_audio_devices() -> Result<Vec<AudioDeviceInfo>> {
    use cpal::traits::{DeviceTrait, HostTrait};

    let host = cpal::default_host();

    let default_device = host.default_input_device();
    let default_name = default_device
        .as_ref()
        .and_then(|d| d.name().ok());

    let devices = host.devices()
        .map_err(|e| AudioError::DeviceUnavailable(format!("枚举设备失败: {}", e)))?;

    let mut device_list: Vec<AudioDeviceInfo> = Vec::new();

    for device in devices {
        // 只保留有输入配置的设备
        if device.default_input_config().is_err() {
            continue;
        }

        let name = device.name()
            .unwrap_or_else(|_| "未知设备".to_string());

        let is_default = default_name.as_ref() == Some(&name);

        device_list.push(AudioDeviceInfo {
            id: name.clone(),
            name,
            is_default,
        });
    }

    Ok(device_list)
}

/// 获取默认音频输入设备
pub fn get_default_device() -> Result<AudioDeviceInfo> {
    use cpal::traits::{DeviceTrait, HostTrait};

    let host = cpal::default_host();
    let device = host.default_input_device()
        .ok_or_else(|| AudioError::DeviceUnavailable("无默认音频输入设备".to_string()))?;

    let name = device.name()
        .map_err(|e| AudioError::DeviceUnavailable(format!("获取设备名称失败: {}", e)))?;

    Ok(AudioDeviceInfo {
        id: name.clone(),
        name,
        is_default: true,
    })
}
