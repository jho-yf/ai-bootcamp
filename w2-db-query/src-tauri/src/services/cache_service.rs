/// SQLite 缓存服务：管理数据库连接配置和元数据缓存
use crate::utils::config::get_db_path;
use crate::utils::error::AppError;
use rusqlite::{params, Connection, OptionalExtension};

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
            database_type TEXT NOT NULL DEFAULT 'postgresql',
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

    // 添加数据库类型列（用于旧版本升级）
    let _ = conn.execute(
        "ALTER TABLE connections ADD COLUMN database_type TEXT DEFAULT 'postgresql'",
        [],
    );

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

/// 保存数据库连接配置（密码简单 Base64 编码，实际应该使用 AES-256）
pub fn save_connection(conn: &crate::models::database::DatabaseConnection) -> Result<(), AppError> {
    let db = get_connection()?;

    // 简单 Base64 编码密码（实际应该使用 AES-256）
    use base64::{engine::general_purpose, Engine as _};
    let password_encrypted = general_purpose::STANDARD.encode(&conn.password);

    // 序列化数据库类型
    let database_type = format!("{:?}", conn.database_type).to_lowercase();

    db.execute(
        "INSERT OR REPLACE INTO connections
         (id, name, database_type, host, port, database_name, user, password_encrypted, status, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            conn.id,
            conn.name,
            database_type,
            conn.host,
            conn.port,
            conn.database_name,
            conn.user,
            password_encrypted,
            format!("{:?}", conn.status),
            conn.created_at.to_rfc3339(),
            conn.updated_at.to_rfc3339()
        ],
    )
    .map_err(|e| AppError::CacheStorage(format!("保存连接失败: {}", e)))?;

    Ok(())
}

/// 加载所有数据库连接配置
pub fn load_connections() -> Result<Vec<crate::models::database::DatabaseConnection>, AppError> {
    use crate::models::database::{ConnectionStatus, DatabaseConnection, DatabaseType};
    use chrono::DateTime;

    let db = get_connection()?;
    let mut stmt = db
        .prepare("SELECT id, name, database_type, host, port, database_name, user, password_encrypted, status, created_at, updated_at FROM connections")
        .map_err(|e| AppError::CacheStorage(format!("准备查询失败: {}", e)))?;

    let rows = stmt
        .query_map([], |row| {
            let password_encrypted: String = row.get(7)?;
            use base64::{engine::general_purpose, Engine as _};
            let password_bytes = general_purpose::STANDARD
                .decode(&password_encrypted)
                .map_err(|_| {
                    rusqlite::Error::InvalidColumnType(
                        7,
                        "password".to_string(),
                        rusqlite::types::Type::Text,
                    )
                })?;
            let password = String::from_utf8(password_bytes).map_err(|_| {
                rusqlite::Error::InvalidColumnType(
                    7,
                    "password".to_string(),
                    rusqlite::types::Type::Text,
                )
            })?;

            let database_type_str: String = row.get(2)?;
            let database_type = match database_type_str.as_str() {
                "mysql" => DatabaseType::MySQL,
                _ => DatabaseType::PostgreSQL,
            };

            let status_str: String = row.get(8)?;
            let status = match status_str.as_str() {
                "Connected" => ConnectionStatus::Connected,
                "Connecting" => ConnectionStatus::Connecting,
                "Failed" => ConnectionStatus::Failed,
                _ => ConnectionStatus::Disconnected,
            };

            Ok(DatabaseConnection {
                id: row.get(0)?,
                name: row.get(1)?,
                database_type,
                host: row.get(3)?,
                port: row.get(4)?,
                database_name: row.get(5)?,
                user: row.get(6)?,
                password,
                status,
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(9)?)
                    .map_err(|_| {
                        rusqlite::Error::InvalidColumnType(
                            9,
                            "created_at".to_string(),
                            rusqlite::types::Type::Text,
                        )
                    })?
                    .with_timezone(&chrono::Utc),
                updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(10)?)
                    .map_err(|_| {
                        rusqlite::Error::InvalidColumnType(
                            10,
                            "updated_at".to_string(),
                            rusqlite::types::Type::Text,
                        )
                    })?
                    .with_timezone(&chrono::Utc),
            })
        })
        .map_err(|e| AppError::CacheStorage(format!("查询连接失败: {}", e)))?;

    let mut connections = Vec::new();
    for row in rows {
        connections.push(row?);
    }

    Ok(connections)
}

