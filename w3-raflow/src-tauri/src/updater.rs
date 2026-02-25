// src-tauri/src/updater.rs

//! 自动更新模块
//!
//! 处理应用的自动更新功能，包括检查更新、下载和安装

use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::Mutex;
use serde::{Deserialize, Serialize};

/// 更新状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UpdateStatus {
    /// 检查中
    Checking,
    /// 有可用更新
    Available { version: String, notes: String },
    /// 下载中
    Downloading { progress: u8 },
    /// 准备安装
    ReadyToInstall,
    /// 已是最新版本
    UpToDate,
    /// 错误
    Error { message: String },
}

/// 更新信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    /// 新版本号
    pub version: String,
    /// 更新说明
    pub notes: String,
    /// 发布日期
    pub pub_date: String,
    /// 下载 URL
    pub download_url: String,
    /// 签名 URL
    pub signature_url: Option<String>,
}

/// 更新管理器
pub struct UpdateManager {
    /// 当前状态
    status: Arc<Mutex<UpdateStatus>>,

    /// 应用句柄
    app_handle: AppHandle,

    /// 更新检查 URL
    update_url: String,
}

impl UpdateManager {
    /// 创建新的更新管理器
    pub fn new(app_handle: AppHandle, update_url: String) -> Self {
        Self {
            status: Arc::new(Mutex::new(UpdateStatus::UpToDate)),
            app_handle,
            update_url,
        }
    }

    /// 获取当前状态
    pub async fn status(&self) -> UpdateStatus {
        self.status.lock().await.clone()
    }

    /// 检查更新
    pub async fn check_for_updates(&self) -> Result<UpdateInfo, String> {
        // 更新状态为检查中
        *self.status.lock().await = UpdateStatus::Checking;
        self.emit_status().await;

        // 发起更新检查请求
        let current_version = self.app_handle.config().version.as_deref().unwrap_or("0.0.0");
        let update_url = format!(
            "{}/{}/{}/{}",
            self.update_url.trim_end_matches('/'),
            std::env::consts::OS,
            std::env::consts::ARCH,
            current_version
        );

        // 使用 reqwest 发起请求
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

        let response = client
            .get(&update_url)
            .send()
            .await
            .map_err(|e| format!("检查更新失败: {}", e))?;

        if !response.status().is_success() {
            *self.status.lock().await = UpdateStatus::UpToDate;
            self.emit_status().await;
            return Err("无可用更新".to_string());
        }

        // 解析响应
        let update_info: UpdateInfo = response
            .json()
            .await
            .map_err(|e| format!("解析更新信息失败: {}", e))?;

        // 更新状态
        *self.status.lock().await = UpdateStatus::Available {
            version: update_info.version.clone(),
            notes: update_info.notes.clone(),
        };
        self.emit_status().await;

        Ok(update_info)
    }

    /// 下载并安装更新
    pub async fn download_and_install(&self, _info: &UpdateInfo) -> Result<(), String> {
        // 更新状态为下载中
        *self.status.lock().await = UpdateStatus::Downloading { progress: 0 };
        self.emit_status().await;

        // 注意: Tauri 2.x 使用内置的更新器插件
        // 这里我们通过事件通知前端，让前端调用 Tauri 的 updater API
        Err("使用 Tauri 内置更新器".to_string())
    }

    /// 发送状态更新事件
    async fn emit_status(&self) {
        let status = self.status.lock().await.clone();
        let _ = self.app_handle.emit("update-status", status);
    }

    /// 标记为准备安装
    pub async fn mark_ready(&self) {
        *self.status.lock().await = UpdateStatus::ReadyToInstall;
        self.emit_status().await;
    }

    /// 设置错误状态
    pub async fn set_error(&self, message: String) {
        *self.status.lock().await = UpdateStatus::Error { message };
        self.emit_status().await;
    }

    /// 重置状态
    pub async fn reset(&self) {
        *self.status.lock().await = UpdateStatus::UpToDate;
        self.emit_status().await;
    }
}

/// Tauri 命令: 检查更新
#[tauri::command]
pub async fn check_for_updates(
    app_handle: AppHandle,
    update_url: String,
) -> Result<UpdateInfo, String> {
    let manager = UpdateManager::new(app_handle, update_url);
    manager.check_for_updates().await
}

/// Tauri 命令: 获取当前更新状态
#[tauri::command]
pub async fn get_update_status(app_handle: AppHandle) -> UpdateStatus {
    // 从 app state 获取状态
    if let Some(manager) = app_handle.try_state::<UpdateManager>() {
        manager.status().await
    } else {
        UpdateStatus::UpToDate
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_status_serialization() {
        let status = UpdateStatus::Available {
            version: "1.0.0".to_string(),
            notes: "Test update".to_string(),
        };

        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("Available"));

        let deserialized: UpdateStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(status, deserialized);
    }

    #[test]
    fn test_update_info_serialization() {
        let info = UpdateInfo {
            version: "1.0.0".to_string(),
            notes: "Test notes".to_string(),
            pub_date: "2026-01-01".to_string(),
            download_url: "https://example.com/update".to_string(),
            signature_url: Some("https://example.com/sig".to_string()),
        };

        let json = serde_json::to_string(&info).unwrap();
        let deserialized: UpdateInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(info.version, deserialized.version);
        assert_eq!(info.notes, deserialized.notes);
    }

    #[test]
    fn test_update_status_equality() {
        let status1 = UpdateStatus::Checking;
        let status2 = UpdateStatus::Checking;
        assert_eq!(status1, status2);

        let status3 = UpdateStatus::UpToDate;
        assert_ne!(status1, status3);
    }
}
