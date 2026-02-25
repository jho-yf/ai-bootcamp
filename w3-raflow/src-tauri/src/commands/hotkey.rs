// src-tauri/src/commands/hotkey.rs

//! 热键相关命令

use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, Code, Modifiers};

/// 注册热键
#[tauri::command]
pub async fn register_hotkey(
    app: AppHandle,
    modifiers: Vec<String>,
    key: String,
) -> Result<(), String> {
    tracing::info!("=== register_hotkey called ===");
    tracing::info!("  modifiers: {:?}", modifiers);
    tracing::info!("  key: {}", key);

    // 解析修饰键
    let mut mods = Modifiers::empty();
    for m in &modifiers {
        match m.to_lowercase().as_str() {
            "ctrl" | "control" => mods.insert(Modifiers::CONTROL),
            "alt" => mods.insert(Modifiers::ALT),
            "shift" => mods.insert(Modifiers::SHIFT),
            "super" | "cmd" | "meta" => mods.insert(Modifiers::SUPER),
            _ => {
                tracing::warn!("Unknown modifier: {}", m);
            }
        }
    }

    tracing::info!("  Parsed modifiers: CONTROL={}, SHIFT={}, ALT={}, SUPER={}",
        mods.contains(Modifiers::CONTROL),
        mods.contains(Modifiers::SHIFT),
        mods.contains(Modifiers::ALT),
        mods.contains(Modifiers::SUPER)
    );

    // 解析按键
    let code = parse_key_code(&key)?;
    tracing::info!("  Parsed key code: {:?}", code);

    // 创建快捷键
    let shortcut = Shortcut::new(Some(mods), code);
    tracing::info!("  Created shortcut: {:?}", shortcut);

    // 先注销所有快捷键
    let gs = app.global_shortcut();
    gs.unregister_all()
        .map_err(|e| format!("Failed to unregister existing shortcuts: {}", e))?;
    tracing::info!("  Unregistered existing shortcuts");

    // 检查快捷键是否已注册
    let is_registered = gs.is_registered(shortcut);
    tracing::info!("  Is shortcut already registered: {}", is_registered);

    // 注册全局快捷键
    let app_clone = app.clone();
    gs.on_shortcut(shortcut, move |_app, _shortcut, _event| {
        tracing::info!("==================================================");
        tracing::info!("=== HOTKEY TRIGGERED! Calling toggle_recording directly ===");
        tracing::info!("==================================================");

        // 直接调用 toggle_recording 命令，而不是通过事件传递
        let app_handle = app_clone.clone();
        // 使用 tauri::async_runtime::spawn 而不是 tokio::spawn
        tauri::async_runtime::spawn(async move {
            tracing::info!("Async task started for toggle_recording");

            // 获取需要的状态
            let storage = app_handle.try_state::<std::sync::Arc<crate::config::ConfigStorage>>();
            let raflow_app = app_handle.try_state::<std::sync::Arc<crate::core::RaFlowApp>>();

            match (storage, raflow_app) {
                (Some(storage), Some(raflow_app)) => {
                    // 获取内部的 Arc
                    let storage_arc = storage.inner().clone();
                    let raflow_app_arc = raflow_app.inner().clone();

                    tracing::info!("Got state, calling toggle_recording_impl");

                    // 调用 toggle_recording
                    match crate::commands::toggle_recording_impl(app_handle.clone(), raflow_app_arc, storage_arc).await {
                        Ok(_) => tracing::info!("toggle_recording completed successfully"),
                        Err(e) => {
                            tracing::error!("toggle_recording failed: {}", e);
                            // 发送错误事件
                            let _ = app_handle.emit("show-error", e);
                        }
                    }
                }
                _ => {
                    tracing::error!("Required state not available");
                    let _ = app_handle.emit("show-error", "应用状态未正确初始化");
                }
            }
        });
    })
    .map_err(|e| {
        tracing::error!("Failed to register shortcut callback: {}", e);
        format!("Failed to register shortcut: {}", e)
    })?;

    // 验证注册成功
    let is_registered = gs.is_registered(shortcut);
    tracing::info!("=== HOTKEY REGISTERED SUCCESSFULLY ===");
    tracing::info!("  Shortcut: {:?}", shortcut);
    tracing::info!("  Is registered: {}", is_registered);

    if !is_registered {
        return Err("Shortcut registration reported as not registered".to_string());
    }

    Ok(())
}

