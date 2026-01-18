// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

mod commands;
mod models;
pub mod services;  // 公开 services 模块以便测试使用
mod utils;

use services::cache_service;
use utils::error::AppError;

use commands::database::{
    add_database, delete_database, list_databases, test_connection, update_database,
};
use commands::metadata::{get_database_metadata, refresh_metadata};
use commands::query::{cancel_query, run_sql_query};

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
        .invoke_handler(tauri::generate_handler![
            greet,
            // 数据库连接管理
            list_databases,
            add_database,
            update_database,
            delete_database,
            test_connection,
            // 元数据管理
            get_database_metadata,
            refresh_metadata,
            // 查询执行
            run_sql_query,
            cancel_query,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
