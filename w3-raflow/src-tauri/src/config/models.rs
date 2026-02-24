// src-tauri/src/config/models.rs

//! 配置数据模型

use serde::{Deserialize, Serialize};

/// 应用配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// 通用设置
    pub general: GeneralConfig,

    /// 音频设置
    pub audio: AudioConfig,

    /// ElevenLabs 设置
    pub elevenlabs: ElevenLabsConfig,

    /// 快捷键设置
    pub hotkey: HotkeyConfig,

    /// 文本设置
    pub text: TextConfig,

    /// UI 设置
    pub ui: UIConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            audio: AudioConfig::default(),
            elevenlabs: ElevenLabsConfig::default(),
            hotkey: HotkeyConfig::default(),
            text: TextConfig::default(),
            ui: UIConfig::default(),
        }
    }
}

/// 通用配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    /// 应用语言
    pub language: String,

    /// 启动时自动运行
    pub autostart: bool,

    /// 最小化到托盘
    pub minimize_to_tray: bool,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            language: "zh-CN".to_string(),
            autostart: false,
            minimize_to_tray: true,
        }
    }
}

/// 音频配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    /// 音频输入设备 ID (空则使用默认)
    pub device_id: String,

    /// 采样率
    pub sample_rate: u32,

    /// 启用回声消除
    pub echo_cancellation: bool,

    /// 启用噪声抑制
    pub noise_suppression: bool,

    /// 启用自动增益
    pub auto_gain: bool,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            device_id: String::new(),
            sample_rate: 16000,
            echo_cancellation: true,
            noise_suppression: true,
            auto_gain: true,
        }
    }
}

/// ElevenLabs 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElevenLabsConfig {
    /// API 密钥
    pub api_key: String,

    /// 默认语言 (auto = 自动检测)
    pub language: String,

    /// 连接超时 (秒)
    pub timeout: u64,
}

impl Default for ElevenLabsConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            language: "auto".to_string(),
            timeout: 30,
        }
    }
}

/// 热键配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyConfig {
    /// 修饰键
    pub modifiers: Vec<KeyModifier>,

    /// 主键
    pub key: KeyCode,

    /// 是否启用
    pub enabled: bool,
}

impl Default for HotkeyConfig {
    fn default() -> Self {
        Self {
            modifiers: vec![KeyModifier::Ctrl, KeyModifier::Shift],
            key: KeyCode::Backslash,
            enabled: true,
        }
    }
}

/// 修饰键
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum KeyModifier {
    Ctrl,
    Alt,
    Shift,
    Super, // Win/Cmd
}

/// 按键码
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum KeyCode {
    Backslash,
    Space,
    Char(char),
}

/// 文本配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextConfig {
    /// 文本插入策略
    pub strategy: String,

    /// 插入延迟 (毫秒)
    pub insertion_delay: u64,
}

impl Default for TextConfig {
    fn default() -> Self {
        Self {
            strategy: "auto".to_string(),
            insertion_delay: 100,
        }
    }
}

/// UI 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIConfig {
    /// 显示通知
    pub show_notifications: bool,

    /// 状态指示器透明度 (0.0 - 1.0)
    pub indicator_opacity: f32,

    /// 是否显示实时预览
    pub show_live_preview: bool,
}

impl Default for UIConfig {
    fn default() -> Self {
        Self {
            show_notifications: true,
            indicator_opacity: 0.9,
            show_live_preview: true,
        }
    }
}
