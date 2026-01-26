// MySQL 服务实现
//
// 实现 DatabaseService trait for MySQL

use super::r#trait::*;
use crate::models::metadata::{DatabaseMetadata};
use crate::utils::error::AppError;
use async_trait::async_trait;
use mysql_async::prelude::*;
use mysql_async::{Row as MySqlRow};
use serde_json::Value;
use std::collections::HashMap;

/// MySQL 服务实现
pub struct MySqlService;

impl MySqlService {
    pub fn new() -> Self {
        Self
    }

    /// 转换 MySQL 行为通用 DbRow
    fn convert_mysql_row_to_db_row(mysql_row: &MySqlRow) -> DbRow {
        let columns: Vec<String> = mysql_row
            .columns()
            .iter()
            .map(|col| col.name_str().to_string())
            .collect();

        let values = columns
            .iter()
            .enumerate()
            .map(|(i, _)| {
                let raw_value = mysql_row.as_ref(i);

                match raw_value {
                    Some(mysql_async::Value::NULL) => Value::Null,

                    Some(mysql_async::Value::Bytes(bytes)) => {
                        // 尝试 UTF-8 解码
                        String::from_utf8(bytes.clone())
                            .map(Value::String)
                            .unwrap_or_else(|_| Value::String(format!("{:?}", bytes)))
                    }

                    Some(mysql_async::Value::Int(num)) => {
                        Value::Number(serde_json::Number::from(*num))
                    }

                    Some(mysql_async::Value::UInt(num)) => {
                        if *num <= i64::MAX as u64 {
                            Value::Number(serde_json::Number::from(*num as i64))
                        } else {
                            // 超出 i64 范围，转为字符串
                            Value::String(num.to_string())
                        }
                    }

                    Some(mysql_async::Value::Float(num)) => {
                        serde_json::Number::from_f64(*num as f64)
                            .map(Value::Number)
                            .unwrap_or(Value::Null)
                    }

                    Some(mysql_async::Value::Double(num)) => {
                        serde_json::Number::from_f64(*num)
                            .map(Value::Number)
                            .unwrap_or(Value::Null)
                    }

                    None => Value::Null,

                    _ => Value::Null,
                }
            })
            .collect();

        DbRow { columns, values }
    }
}

#[async_trait]
impl DatabaseService for MySqlService {
    fn service_name(&self) -> &'static str {
        "MySQL"
    }

    async fn connect(
        &self,
        host: &str,
        port: u16,
        database_name: &str,
        user: &str,
        password: &str,
    ) -> Result<DbConnection, AppError> {
        let url = format!(
            "mysql://{}:{}@{}:{}/{}",
            user, password, host, port, database_name
        );

        let opts = mysql_async::Opts::from_url(&url)
            .map_err(|e| AppError::DatabaseConnection(format!("无效的连接URL: {}", e)))?;

        let pool = mysql_async::Pool::new(opts);

        // 测试连接
        let conn = pool
            .get_conn()
            .await
            .map_err(|e| AppError::DatabaseConnection(format!("连接失败: {}", e)))?;

        let _ = conn.disconnect().await;

        Ok(DbConnection::MySQL(pool))
    }

    async fn test_connection(
        &self,
        host: &str,
        port: u16,
        database_name: &str,
        user: &str,
        password: &str,
    ) -> Result<bool, AppError> {
        let pool = match self.connect(host, port, database_name, user, password).await? {
            DbConnection::MySQL(p) => p,
            _ => return Err(AppError::DatabaseConnection("Invalid connection type".to_string())),
        };

        let mut conn = pool
            .get_conn()
            .await
            .map_err(|e| AppError::DatabaseConnection(format!("获取连接失败: {}", e)))?;

        let _result: Option<u32> = conn
            .query_first("SELECT 1")
            .await
            .map_err(|e| AppError::DatabaseConnection(format!("测试查询失败: {}", e)))?;

        let _ = conn.disconnect().await;
        Ok(true)
    }

    async fn execute_query(
        &self,
        connection: &DbConnection,
        sql: &str,
        _params: &[QueryParam],
    ) -> Result<QueryExecutionResult, AppError> {
        let pool = match connection {
            DbConnection::MySQL(p) => p,
            _ => return Err(AppError::DatabaseConnection("Invalid connection type".to_string())),
        };

        let start_time = std::time::Instant::now();

        let mut conn = pool
            .get_conn()
            .await
            .map_err(|e| AppError::QueryExecution(format!("获取连接失败: {}", e)))?;

        let rows: Vec<MySqlRow> = conn
            .query(sql)
            .await
            .map_err(|e| AppError::QueryExecution(format!("查询执行失败: {}", e)))?;

        let exec_time_ms = start_time.elapsed().as_millis() as u64;

        let db_rows = rows.iter().map(Self::convert_mysql_row_to_db_row).collect();

        Ok(QueryExecutionResult {
            rows: db_rows,
            exec_time_ms,
        })
    }

    async fn extract_metadata(
        &self,
        connection: &DbConnection,
        connection_id: &str,
    ) -> Result<DatabaseMetadata, AppError> {
        let pool = match connection {
            DbConnection::MySQL(p) => p,
            _ => return Err(AppError::DatabaseConnection("Invalid connection type".to_string())),
        };

        // 复用现有的元数据提取逻辑
        let metadata = crate::services::mysql_service::extract_metadata(pool, connection_id).await?;
        Ok(metadata)
    }

    fn convert_row_to_json(
        &self,
        row: &DbRow,
        columns: &[String],
    ) -> Result<HashMap<String, Value>, AppError> {
        convert_row_to_json_default(row, columns)
    }

    fn get_sql_dialect(&self) -> &SqlDialect {
        &MYSQL_DIALECT
    }
}

/// MySQL SQL 方言
static MYSQL_DIALECT: SqlDialect = SqlDialect {
    name: "MySQL",
    string_quote: '\'',
    identifier_quote: '`',
    supports_limit: true,
    limit_syntax: LimitSyntax::Clause,
    parameter_syntax: ParameterSyntax::QuestionMark,
    exclude_schemas: &["information_schema", "performance_schema", "mysql", "sys"],
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_name() {
        let service = MySqlService::new();
        assert_eq!(service.service_name(), "MySQL");
    }

    #[test]
    fn test_sql_dialect() {
        let service = MySqlService::new();
        let dialect = service.get_sql_dialect();

        assert_eq!(dialect.name, "MySQL");
        assert_eq!(dialect.string_quote, '\'');
        assert_eq!(dialect.identifier_quote, '`');
        assert_eq!(dialect.supports_limit, true);
        assert_eq!(dialect.limit_syntax, LimitSyntax::Clause);
        assert_eq!(dialect.parameter_syntax, ParameterSyntax::QuestionMark);
    }
}
