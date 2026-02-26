// src-tauri/src/network/protocol.rs

//! WebSocket 消息协议 - 基于 ElevenLabs Speech-to-Text Realtime API
//! 参考: https://elevenlabs.io/docs/api-reference/speech-to-text/v-1-speech-to-text-realtime

use serde::{Deserialize, Serialize};

/// 客户端消息类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "message_type")]
pub enum ClientMessage {
    /// 输入音频块
    #[serde(rename = "input_audio_chunk")]
    InputAudioChunk {
        /// Base64 编码的音频数据
        audio_base_64: String,
        /// 是否提交此音频块（手动提交模式）
        #[serde(skip_serializing_if = "Option::is_none")]
        commit: Option<bool>,
        /// 采样率
        sample_rate: u32,
        /// 上下文文本（可选）
        #[serde(skip_serializing_if = "Option::is_none")]
        previous_text: Option<String>,
    },
}

/// 服务器消息类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "message_type")]
pub enum ServerMessage {
    /// 会话已开始
    #[serde(rename = "session_started")]
    SessionStarted {
        /// 会话 ID
        session_id: String,
        /// 配置信息
        config: SessionConfig,
    },

    /// 部分转录结果
    #[serde(rename = "partial_transcript")]
    PartialTranscript {
        /// 识别的文本
        text: String,
    },

    /// 已提交的转录结果
    #[serde(rename = "committed_transcript")]
    CommittedTranscript {
        /// 识别的文本
        text: String,
    },

    /// 带时间戳的已提交转录结果
    #[serde(rename = "committed_transcript_with_timestamps")]
    CommittedTranscriptWithTimestamps {
        /// 识别的文本
        text: String,
        /// 语言代码
        language_code: Option<String>,
        /// 单词级别的时间戳
        words: Vec<WordTimestamp>,
    },

    /// 错误消息
    #[serde(rename = "error")]
    Error {
        /// 错误代码
        code: String,
        /// 错误信息
        message: String,
    },

    /// 认证错误
    #[serde(rename = "auth_error")]
    AuthError {
        /// 错误信息
        message: String,
    },

    /// 配额超限错误
    #[serde(rename = "quota_exceeded_error")]
    QuotaExceededError {
        /// 错误信息
        message: String,
    },

    /// 限流错误
    #[serde(rename = "throttled_error")]
    ThrottledError {
        /// 错误信息
        message: String,
    },

    /// 速率限制错误
    #[serde(rename = "rate_limited_error")]
    RateLimitedError {
        /// 错误信息
        message: String,
    },

    /// 提交被限流（音频太短）
    #[serde(rename = "commit_throttled")]
    CommitThrottled {
        /// 错误信息
        error: String,
    },
}

/// 会话配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    /// 采样率
    pub sample_rate: u32,
    /// 音频格式
    pub audio_format: String,
    /// 语言代码（可能为 null）
    pub language_code: Option<String>,
    /// 时间戳粒度
    pub timestamps_granularity: Option<String>,
    /// VAD 提交策略
    pub vad_commit_strategy: Option<bool>,
    /// VAD 静音阈值（秒）
    pub vad_silence_threshold_secs: Option<f32>,
    /// VAD 阈值
    pub vad_threshold: Option<f32>,
    /// 最小语音持续时间（毫秒）
    pub min_speech_duration_ms: Option<u32>,
    /// 最小静音持续时间（毫秒）
    pub min_silence_duration_ms: Option<u32>,
    /// 最大重新计算 token 数
    pub max_tokens_to_recompute: Option<u32>,
    /// 模型 ID
    pub model_id: String,
    /// 是否禁用日志
    pub disable_logging: Option<bool>,
    /// 是否包含时间戳
    pub include_timestamps: Option<bool>,
    /// 是否包含语言检测
    pub include_language_detection: Option<bool>,
}

/// 单词时间戳
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordTimestamp {
    /// 单词文本
    pub text: String,
    /// 开始时间（秒）
    pub start: f32,
    /// 结束时间（秒）
    pub end: f32,
    /// 类型（word 或 spacing）
    #[serde(rename = "type")]
    pub word_type: String,
    /// 对数概率（可选）
    pub logprob: Option<f32>,
    /// 字符列表（可选）
    pub characters: Option<Vec<String>>,
}

/// 转录结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionResult {
    /// 识别的文本
    pub text: String,

    /// 是否为最终结果
    pub is_final: bool,

    /// 置信度 (0.0 - 1.0)，可选
    pub confidence: Option<f32>,

    /// 语言检测
    pub language: Option<String>,

    /// 时间戳
    pub timestamp: u64,
}

impl TranscriptionResult {
    /// 创建新的转录结果
    pub fn new(text: String, is_final: bool) -> Self {
        Self {
            text,
            is_final,
            confidence: None,
            language: None,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    /// 设置置信度
    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = Some(confidence);
        self
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
            ServerMessage::PartialTranscript { text } => {
                Some(TranscriptionResult::new(text.clone(), false))
            }
            ServerMessage::CommittedTranscript { text } => {
                Some(TranscriptionResult::new(text.clone(), true))
            }
            ServerMessage::CommittedTranscriptWithTimestamps { text, language_code, .. } => {
                let mut result = TranscriptionResult::new(text.clone(), true);
                result.language = language_code.clone();
                Some(result)
            }
            _ => None,
        }
    }
}
