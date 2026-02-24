// src-tauri/src/input/service.rs

//! 文本服务

use std::sync::Arc;
use tokio::sync::Mutex;
use std::time::Duration;

use crate::core::{Result, error::InputError};
use crate::config::TextConfig;
use super::{KeyboardSimulator, ClipboardService};

/// 文本插入策略
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextInsertionStrategy {
    /// 自动选择 (优先键盘输入，失败则剪贴板)
    Auto,
    /// 仅使用键盘输入
    KeyboardOnly,
    /// 仅使用剪贴板
    ClipboardOnly,
}

impl From<&str> for TextInsertionStrategy {
    fn from(s: &str) -> Self {
        match s {
            "auto" => Self::Auto,
            "keyboard" => Self::KeyboardOnly,
            "clipboard" => Self::ClipboardOnly,
            _ => Self::Auto,
        }
    }
}

impl From<String> for TextInsertionStrategy {
    fn from(s: String) -> Self {
        Self::from(s.as_str())
    }
}

impl From<TextInsertionStrategy> for String {
    fn from(strategy: TextInsertionStrategy) -> Self {
        match strategy {
            TextInsertionStrategy::Auto => "auto".to_string(),
            TextInsertionStrategy::KeyboardOnly => "keyboard".to_string(),
            TextInsertionStrategy::ClipboardOnly => "clipboard".to_string(),
        }
    }
}

/// 文本插入结果
#[derive(Debug, Clone)]
pub enum TextInsertionResult {
    /// 成功插入
    Success {
        /// 使用的策略
        strategy: TextInsertionStrategy,
    },
    /// 失败，已复制到剪贴板
    FallbackToClipboard {
        /// 失败原因
        reason: String,
    },
    /// 完全失败
    Failed {
        /// 错误信息
        error: String,
    },
}

impl TextInsertionResult {
    /// 是否成功
    pub fn is_success(&self) -> bool {
        matches!(self, TextInsertionResult::Success { .. })
    }

    /// 获取使用的策略
    pub fn strategy(&self) -> Option<TextInsertionStrategy> {
        match self {
            TextInsertionResult::Success { strategy } => Some(*strategy),
            TextInsertionResult::FallbackToClipboard { .. } => Some(TextInsertionStrategy::ClipboardOnly),
            TextInsertionResult::Failed { .. } => None,
        }
    }

    /// 获取错误信息
    pub fn error(&self) -> Option<String> {
        match self {
            TextInsertionResult::FallbackToClipboard { reason } => Some(reason.clone()),
            TextInsertionResult::Failed { error } => Some(error.clone()),
            _ => None,
        }
    }
}

/// 文本服务
pub struct TextService {
    /// 键盘模拟器
    keyboard: Arc<Mutex<KeyboardSimulator>>,

    /// 剪贴板服务
    clipboard: ClipboardService,

    /// 插入策略
    strategy: Arc<Mutex<TextInsertionStrategy>>,

    /// 插入延迟
    insertion_delay: Arc<Mutex<Duration>>,

    /// 重试次数
    max_retries: Arc<Mutex<usize>>,
}

impl TextService {
    /// 创建新的文本服务
    pub fn new(strategy: String) -> Result<Self> {
        let delay_ms = 100; // 默认延迟
        let strategy_enum = TextInsertionStrategy::from(strategy);

        Ok(Self {
            keyboard: Arc::new(Mutex::new(KeyboardSimulator::new(delay_ms)?)),
            clipboard: ClipboardService::new()?,
            strategy: Arc::new(Mutex::new(strategy_enum)),
            insertion_delay: Arc::new(Mutex::new(Duration::from_millis(delay_ms))),
            max_retries: Arc::new(Mutex::new(2)),
        })
    }

    /// 插入文本
    pub async fn insert_text(&self, text: &str) -> Result<TextInsertionResult> {
        let strategy = *self.strategy.lock().await;

        match strategy {
            TextInsertionStrategy::Auto => {
                // 先尝试键盘
                match self.try_keyboard(text).await {
                    Ok(_) => Ok(TextInsertionResult::Success {
                        strategy: TextInsertionStrategy::KeyboardOnly,
                    }),
                    Err(e) => {
                        tracing::warn!("Keyboard input failed: {}, falling back to clipboard", e);
                        // 降级到剪贴板
                        self.try_clipboard(text).await?;
                        Ok(TextInsertionResult::FallbackToClipboard {
                            reason: e.to_string(),
                        })
                    }
                }
            }
            TextInsertionStrategy::KeyboardOnly => {
                self.try_keyboard(text).await?;
                Ok(TextInsertionResult::Success {
                    strategy: TextInsertionStrategy::KeyboardOnly,
                })
            }
            TextInsertionStrategy::ClipboardOnly => {
                self.try_clipboard(text).await?;
                Ok(TextInsertionResult::Success {
                    strategy: TextInsertionStrategy::ClipboardOnly,
                })
            }
        }
    }

