// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;
use raflow_lib::{RaFlowApp, ConfigStorage, AppConfig};
use tauri_plugin_global_shortcut::GlobalShortcutExt;

#[tokio::main]
async fn main() {
    // 初始化日志
    tracing_subscriber::fmt::init();

    // 加载配置
    let storage = ConfigStorage::new().expect("Failed to create config storage");
    let config = storage.load().unwrap_or_else(|_| AppConfig::default());

    // 创建应用实例
    let app: RaFlowApp = RaFlowApp::new(config.clone())
        .await
        .expect("Failed to create app");

    // 启动应用
    app.start().await.expect("Failed to start app");

    // 运行 Tauri
    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(move |app| {
            tracing::info!("=== Tauri setup started ===");

            // 检查全局快捷键插件是否可用
            let shortcut_app = app.handle().clone();
            let test_shortcut = tauri_plugin_global_shortcut::Shortcut::new(
                Some(tauri_plugin_global_shortcut::Modifiers::CONTROL | tauri_plugin_global_shortcut::Modifiers::SHIFT),
                tauri_plugin_global_shortcut::Code::KeyO,
            );
            let test_result = shortcut_app.global_shortcut().is_registered(test_shortcut);
            tracing::info!("Is Ctrl+Shift+O registered: {}", test_result);

            // 设置系统托盘
            raflow_lib::tray::setup_tray(app.handle())?;

            // 注册默认热键
            let hotkey_config = config.hotkey.clone();
            if hotkey_config.enabled {
                let modifiers: Vec<String> = hotkey_config.modifiers
                    .iter()
                    .map(|m| match m {
                        raflow_lib::config::KeyModifier::Ctrl => "ctrl".to_string(),
                        raflow_lib::config::KeyModifier::Alt => "alt".to_string(),
                        raflow_lib::config::KeyModifier::Shift => "shift".to_string(),
                        raflow_lib::config::KeyModifier::Super => "super".to_string(),
                    })
                    .collect();
                let key = match &hotkey_config.key {
                    raflow_lib::config::KeyCode::Char(c) => c.to_string(),
                    raflow_lib::config::KeyCode::Backslash => "\\".to_string(),
                    raflow_lib::config::KeyCode::Space => "Space".to_string(),
                };

                tracing::info!("Registering default hotkey: modifiers={:?}, key={}", modifiers, key);

                // 在新线程中注册热键（因为 register_hotkey 是 async）
                let app_handle = app.handle().clone();
                std::thread::spawn(move || {
                    tracing::info!("Hotkey registration thread started");
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        match raflow_lib::commands::register_hotkey(app_handle, modifiers, key).await {
                            Ok(_) => {
                                tracing::info!("=== DEFAULT HOTKEY REGISTERED SUCCESSFULLY ===");
                            }
                            Err(e) => {
                                tracing::error!("=== FAILED TO REGISTER DEFAULT HOTKEY: {} ===", e);
                            }
                        }
                    });
                });
            } else {
                tracing::info!("Hotkey is disabled in config");
            }

            tracing::info!("=== Tauri setup completed ===");
            Ok(())
        })
        .manage(Arc::new(storage))
        .manage(app.audio_service().clone())
        .manage(Arc::new(app))
        .invoke_handler(tauri::generate_handler![
            // 配置命令
            raflow_lib::commands::get_config,
            raflow_lib::commands::save_config,
            raflow_lib::commands::reset_config,
            raflow_lib::commands::get_config_schema,
            // 音频命令
            raflow_lib::commands::get_audio_devices,
            raflow_lib::commands::start_recording,
            raflow_lib::commands::stop_recording,
            raflow_lib::commands::test_microphone,
            raflow_lib::commands::get_recording_state,
            // 录音命令（带验证）
            raflow_lib::commands::toggle_recording,
            raflow_lib::commands::check_recording_availability,
            // 热键命令
            raflow_lib::commands::register_hotkey,
            raflow_lib::commands::unregister_hotkey,
            raflow_lib::commands::test_hotkey,
            // 更新命令
            raflow_lib::updater::check_for_updates,
            raflow_lib::updater::get_update_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
