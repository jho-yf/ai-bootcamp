// DatabaseService Trait 定义
//
// 这是数据库服务抽象层的核心接口
// 所有数据库服务（PostgreSQL, MySQL, SQLite等）都必须实现此trait
//
// 文件位置: src-tauri/src/services/database/trait.rs

use async_trait::async_trait;
use crate::models::metadata::{DatabaseMetadata};
use crate::utils::error::AppError;
use serde_json::Value;
use std::collections::HashMap;

/// 数据库连接抽象 - 允许任何数据库特定的连接类型
#[derive(Debug)]
pub enum DbConnection {
    PostgreSQL(tokio_postgres::Client),
    MySQL(mysql_async::Pool),
    // 未来扩展：
    // SQLite(rusqlite::Connection),
    // SQL Server(tiberius::Client),
    // Oracle(mongodb::Client),
}

/// 通用行表示 - 用于类型转换
#[derive(Debug, Clone)]
pub struct DbRow {
    pub columns: Vec<String>,
    pub values: Vec<Value>,
}

/// 查询执行结果（带计时信息）
#[derive(Debug, Clone)]
pub struct QueryExecutionResult {
    pub rows: Vec<DbRow>,
    pub exec_time_ms: u64,
}

/// 数据库查询参数 - 数据库无关表示
#[derive(Debug, Clone)]
pub enum QueryParam {
    Null,
    Bool(bool),
    I32(i32),
    I64(i64),
    F64(f64),
    String(String),
    Bytes(Vec<u8>),
}

impl QueryParam {
    /// 转换为 JSON Value
    pub fn to_json(&self) -> Value {
        match self {
            QueryParam::Null => Value::Null,
            QueryParam::Bool(b) => Value::Bool(*b),
            QueryParam::I32(n) => Value::Number(serde_json::Number::from(*n)),
            QueryParam::I64(n) => Value::Number(serde_json::Number::from(*n)),
            QueryParam::F64(f) => serde_json::Number::from_f64(*f)
                .map(Value::Number)
                .unwrap_or(Value::Null),
            QueryParam::String(s) => Value::String(s.clone()),
            QueryParam::Bytes(b) => Value::String(base64::encode(b)),
        }
    }
}

/// SQL 方言信息 - 用于代码生成和 AI 辅助
#[derive(Debug, Clone)]
pub struct SqlDialect {
    pub name: &'static str,
    pub string_quote: char,           // 字符串引号：'
    pub identifier_quote: char,        // 标识符引号：" 或 `
    pub supports_limit: bool,
    pub limit_syntax: LimitSyntax,
    pub parameter_syntax: ParameterSyntax,
    pub exclude_schemas: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LimitSyntax {
    Clause,      // LIMIT n (PostgreSQL, MySQL, SQLite)
    Top,         // SELECT TOP n (SQL Server)
    FetchFirst,  // FETCH FIRST n ROWS ONLY (Oracle)
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParameterSyntax {
    DollarNumeric,   // $1, $2 (PostgreSQL)
    QuestionMark,    // ? (MySQL, SQLite)
    Named,           // :name (Oracle, SQLite named)
}

/// 核心数据库服务 Trait - 抽象所有数据库操作
///
/// # 设计原则
/// - 所有方法都是异步的，支持非阻塞 I/O
/// - 返回统一的 Result 类型，便于错误处理
/// - 使用泛型参数，支持参数化查询
/// - 每个方法都有单一职责
#[async_trait]
pub trait DatabaseService: Send + Sync {
    /// 服务名称（用于日志/调试）
    fn service_name(&self) -> &'static str;

    /// 创建新的数据库连接
    async fn connect(
        &self,
        host: &str,
        port: u16,
        database_name: &str,
        user: &str,
        password: &str,
    ) -> Result<DbConnection, AppError>;

    /// 测试数据库连接
    async fn test_connection(
        &self,
        host: &str,
        port: u16,
        database_name: &str,
        user: &str,
        password: &str,
    ) -> Result<bool, AppError>;

    /// 执行 SQL 查询并返回通用结果
    async fn execute_query(
        &self,
        connection: &DbConnection,
        sql: &str,
        params: &[QueryParam],
    ) -> Result<QueryExecutionResult, AppError>;

    /// 提取完整的数据库元数据
    async fn extract_metadata(
        &self,
        connection: &DbConnection,
        connection_id: &str,
    ) -> Result<DatabaseMetadata, AppError>;

    /// 将数据库特定的行转换为 JSON
    fn convert_row_to_json(
        &self,
        row: &DbRow,
        columns: &[String],
    ) -> Result<HashMap<String, Value>, AppError>;

    /// 获取数据库特定的 SQL 方言信息
    fn get_sql_dialect(&self) -> &SqlDialect;
}

/// 辅助函数：将 DbRow 转换为 JSON HashMap
pub fn convert_row_to_json_default(
    row: &DbRow,
    columns: &[String],
) -> Result<HashMap<String, Value>, AppError> {
    let mut map = HashMap::new();
    for (i, col_name) in columns.iter().enumerate() {
        if let Some(value) = row.values.get(i) {
            map.insert(col_name.clone(), value.clone());
        }
    }
    Ok(map)
}