/// 删除数据库连接
pub fn delete_connection(connection_id: &str) -> Result<(), AppError> {
    let db = get_connection()?;
    db.execute(
        "DELETE FROM connections WHERE id = ?1",
        params![connection_id],
    )
    .map_err(|e| AppError::CacheStorage(format!("删除连接失败: {}", e)))?;
    Ok(())
}

/// 保存元数据到缓存
pub fn save_metadata(connection_id: &str, metadata_json: &str) -> Result<(), AppError> {
    use chrono::Utc;

    let db = get_connection()?;
    let extracted_at = Utc::now().to_rfc3339();

    db.execute(
        "INSERT OR REPLACE INTO metadata (connection_id, metadata_json, extracted_at) VALUES (?1, ?2, ?3)",
        params![connection_id, metadata_json, extracted_at],
    )
    .map_err(|e| AppError::CacheStorage(format!("保存元数据失败: {}", e)))?;

    Ok(())
}

/// 加载元数据缓存
pub fn load_metadata(connection_id: &str) -> Result<Option<String>, AppError> {
    let db = get_connection()?;
    let mut stmt = db
        .prepare("SELECT metadata_json FROM metadata WHERE connection_id = ?1")
        .map_err(|e| AppError::CacheStorage(format!("准备查询失败: {}", e)))?;

    let result = stmt
        .query_row(params![connection_id], |row| Ok(row.get::<_, String>(0)?))
        .optional()
        .map_err(|e| AppError::CacheStorage(format!("查询元数据失败: {}", e)))?;

    Ok(result)
}

