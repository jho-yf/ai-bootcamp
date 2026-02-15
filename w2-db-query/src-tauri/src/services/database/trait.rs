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
    fn test_query_param_creation() {
        // 测试 QueryParam 所有变体能够正确创建
        let _param_null = QueryParam::Null;
        let _param_bool = QueryParam::Bool(true);
        let _param_bool_false = QueryParam::Bool(false);
        let _param_i32 = QueryParam::I32(42);
        let _param_i32_min = QueryParam::I32(i32::MIN);
        let _param_i32_max = QueryParam::I32(i32::MAX);
        let _param_i64 = QueryParam::I64(123456789);
        let _param_f64 = QueryParam::F64(3.14);
        let _param_string = QueryParam::String("test".to_string());
        let _param_bytes = QueryParam::Bytes(vec![1, 2, 3]);
    }

    #[test]
    fn test_query_param_edge_cases() {
        // 测试 QueryParam 边界值和特殊情况

        // 测试布尔值两个状态
        let param_true = QueryParam::Bool(true);
        let param_false = QueryParam::Bool(false);
        assert!(matches!(param_true, QueryParam::Bool(true)));
        assert!(matches!(param_false, QueryParam::Bool(false)));

        // 测试整数边界值
        let param_i32_min = QueryParam::I32(i32::MIN);
        let param_i32_max = QueryParam::I32(i32::MAX);
        assert!(matches!(param_i32_min, QueryParam::I32(x) if x == i32::MIN));
        assert!(matches!(param_i32_max, QueryParam::I32(x) if x == i32::MAX));

        // 测试 I64 边界值
        let param_i64_min = QueryParam::I64(i64::MIN);
        let param_i64_max = QueryParam::I64(i64::MAX);
        assert!(matches!(param_i64_min, QueryParam::I64(x) if x == i64::MIN));
        assert!(matches!(param_i64_max, QueryParam::I64(x) if x == i64::MAX));

        // 测试浮点数特殊情况
        let param_f64_zero = QueryParam::F64(0.0);
        let param_f64_negative = QueryParam::F64(-3.14);
        let param_f64_very_small = QueryParam::F64(0.000001);
        let param_f64_very_large = QueryParam::F64(999999.999999);
        assert!(matches!(param_f64_zero, QueryParam::F64(x) if x == 0.0));
        assert!(matches!(param_f64_negative, QueryParam::F64(x) if x < 0.0));
        assert!(matches!(param_f64_very_small, QueryParam::F64(x) if x > 0.0 && x < 0.00001));
        assert!(matches!(param_f64_very_large, QueryParam::F64(x) if x > 999999.0));

        // 测试空字符串
        let param_empty_string = QueryParam::String(String::new());
        assert!(matches!(param_empty_string, QueryParam::String(s) if s.is_empty()));

        // 测试空字节向量
        let param_empty_bytes = QueryParam::Bytes(vec![]);
        assert!(matches!(param_empty_bytes, QueryParam::Bytes(b) if b.is_empty()));

        // 测试包含特殊字符的字符串
        let param_special_chars = QueryParam::String("你好世界\n\t\r".to_string());
        assert!(matches!(param_special_chars, QueryParam::String(s) if s.len() > 0));
    }

    #[test]
    fn test_convert_row_to_json_default() {
        // 测试正常的行转换
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
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_convert_row_with_null_values() {
        // 测试包含 NULL 值的行转换
        let row = DbRow {
            columns: vec!["id".to_string(), "name".to_string(), "email".to_string()],
            values: vec![
                Value::Number(serde_json::Number::from(1)),
                Value::Null,
                Value::Null,
            ],
        };

        let result = convert_row_to_json_default(&row, &row.columns).unwrap();

        assert_eq!(result.get("id"), Some(&Value::Number(serde_json::Number::from(1))));
        assert_eq!(result.get("name"), Some(&Value::Null));
        assert_eq!(result.get("email"), Some(&Value::Null));
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_convert_row_with_mixed_types() {
        // 测试混合数据类型的行转换
        let row = DbRow {
            columns: vec![
                "int_col".to_string(),
                "str_col".to_string(),
                "bool_col".to_string(),
                "null_col".to_string(),
                "float_col".to_string(),
            ],
            values: vec![
                Value::Number(serde_json::Number::from(42)),
                Value::String("test string".to_string()),
                Value::Bool(true),
                Value::Null,
                Value::Number(serde_json::Number::from_f64(3.14159).unwrap()),
            ],
        };

        let result = convert_row_to_json_default(&row, &row.columns).unwrap();

        assert_eq!(result.get("int_col"), Some(&Value::Number(serde_json::Number::from(42))));
        assert_eq!(result.get("str_col"), Some(&Value::String("test string".to_string())));
        assert_eq!(result.get("bool_col"), Some(&Value::Bool(true)));
        assert_eq!(result.get("null_col"), Some(&Value::Null));
        assert_eq!(result.len(), 5);
    }

    #[test]
    fn test_convert_row_with_empty_columns() {
        // 测试空列的行转换
        let row = DbRow {
            columns: vec![],
            values: vec![],
        };

        let result = convert_row_to_json_default(&row, &[]).unwrap();
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_convert_row_with_subset_columns() {
        // 测试只转换部分列
        let row = DbRow {
            columns: vec!["id".to_string(), "name".to_string(), "email".to_string()],
            values: vec![
                Value::Number(serde_json::Number::from(1)),
                Value::String("Alice".to_string()),
                Value::String("alice@example.com".to_string()),
            ],
        };

        // 只获取前两列
        let subset_columns = vec!["id".to_string(), "name".to_string()];
        let result = convert_row_to_json_default(&row, &subset_columns).unwrap();

        assert_eq!(result.get("id"), Some(&Value::Number(serde_json::Number::from(1))));
        assert_eq!(result.get("name"), Some(&Value::String("Alice".to_string())));
        assert_eq!(result.get("email"), None);  // email 不在结果中
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_sql_dialect_configuration() {
        // 测试 PostgreSQL 方言配置
        let pg_dialect = SqlDialect {
            name: "PostgreSQL",
            string_quote: '\'',
            identifier_quote: '"',
            supports_limit: true,
            limit_syntax: LimitSyntax::Clause,
            parameter_syntax: ParameterSyntax::DollarNumeric,
            exclude_schemas: &["information_schema", "pg_catalog", "pg_toast"],
        };

        // 验证基本属性
        assert_eq!(pg_dialect.name, "PostgreSQL");
        assert_eq!(pg_dialect.string_quote, '\'');
        assert_eq!(pg_dialect.identifier_quote, '"');
        assert!(pg_dialect.supports_limit);
        assert_eq!(pg_dialect.limit_syntax, LimitSyntax::Clause);
        assert_eq!(pg_dialect.parameter_syntax, ParameterSyntax::DollarNumeric);

        // 验证排除的模式
        assert_eq!(pg_dialect.exclude_schemas.len(), 3);
        assert!(pg_dialect.exclude_schemas.contains(&"information_schema"));

        // 测试 MySQL 方言配置
        let mysql_dialect = SqlDialect {
            name: "MySQL",
            string_quote: '\'',
            identifier_quote: '`',
            supports_limit: true,
            limit_syntax: LimitSyntax::Clause,
            parameter_syntax: ParameterSyntax::QuestionMark,
            exclude_schemas: &["information_schema", "performance_schema", "mysql", "sys"],
        };

        // 验证 MySQL 特有的配置
        assert_eq!(mysql_dialect.identifier_quote, '`');
        assert_eq!(mysql_dialect.parameter_syntax, ParameterSyntax::QuestionMark);
        assert_eq!(mysql_dialect.exclude_schemas.len(), 4);
    }

    #[test]
    fn test_sql_dialect_differences() {
        // 测试不同数据库方言的差异
        let pg_dialect = SqlDialect {
            name: "PostgreSQL",
            string_quote: '\'',
            identifier_quote: '"',
            supports_limit: true,
            limit_syntax: LimitSyntax::Clause,
            parameter_syntax: ParameterSyntax::DollarNumeric,
            exclude_schemas: &["information_schema"],
        };

        let mysql_dialect = SqlDialect {
            name: "MySQL",
            string_quote: '\'',
            identifier_quote: '`',
            supports_limit: true,
            limit_syntax: LimitSyntax::Clause,
            parameter_syntax: ParameterSyntax::QuestionMark,
            exclude_schemas: &["information_schema"],
        };

        // 验证关键差异
        assert_ne!(
            pg_dialect.identifier_quote,
            mysql_dialect.identifier_quote
        );

        assert_ne!(
            pg_dialect.parameter_syntax,
            mysql_dialect.parameter_syntax
        );

        // 验证相同点
        assert_eq!(pg_dialect.string_quote, mysql_dialect.string_quote);
        assert_eq!(pg_dialect.supports_limit, mysql_dialect.supports_limit);
    }

    #[test]
    fn test_limit_syntax_variants() {
        // 测试 LimitSyntax 所有变体
        let clause_syntax = LimitSyntax::Clause;
        let top_syntax = LimitSyntax::Top;
        let fetch_first_syntax = LimitSyntax::FetchFirst;

        // 验证 PartialEq 实现
        assert_eq!(clause_syntax, LimitSyntax::Clause);
        assert_eq!(top_syntax, LimitSyntax::Top);
        assert_eq!(fetch_first_syntax, LimitSyntax::FetchFirst);

        // 验证变体之间的差异
        assert_ne!(clause_syntax, top_syntax);
        assert_ne!(top_syntax, fetch_first_syntax);
        assert_ne!(fetch_first_syntax, clause_syntax);
    }

    #[test]
    fn test_parameter_syntax_variants() {
        // 测试 ParameterSyntax 所有变体
        let dollar_syntax = ParameterSyntax::DollarNumeric;
        let question_syntax = ParameterSyntax::QuestionMark;
        let named_syntax = ParameterSyntax::Named;

        // 验证 PartialEq 实现
        assert_eq!(dollar_syntax, ParameterSyntax::DollarNumeric);
        assert_eq!(question_syntax, ParameterSyntax::QuestionMark);
        assert_eq!(named_syntax, ParameterSyntax::Named);

        // 验证变体之间的差异
        assert_ne!(dollar_syntax, question_syntax);
        assert_ne!(question_syntax, named_syntax);
        assert_ne!(named_syntax, dollar_syntax);
    }

    #[test]
    fn test_query_result_with_execution_time() {
        // 测试 QueryExecutionResult 结构
        let rows = vec![
            DbRow {
                columns: vec!["id".to_string()],
                values: vec![Value::Number(serde_json::Number::from(1))],
            }
        ];

        let result = QueryExecutionResult {
            rows: rows.clone(),
            exec_time_ms: 123,
        };

        assert_eq!(result.rows.len(), 1);
        assert_eq!(result.exec_time_ms, 123);
        assert_eq!(result.rows[0].columns.len(), 1);
    }

    #[test]
    fn test_db_connection_debug() {
        // 测试 DbConnection 的 Debug trait（虽然不能直接实例化，但可以验证结构存在）
        // 这个测试确保 DbConnection 结构定义正确
        // 实际测试需要真实的数据库连接

        // 验证类型存在且可以正常编译
        let _ = std::mem::size_of::<DbConnection>();
        let _ = std::mem::size_of::<DbRow>();
        let _ = std::mem::size_of::<QueryExecutionResult>();

        // DbRow 应该是 Clone 的
        let row = DbRow {
            columns: vec!["test".to_string()],
            values: vec![Value::Null],
        };

        let cloned_row = row.clone();
        assert_eq!(row.columns, cloned_row.columns);
        assert_eq!(row.values, cloned_row.values);
    }
}
