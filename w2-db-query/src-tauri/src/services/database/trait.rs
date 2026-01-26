// DatabaseService Trait 定义
//
// 数据库服务抽象层
// 定义所有数据库服务必须实现的接口

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

/// SQL 方言信息 - 用于代码生成和 AI 辅助
#[derive(Debug, Clone)]
pub struct SqlDialect {
    pub name: &'static str,
    pub string_quote: char,
    pub identifier_quote: char,
    pub supports_limit: bool,
    pub limit_syntax: LimitSyntax,
    pub parameter_syntax: ParameterSyntax,
    pub exclude_schemas: &'static [&'static str],
}

#[derive(Debug, Clone, PartialEq)]
pub enum LimitSyntax {
    Clause,
    Top,
    FetchFirst,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParameterSyntax {
    DollarNumeric,
    QuestionMark,
    Named,
}

/// 核心数据库服务 Trait - 抽象所有数据库操作
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_param() {
        // Basic test that QueryParam can be created
        let _param = QueryParam::Null;
        let _param = QueryParam::Bool(true);
        let _param = QueryParam::I32(42);
        let _param = QueryParam::I64(42);
        let _param = QueryParam::F64(3.14);
        let _param = QueryParam::String("test".to_string());
        let _param = QueryParam::Bytes(vec![1, 2, 3]);
    }

    #[test]
    fn test_convert_row_to_json_default() {
        let row = DbRow {
            columns: vec!["id".to_string(), "name".to_string()],
            values: vec![
                Value::Number(serde_json::Number::from(1)),
                Value::String("test".to_string())
            ],
        };

        let result = convert_row_to_json_default(&row, &row.columns).unwrap();

        assert_eq!(result.get("id"), Some(&Value::Number(serde_json::Number::from(1))));
        assert_eq!(result.get("name"), Some(&Value::String("test".to_string())));
    }

    #[test]
    fn test_sql_dialect() {
        let dialect = SqlDialect {
            name: "Test",
            string_quote: '\'',
            identifier_quote: '"',
            supports_limit: true,
            limit_syntax: LimitSyntax::Clause,
            parameter_syntax: ParameterSyntax::DollarNumeric,
            exclude_schemas: &["test_schema"],
        };

        assert_eq!(dialect.name, "Test");
        assert_eq!(dialect.string_quote, '\'');
        assert_eq!(dialect.identifier_quote, '"');
        assert_eq!(dialect.supports_limit, true);
        assert_eq!(dialect.limit_syntax, LimitSyntax::Clause);
        assert_eq!(dialect.parameter_syntax, ParameterSyntax::DollarNumeric);
        assert_eq!(dialect.exclude_schemas, vec!["test_schema"]);
    }
}
