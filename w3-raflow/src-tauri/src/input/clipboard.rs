// src-tauri/src/input/clipboard.rs

//! 剪贴板操作

use crate::core::{Result, error::InputError};

/// 剪贴板服务（简化版，每次操作都创建新实例）
pub struct ClipboardService;

impl ClipboardService {
    /// 创建新的剪贴板服务
    pub fn new() -> Result<Self> {
        Ok(Self)
    }

    /// 保存原始剪贴板内容（返回内容供调用方保存）
    pub fn save_original(&self) -> Result<Option<String>> {
        if let Ok(mut clipboard) = arboard::Clipboard::new() {
            let content = clipboard.get_text().ok();
            tracing::debug!("Saved original clipboard content");
            Ok(content)
        } else {
            Ok(None)
        }
    }

    /// 设置文本
    pub fn set_text(&self, text: &str) -> Result<()> {
        if let Ok(mut clipboard) = arboard::Clipboard::new() {
            clipboard
                .set_text(text)
                .map_err(|e| InputError::ClipboardError(format!("设置剪贴板失败: {}", e)))?;
            tracing::debug!("Set clipboard text: {}", text);
        }
        Ok(())
    }

    /// 获取文本
    pub fn get_text(&self) -> Result<String> {
        if let Ok(mut clipboard) = arboard::Clipboard::new() {
            let text = clipboard
                .get_text()
                .map_err(|e| InputError::ClipboardError(format!("获取剪贴板失败: {}", e)))?;
            Ok(text)
        } else {
            Ok(String::new())
        }
    }

    /// 恢复原始内容（需要传入之前保存的内容）
    pub fn restore_original(&self, original: &Option<String>) -> Result<()> {
        if let Some(text) = original {
            self.set_text(text)?;
            tracing::debug!("Restored original clipboard content");
        }
        Ok(())
    }
}

impl Default for ClipboardService {
    fn default() -> Self {
        Self::new().expect("Failed to create ClipboardService")
    }
}

/// 剪贴板操作辅助工具
pub struct ClipboardHelper {
    service: ClipboardService,
}

impl ClipboardHelper {
    /// 创建新的辅助工具
    pub fn new() -> Result<Self> {
        Ok(Self {
            service: ClipboardService::new()?,
        })
    }

    /// 获取服务
    pub fn service(&self) -> &ClipboardService {
        &self.service
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clipboard_creation() {
        let service = ClipboardService::new();
        assert!(service.is_ok());
    }

    #[test]
    fn test_helper_creation() {
        let helper = ClipboardHelper::new();
        assert!(helper.is_ok());
    }
}
