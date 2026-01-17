/// 配置管理：读取环境变量
use crate::utils::error::AppError;
use std::env;

/// 获取 OpenAI API Key
pub fn get_openai_api_key() -> Result<String, AppError> {
    env::var("OPENAI_API_KEY")
        .map_err(|_| AppError::Configuration("未设置 OPENAI_API_KEY 环境变量".to_string()))
}

/// 获取 SQLite 数据库路径（默认 ./db_query.db）
pub fn get_db_path() -> String {
    env::var("DB_QUERY_DB_PATH").unwrap_or_else(|_| "./db_query.db".to_string())
}
