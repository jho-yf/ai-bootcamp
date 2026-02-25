// src-tauri/src/commands/hotkey.rs

//! 热键相关命令

use tauri::{AppHandle, Emitter};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, Code, Modifiers};

/// 注册热键
#[tauri::command]
pub async fn register_hotkey(
    app: AppHandle,
    modifiers: Vec<String>,
    key: String,
) -> Result<(), String> {
    // 解析修饰键
    let mut mods = Modifiers::empty();
    for m in &modifiers {
        match m.to_lowercase().as_str() {
            "ctrl" | "control" => mods.insert(Modifiers::CONTROL),
            "alt" => mods.insert(Modifiers::ALT),
            "shift" => mods.insert(Modifiers::SHIFT),
            "super" | "cmd" | "meta" => mods.insert(Modifiers::SUPER),
            _ => {}
        }
    }

    // 解析按键
    let code = parse_key_code(&key)?;

    // 创建快捷键 - API 接受 Option<Modifiers>
    let shortcut = Shortcut::new(Some(mods), code);

    // 先注销所有快捷键
    app.global_shortcut().unregister_all()
        .map_err(|e| format!("Failed to unregister existing shortcuts: {}", e))?;

    // 注册全局快捷键
    let app_clone = app.clone();
    app.global_shortcut().on_shortcut(shortcut, move |_app, _shortcut, _event| {
        tracing::info!("Hotkey triggered, emitting event to frontend");
        // 触发事件通知前端
        match app_clone.emit("hotkey-triggered", ()) {
            Ok(_) => tracing::info!("Event emitted successfully"),
            Err(e) => tracing::error!("Failed to emit event: {}", e),
        }
    })
    .map_err(|e| format!("Failed to register shortcut: {}", e))?;

    tracing::info!("Hotkey registered: {:?}", shortcut);

    Ok(())
}

/// 注销热键
#[tauri::command]
pub async fn unregister_hotkey(app: AppHandle) -> Result<(), String> {
    app.global_shortcut().unregister_all()
        .map_err(|e| format!("Failed to unregister shortcuts: {}", e))?;

    tracing::info!("All hotkeys unregistered");

    Ok(())
}

/// 测试热键
#[tauri::command]
pub async fn test_hotkey(app: AppHandle) -> Result<bool, String> {
    // 简单检查插件是否可用
    Ok(app.global_shortcut().unregister_all().is_ok())
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
