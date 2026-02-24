// src-tauri/src/tray/mod.rs

//! 托盘模块

use tauri::{AppHandle, Manager, Emitter};
use tauri::tray::{TrayIconBuilder, TrayIconEvent, MouseButton, MouseButtonState};

use crate::core::Result;

/// 托盘图标状态
#[derive(Debug, Clone, Copy)]
pub enum TrayIconState {
    /// 空闲状态
    Idle,
    /// 录音中
    Recording,
    /// 处理中
    Processing,
    /// 错误状态
    Error,
}

/// 托盘菜单动作
#[derive(Debug, Clone)]
pub enum TrayMenuAction {
    /// 打开设置
    OpenSettings,
    /// 显示状态
    ShowStatus,
    /// 显示关于
    ShowAbout,
    /// 退出应用
    Quit,
}

/// 设置托盘
pub fn setup_tray(app: &AppHandle) -> Result<()> {
    let _tray = TrayIconBuilder::with_id("main")
        .tooltip("RaFlow 语音输入")
        .on_tray_icon_event({
            let app = app.clone();
            move |_tray, event| {
                if let TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                } = event {
                    // 左键点击托盘图标 - 显示/隐藏窗口
                    tracing::info!("Tray icon clicked");
                    let _ = app.emit("tray-clicked", ());
                }
            }
        })
        .build(app)?;

    tracing::info!("Tray icon setup completed");

    Ok(())
}

/// 更新托盘状态
pub fn update_tray_state(app: &AppHandle, state: TrayIconState) -> Result<()> {
    // 更新提示文本
    if let Some(tray) = app.tray_by_id("main") {
        let tooltip = match state {
            TrayIconState::Idle => Some("RaFlow 语音输入 - 就绪"),
            TrayIconState::Recording => Some("RaFlow 语音输入 - 录音中"),
            TrayIconState::Processing => Some("RaFlow 语音输入 - 处理中"),
            TrayIconState::Error => Some("RaFlow 语音输入 - 错误"),
        };
        let _ = tray.set_tooltip(tooltip);
    }

    Ok(())
}

/// 发送前端事件更新托盘状态
pub fn emit_tray_state_update(app: &AppHandle, state: TrayIconState) -> Result<()> {
    let state_str = match state {
        TrayIconState::Idle => "idle",
        TrayIconState::Recording => "recording",
        TrayIconState::Processing => "processing",
        TrayIconState::Error => "error",
    };

    app.emit("tray-state-changed", state_str)?;
    Ok(())
}
