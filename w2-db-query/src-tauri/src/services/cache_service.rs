use crate::utils::config::get_db_path;
/// SQLite 缓存服务：管理数据库连接配置和元数据缓存
use crate::utils::error::AppError;
use rusqlite::{params, Connection};

/// 初始化 SQLite 数据库，创建必要的表
pub fn init_database() -> Result<(), AppError> {
    let db_path = get_db_path();
    let conn = Connection::open(&db_path)
        .map_err(|e| AppError::CacheStorage(format!("无法打开数据库 {}: {}", db_path, e)))?;

    // 创建数据库连接表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS connections (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            host TEXT NOT NULL,
            port INTEGER NOT NULL,
            database_name TEXT NOT NULL,
            user TEXT NOT NULL,
            password_encrypted TEXT NOT NULL,
            status TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )",
        [],
    )
    .map_err(|e| AppError::CacheStorage(format!("创建 connections 表失败: {}", e)))?;

    // 创建元数据表（JSON 格式）
    conn.execute(
        "CREATE TABLE IF NOT EXISTS metadata (
            connection_id TEXT PRIMARY KEY,
            metadata_json TEXT NOT NULL,
            extracted_at TEXT NOT NULL,
            FOREIGN KEY (connection_id) REFERENCES connections(id) ON DELETE CASCADE
        )",
        [],
    )
    .map_err(|e| AppError::CacheStorage(format!("创建 metadata 表失败: {}", e)))?;

    // 创建查询历史表（统一表，包含 SQL 查询和自然语言查询）
    conn.execute(
        "CREATE TABLE IF NOT EXISTS query_history (
            id TEXT PRIMARY KEY,
            connection_id TEXT NOT NULL,
            query_type TEXT NOT NULL,
            sql TEXT,
            prompt TEXT,
            executed_at TEXT NOT NULL,
            exec_time_ms INTEGER,
            status TEXT NOT NULL,
            FOREIGN KEY (connection_id) REFERENCES connections(id) ON DELETE CASCADE
        )",
        [],
    )
    .map_err(|e| AppError::CacheStorage(format!("创建 query_history 表失败: {}", e)))?;

    // 创建索引
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_query_history_connection_time
         ON query_history(connection_id, executed_at DESC)",
        [],
    )
    .map_err(|e| AppError::CacheStorage(format!("创建索引失败: {}", e)))?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_query_history_type
         ON query_history(query_type)",
        [],
    )
    .map_err(|e| AppError::CacheStorage(format!("创建索引失败: {}", e)))?;

    Ok(())
}

/// 获取数据库连接（用于其他服务使用）
pub fn get_connection() -> Result<Connection, AppError> {
    let db_path = get_db_path();
    Connection::open(&db_path)
        .map_err(|e| AppError::CacheStorage(format!("无法打开数据库 {}: {}", db_path, e)))
}
