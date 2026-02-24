// src-tauri/src/commands/config.rs

//! 配置相关 Tauri 命令

use tauri::State;
use std::sync::Arc;

use crate::config::{ConfigStorage, AppConfig};

/// 获取配置
#[tauri::command]
pub async fn get_config(
    storage: State<'_, Arc<ConfigStorage>>,
) -> std::result::Result<AppConfig, String> {
    storage.load().map_err(|e| e.to_string())
}

/// 保存配置
#[tauri::command]
pub async fn save_config(
    config: AppConfig,
    storage: State<'_, Arc<ConfigStorage>>,
) -> std::result::Result<(), String> {
    storage.save(&config).map_err(|e| e.to_string())
}

/// 重置配置
#[tauri::command]
pub async fn reset_config(
    storage: State<'_, Arc<ConfigStorage>>,
) -> std::result::Result<AppConfig, String> {
    storage.reset().map_err(|e| e.to_string())
}

/// 获取配置 schema（用于前端验证）
#[tauri::command]
pub async fn get_config_schema() -> std::result::Result<serde_json::Value, String> {
    // 返回配置的 JSON Schema
    Ok(serde_json::json!({
        "general": {
            "language": { "type": "string", "default": "zh-CN" },
            "autostart": { "type": "boolean", "default": false },
            "minimize_to_tray": { "type": "boolean", "default": true }
        },
        "audio": {
            "device_id": { "type": "string", "default": "" },
            "sample_rate": { "type": "integer", "default": 16000 },
            "echo_cancellation": { "type": "boolean", "default": true },
            "noise_suppression": { "type": "boolean", "default": true },
            "auto_gain": { "type": "boolean", "default": true }
        },
        "elevenlabs": {
            "api_key": { "type": "string", "default": "" },
            "language": { "type": "string", "default": "auto" },
            "timeout": { "type": "integer", "default": 30 }
        },
        "hotkey": {
            "modifiers": { "type": "array", "items": { "type": "string" }, "default": ["Ctrl", "Shift"] },
            "key": { "type": "string", "default": "Backslash" },
            "enabled": { "type": "boolean", "default": true }
        },
        "text": {
            "strategy": { "type": "string", "default": "auto" },
            "insertion_delay": { "type": "integer", "default": 100 }
        },
        "ui": {
            "show_notifications": { "type": "boolean", "default": true },
            "indicator_opacity": { "type": "number", "default": 0.9 },
            "show_live_preview": { "type": "boolean", "default": true }
        }
    }))
}
