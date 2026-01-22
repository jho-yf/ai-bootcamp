/// 配置管理：读取环境变量
use crate::utils::error::AppError;
use std::env;

/// 获取 OpenAI API Key
pub fn get_openai_api_key() -> Result<String, AppError> {
    env::var("OPENAI_API_KEY")
        .map_err(|_| AppError::Configuration("未设置 OPENAI_API_KEY 环境变量".to_string()))
}

/// 获取 OpenAI API Base URL（默认使用官方 endpoint）
///
/// 可以通过 OPENAI_API_BASE 环境变量自定义 endpoint
/// 例如：https://api.openai.com/v1 或 https://your-proxy.com/v1
pub fn get_openai_api_base() -> String {
    env::var("OPENAI_API_BASE").unwrap_or_else(|_| "https://api.openai.com/v1".to_string())
}

/// 获取 OpenAI 模型名称（默认使用 gpt-4o-mini）
///
/// 可以通过 OPENAI_MODEL 环境变量自定义模型
/// 例如：gpt-4o-mini, gpt-4, gpt-3.5-turbo 或其他兼容的模型名称
pub fn get_openai_model() -> String {
    env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-4o-mini".to_string())
}

/// 获取 SQLite 数据库路径（默认 ./db_query.db）
pub fn get_db_path() -> String {
    env::var("DB_QUERY_DB_PATH").unwrap_or_else(|_| "./db_query.db".to_string())
}
