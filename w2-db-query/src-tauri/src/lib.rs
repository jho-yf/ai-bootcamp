// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

mod commands;
pub mod models; // 公开 models 模块以便测试使用
pub mod services; // 公开 services 模块以便测试使用
pub mod utils; // 公开 utils 模块以便测试使用

use services::cache_service;
use utils::error::AppError;

use commands::ai::{generate_sql_from_nl, run_nl_query};
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
    // 加载 .env 文件
    // Tauri 应用运行时的工作目录可能不同，需要尝试多个位置
    use std::path::Path;

    let mut env_loaded = false;

    // 获取当前可执行文件的目录
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            // 尝试可执行文件目录
            let env_in_exe_dir = exe_dir.join(".env");
            if env_in_exe_dir.exists() && dotenv::from_path(&env_in_exe_dir).is_ok() {
                eprintln!("已加载环境变量文件: {:?}", env_in_exe_dir);
                env_loaded = true;
            }

            // 尝试可执行文件目录的上级目录（适用于开发环境）
            if !env_loaded {
                let env_in_parent = exe_dir.parent().and_then(|p| Some(p.join(".env")));
                if let Some(ref env_path) = env_in_parent {
                    if env_path.exists() && dotenv::from_path(env_path).is_ok() {
                        eprintln!("已加载环境变量文件: {:?}", env_path);
                        env_loaded = true;
                    }
                }
            }
        }
    }

    // 尝试当前工作目录
    if !env_loaded {
        if Path::new(".env").exists() && dotenv::from_filename(".env").is_ok() {
            eprintln!("已加载环境变量文件: .env (当前工作目录)");
            env_loaded = true;
        }
    }

    // 尝试上级目录（项目根目录，适用于从 src-tauri 目录运行）
    if !env_loaded {
        if Path::new("../.env").exists() && dotenv::from_filename("../.env").is_ok() {
            eprintln!("已加载环境变量文件: ../.env (项目根目录)");
            env_loaded = true;
        }
    }

    // 最后尝试 dotenv 的默认行为（从当前目录向上查找）
    if !env_loaded {
        if let Ok(path) = dotenv::dotenv() {
            eprintln!("已加载环境变量文件: {:?}", path);
            env_loaded = true;
        }
    }

    if !env_loaded {
        eprintln!("警告: 未找到 .env 文件，将仅使用系统环境变量");
        eprintln!("提示: 请确保 .env 文件位于项目根目录或可执行文件目录");
    }

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
            // AI 自然语言查询
            generate_sql_from_nl,
            run_nl_query,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
