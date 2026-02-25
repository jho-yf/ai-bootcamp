// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;
use raflow_lib::{RaFlowApp, ConfigStorage, AppConfig};

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
            // 设置系统托盘
            raflow_lib::tray::setup_tray(app.handle())?;

            // 注册默认热键
            let hotkey_config = config.hotkey.clone();
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

            // 在新线程中注册热键
            let app_handle = app.handle().clone();
            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    if let Err(e) = raflow_lib::commands::register_hotkey(app_handle, modifiers, key).await {
                        tracing::error!("Failed to register default hotkey: {}", e);
                    }
                });
            });

            Ok(())
        })
        .manage(Arc::new(storage))
        .manage(app.audio_service().clone())
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
