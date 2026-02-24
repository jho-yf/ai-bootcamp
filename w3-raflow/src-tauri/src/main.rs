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
        .setup(|app| {
            // 设置系统托盘
            raflow_lib::tray::setup_tray(app.handle())?;
            Ok(())
        })
        .manage(Arc::new(storage))
        .manage(Arc::new(app.audio_service().clone()))
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