/// 保存查询历史记录
///
/// # 参数
/// * `connection_id` - 数据库连接 ID
/// * `query_type` - 查询类型："sql" 或 "natural_language"
/// * `sql` - SQL 查询语句（可选，natural_language 类型时为生成的 SQL）
/// * `prompt` - 自然语言描述（仅 natural_language 类型）
/// * `exec_time_ms` - 执行时间（毫秒，可选）
/// * `status` - 查询状态："success" 或 "failed"
pub fn save_query_history(
    connection_id: &str,
    query_type: &str,
    sql: Option<&str>,
    prompt: Option<&str>,
    exec_time_ms: Option<u64>,
    status: &str,
) -> Result<(), AppError> {
    use chrono::Utc;
    use uuid::Uuid;

    let db = get_connection()?;
    let id = Uuid::new_v4().to_string();
    let executed_at = Utc::now().to_rfc3339();

    db.execute(
        "INSERT INTO query_history
         (id, connection_id, query_type, sql, prompt, executed_at, exec_time_ms, status)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            id,
            connection_id,
            query_type,
            sql,
            prompt,
            executed_at,
            exec_time_ms.map(|v| v as i64),
            status
        ],
    )
    .map_err(|e| AppError::CacheStorage(format!("保存查询历史失败: {}", e)))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::database::{ConnectionStatus, DatabaseConnection};
    use chrono::Utc;
    use std::fs;
    use tempfile::NamedTempFile;

    fn setup_test_db() -> Result<(String, tempfile::TempDir), AppError> {
        // 使用 tempfile 创建临时目录，然后在该目录中创建数据库文件
        // 使用 UUID 确保唯一性
        use uuid::Uuid;
        let db_name = format!("test_{}.db", Uuid::new_v4());

        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join(db_name).to_str().unwrap().to_string();

        // 确保文件不存在
        let _ = fs::remove_file(&db_path);

        // 设置环境变量指向临时数据库
        std::env::set_var("DB_QUERY_DB_PATH", &db_path);

        // 初始化数据库
        init_database()?;

        // 返回路径和 temp_dir（调用者负责保持 temp_dir 存在）
        Ok((db_path, temp_dir))
    }

    fn cleanup_test_db(db_path: &str, _temp_dir: tempfile::TempDir) {
        // temp_dir 会在 drop 时自动清理
        let _ = fs::remove_file(db_path);
        let _ = std::env::remove_var("DB_QUERY_DB_PATH");
    }

    #[test]
    fn test_init_database() {
        let (db_path, temp_dir) = setup_test_db().unwrap();
        // 如果初始化成功，应该能获取连接
        assert!(get_connection().is_ok());
        cleanup_test_db(&db_path, temp_dir);
    }

    #[test]
    fn test_save_and_load_connection() {
        let (db_path, temp_dir) = setup_test_db().unwrap();

        // 确保数据库是空的
        assert_eq!(load_connections().unwrap().len(), 0);

        let connection = DatabaseConnection {
            id: "test-id".to_string(),
            name: "Test DB".to_string(),
            database_type: crate::models::database::DatabaseType::PostgreSQL,
            host: "localhost".to_string(),
            port: 5432,
            database_name: "testdb".to_string(),
            user: "testuser".to_string(),
            password: "testpass".to_string(),
            status: ConnectionStatus::Connected,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        // 保存连接
        assert!(save_connection(&connection).is_ok());

        // 加载连接
        let connections = load_connections().unwrap();
        assert_eq!(connections.len(), 1);
        assert_eq!(connections[0].id, "test-id");
        assert_eq!(connections[0].name, "Test DB");
        assert!(matches!(connections[0].database_type, crate::models::database::DatabaseType::PostgreSQL));
        assert_eq!(connections[0].host, "localhost");
        assert_eq!(connections[0].password, "testpass");

        cleanup_test_db(&db_path, temp_dir);
    }

    #[test]
    fn test_delete_connection() {
        let (db_path, temp_dir) = setup_test_db().unwrap();

        let connection = DatabaseConnection {
            id: "test-id".to_string(),
            name: "Test DB".to_string(),
            database_type: crate::models::database::DatabaseType::PostgreSQL,
            host: "localhost".to_string(),
            port: 5432,
            database_name: "testdb".to_string(),
            user: "testuser".to_string(),
            password: "testpass".to_string(),
            status: ConnectionStatus::Connected,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        save_connection(&connection).unwrap();
        assert_eq!(load_connections().unwrap().len(), 1);

        // 删除连接
        assert!(delete_connection("test-id").is_ok());
        assert_eq!(load_connections().unwrap().len(), 0);

        cleanup_test_db(&db_path, temp_dir);
    }

    #[test]
    fn test_save_and_load_metadata() {
        let (db_path, temp_dir) = setup_test_db().unwrap();

        // 先创建一个连接（因为外键约束）
        let connection = DatabaseConnection {
            id: "test-conn-id".to_string(),
            name: "Test DB".to_string(),
            database_type: crate::models::database::DatabaseType::PostgreSQL,
            host: "localhost".to_string(),
            port: 5432,
            database_name: "testdb".to_string(),
            user: "testuser".to_string(),
            password: "testpass".to_string(),
            status: ConnectionStatus::Connected,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        save_connection(&connection).unwrap();

        let connection_id = "test-conn-id";
        let metadata_json = r#"{"connectionId":"test-conn-id","tables":[],"views":[]}"#;

        // 保存元数据
        assert!(save_metadata(connection_id, metadata_json).is_ok());

        // 加载元数据
        let loaded = load_metadata(connection_id).unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap(), metadata_json);

        // 加载不存在的元数据
        let not_found = load_metadata("non-existent").unwrap();
        assert!(not_found.is_none());

        cleanup_test_db(&db_path, temp_dir);
    }
}