    /// 尝试键盘输入
    async fn try_keyboard(&self, text: &str) -> Result<()> {
        let max_retries = *self.max_retries.lock().await;

        for attempt in 0..=max_retries {
            let mut keyboard = self.keyboard.lock().await;

            match keyboard.type_text(text).await {
                Ok(_) => {
                    // 等待一小段时间确保输入完成
                    tokio::time::sleep(Duration::from_millis(50)).await;
                    return Ok(());
                }
                Err(e) => {
                    if attempt < max_retries {
                        tracing::warn!("Keyboard input attempt {} failed: {}, retrying...", attempt, e);
                        tokio::time::sleep(Duration::from_millis(100)).await;
                    } else {
                        return Err(e);
                    }
                }
            }
        }

        Ok(())
    }

    /// 尝试剪贴板方式
    async fn try_clipboard(&self, text: &str) -> Result<()> {
        // 保存原始剪贴板
        let _original = self.clipboard.save_original()?;

        // 设置新内容
        self.clipboard.set_text(text)?;

        // 等待一小段时间
        tokio::time::sleep(Duration::from_millis(50)).await;

        tracing::info!("Text copied to clipboard");

        Ok(())
    }

    /// 使用键盘模拟粘贴
    pub async fn paste_from_clipboard(&self) -> Result<()> {
        let mut keyboard = self.keyboard.lock().await;

        // 模拟 Ctrl+V / Cmd+V
        #[cfg(target_os = "macos")]
        {
            keyboard.key_sequence("c+v")?;
        }

        #[cfg(not(target_os = "macos"))]
        {
            keyboard.key_sequence("c+v")?;
        }

        Ok(())
    }

    /// 插入文本并自动粘贴（剪贴板模式）
    pub async fn insert_text_with_paste(&self, text: &str) -> Result<TextInsertionResult> {
        // 先设置剪贴板
        let _original = self.clipboard.save_original()?;
        self.clipboard.set_text(text)?;

        // 等待剪贴板设置完成
        tokio::time::sleep(Duration::from_millis(100)).await;

        // 模拟粘贴
        self.paste_from_clipboard().await?;

        // 等待粘贴完成
        tokio::time::sleep(Duration::from_millis(200)).await;

        Ok(TextInsertionResult::Success {
            strategy: TextInsertionStrategy::ClipboardOnly,
        })
    }

    /// 设置策略
    pub async fn set_strategy(&self, strategy: TextInsertionStrategy) {
        *self.strategy.lock().await = strategy;
    }

    /// 获取策略
    pub async fn get_strategy(&self) -> TextInsertionStrategy {
        *self.strategy.lock().await
    }

    /// 设置插入延迟
    pub async fn set_insertion_delay(&self, delay_ms: u64) {
        *self.insertion_delay.lock().await = Duration::from_millis(delay_ms);
        let mut keyboard = self.keyboard.lock().await;
        keyboard.set_delay(delay_ms);
    }

    /// 设置最大重试次数
    pub async fn set_max_retries(&self, retries: usize) {
        *self.max_retries.lock().await = retries;
    }

    /// 检查窗口焦点（简化版本）
    pub fn check_focus(&self) -> Result<bool> {
        // TODO: 实现实际的焦点检测
        // 这需要平台特定的 API
        Ok(true)
    }
}

/// 文本服务构建器
pub struct TextServiceBuilder {
    strategy: Option<TextInsertionStrategy>,
    delay_ms: u64,
    max_retries: usize,
}

impl Default for TextServiceBuilder {
    fn default() -> Self {
        Self {
            strategy: None,
            delay_ms: 100,
            max_retries: 2,
        }
    }
}

impl TextServiceBuilder {
    /// 创建新的构建器
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置策略
    pub fn strategy(mut self, strategy: TextInsertionStrategy) -> Self {
        self.strategy = Some(strategy);
        self
    }

    /// 设置延迟
    pub fn delay_ms(mut self, delay_ms: u64) -> Self {
        self.delay_ms = delay_ms;
        self
    }

    /// 设置重试次数
    pub fn max_retries(mut self, retries: usize) -> Self {
        self.max_retries = retries;
        self
    }

    /// 构建服务
    pub fn build(self) -> Result<TextService> {
        let strategy = self.strategy.unwrap_or(TextInsertionStrategy::Auto);
        let mut service = TextService::new(strategy.into())?;

        // 应用设置
        let delay = Arc::try_unwrap(service.insertion_delay.clone())
            .map_err(|_| InputError::Other("无法获取设置锁".to_string()))?;

        let retries = Arc::try_unwrap(service.max_retries.clone())
            .map_err(|_| InputError::Other("无法获取设置锁".to_string()))?;

        // 这里由于 Arc 的复杂性，我们使用其他方式设置
        // 实际应用中可以通过 async 方法设置

        Ok(service)
    }
}