/// 注销热键
#[tauri::command]
pub async fn unregister_hotkey(app: AppHandle) -> Result<(), String> {
    app.global_shortcut().unregister_all()
        .map_err(|e| format!("Failed to unregister shortcuts: {}", e))?;

    tracing::info!("=== ALL HOTKEYS UNREGISTERED ===");

    Ok(())
}

/// 测试热键
#[tauri::command]
pub async fn test_hotkey(app: AppHandle) -> Result<bool, String> {
    // 检查插件是否可用
    let gs = app.global_shortcut();
    let result = gs.unregister_all().is_ok();

    tracing::info!("Hotkey plugin test result: {}", result);

    Ok(result)
}

/// 解析按键代码
fn parse_key_code(key: &str) -> Result<Code, String> {
    match key.to_lowercase().as_str() {
        // 字母
        "a" => Ok(Code::KeyA),
        "b" => Ok(Code::KeyB),
        "c" => Ok(Code::KeyC),
        "d" => Ok(Code::KeyD),
        "e" => Ok(Code::KeyE),
        "f" => Ok(Code::KeyF),
        "g" => Ok(Code::KeyG),
        "h" => Ok(Code::KeyH),
        "i" => Ok(Code::KeyI),
        "j" => Ok(Code::KeyJ),
        "k" => Ok(Code::KeyK),
        "l" => Ok(Code::KeyL),
        "m" => Ok(Code::KeyM),
        "n" => Ok(Code::KeyN),
        "o" => Ok(Code::KeyO),
        "p" => Ok(Code::KeyP),
        "q" => Ok(Code::KeyQ),
        "r" => Ok(Code::KeyR),
        "s" => Ok(Code::KeyS),
        "t" => Ok(Code::KeyT),
        "u" => Ok(Code::KeyU),
        "v" => Ok(Code::KeyV),
        "w" => Ok(Code::KeyW),
        "x" => Ok(Code::KeyX),
        "y" => Ok(Code::KeyY),
        "z" => Ok(Code::KeyZ),
        // 数字
        "0" => Ok(Code::Digit0),
        "1" => Ok(Code::Digit1),
        "2" => Ok(Code::Digit2),
        "3" => Ok(Code::Digit3),
        "4" => Ok(Code::Digit4),
        "5" => Ok(Code::Digit5),
        "6" => Ok(Code::Digit6),
        "7" => Ok(Code::Digit7),
        "8" => Ok(Code::Digit8),
        "9" => Ok(Code::Digit9),
        // 特殊键
        "\\" => Ok(Code::Backslash),
        "/" => Ok(Code::Slash),
        "space" => Ok(Code::Space),
        "enter" | "return" => Ok(Code::Enter),
        "tab" => Ok(Code::Tab),
        "escape" | "esc" => Ok(Code::Escape),
        "backspace" => Ok(Code::Backspace),
        // 箭头
        "up" => Ok(Code::ArrowUp),
        "down" => Ok(Code::ArrowDown),
        "left" => Ok(Code::ArrowLeft),
        "right" => Ok(Code::ArrowRight),
        // 功能键
        "f1" => Ok(Code::F1),
        "f2" => Ok(Code::F2),
        "f3" => Ok(Code::F3),
        "f4" => Ok(Code::F4),
        "f5" => Ok(Code::F5),
        "f6" => Ok(Code::F6),
        "f7" => Ok(Code::F7),
        "f8" => Ok(Code::F8),
        "f9" => Ok(Code::F9),
        "f10" => Ok(Code::F10),
        "f11" => Ok(Code::F11),
        "f12" => Ok(Code::F12),
        _ => Err(format!("Unknown key: {}", key)),
    }
}
