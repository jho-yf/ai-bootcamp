// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

mod services;
mod utils;

use services::cache_service;
use utils::error::AppError;

/// 初始化应用：设置数据库等基础设施
fn init_app() -> Result<(), AppError> {
    // 初始化 SQLite 数据库
    cache_service::init_database()?;
    Ok(())
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 初始化应用
    if let Err(e) = init_app() {
        eprintln!("应用初始化失败: {}", e);
        std::process::exit(1);
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
