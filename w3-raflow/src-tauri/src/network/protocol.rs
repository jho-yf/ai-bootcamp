// src-tauri/src/network/protocol.rs

//! WebSocket 消息协议

use serde::{Deserialize, Serialize};
use crate::config::ElevenLabsConfig;

/// 客户端消息类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    /// 初始化配置
    #[serde(rename = "init")]
    Init {
        /// API 密钥
        api_key: String,
        /// 语言代码
        language: String,
        /// 音频格式
        format: String,
        /// 采样率
        sample_rate: u32,
    },

    /// 音频数据
    #[serde(rename = "audio")]
    Audio {
        /// Base64 编码的音频数据
        data: String,
    },

    /// 结束标记
    #[serde(rename = "end")]
    End,
}

/// 服务器消息类型
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    /// 识别结果
    #[serde(rename = "result")]
    Result {
        /// 识别文本
        text: String,
        /// 是否为最终结果
        is_final: bool,
        /// 置信度
        confidence: f32,
        /// 语言检测
        language: Option<String>,
    },

    /// 错误消息
    #[serde(rename = "error")]
    Error {
        /// 错误代码
        code: String,
        /// 错误信息
        message: String,
    },

    /// 状态消息
    #[serde(rename = "status")]
    Status {
        /// 状态
        state: String,
    },
}

/// 转录结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionResult {
    /// 识别的文本
    pub text: String,

    /// 是否为最终结果
    pub is_final: bool,

    /// 置信度 (0.0 - 1.0)
    pub confidence: f32,

    /// 语言检测
    pub language: Option<String>,

    /// 时间戳
    pub timestamp: u64,
}

impl TranscriptionResult {
    /// 创建新的转录结果
    pub fn new(text: String, is_final: bool, confidence: f32) -> Self {
        Self {
            text,
            is_final,
            confidence,
            language: None,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    /// 设置语言
    pub fn with_language(mut self, language: String) -> Self {
        self.language = Some(language);
        self
    }
}

impl From<&ServerMessage> for Option<TranscriptionResult> {
    fn from(msg: &ServerMessage) -> Self {
        match msg {
            ServerMessage::Result { text, is_final, confidence, language } => {
                let mut result = TranscriptionResult::new(text.clone(), *is_final, *confidence);
                result.language = language.clone();
                Some(result)
            }
            _ => None,
        }
    }
}
