// MySQL 服务实现
//
// 实现 DatabaseService trait for MySQL
//
// 文件位置: src-tauri/src/services/database/mysql_impl.rs

use super::trait::*;
use crate::models::metadata::{
    DatabaseMetadata, TableInfo, ViewInfo, ColumnInfo, ForeignKeyInfo
};
use crate::utils::error::AppError;
use async_trait::async_trait;
use mysql_async::{Pool, Row as MySqlRow, Value as MySqlValue};
use chrono::Utc;
use std::collections::HashMap;

/// MySQL 服务实现
pub struct MySqlService;

impl MySqlService {
    pub fn new() -> Self {
        Self
    }

    /// 转换 MySQL 行为通用 DbRow
    ///
    /// # 类型转换规则
    /// - MYSQL_TYPE_TINY (bool) → Bool
    /// - Value::Int → Number
    /// - Value::UInt → Number (如果太大则转为 String)
    /// - Value::Float/Double → Number
    /// - Value::Bytes → String (UTF-8)
    /// - Value::NULL → Null
    fn convert_mysql_row_to_db_row(mysql_row: &MySqlRow) -> DbRow {
        let columns = mysql_row
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
                    Some(MySqlValue::NULL) => Value::Null,

                    Some(MySqlValue::Bytes(bytes)) => {
                        // 尝试 UTF-8 解码
                        String::from_utf8(bytes.clone())
                            .map(Value::String)
                            .unwrap_or_else(|_| Value::String(format!("{:?}", bytes)))
                    }

                    Some(MySqlValue::Int(num)) => {
                        Value::Number(serde_json::Number::from(*num))
                    }

                    Some(MySqlValue::UInt(num)) => {
                        if *num <= i64::MAX as u64 {
                            Value::Number(serde_json::Number::from(*num as i64))
                        } else {
                            // 超出 i64 范围，转为字符串
                            Value::String(num.to_string())
                        }
                    }

                    Some(MySqlValue::Float(num)) => {
                        serde_json::Number::from_f64(*num as f64)
                            .map(Value::Number)
                            .unwrap_or(Value::Null)
                    }

                    Some(MySqlValue::Double(num)) => {
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

    /// 检测字段是否为布尔类型
    fn is_boolean_column(column: &mysql_async::Column) -> bool {
        column.column_type() == mysql_async::constants::ColumnType::MYSQL_TYPE_TINY
            && column.length() == 1
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
        match self.connect(host, port, database_name, user, password).await? {
            DbConnection::MySQL(pool) => {
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
            _ => Err(AppError::DatabaseConnection("Invalid connection type".to_string())),
        }
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

        // 提取表信息
        let tables = self.extract_tables(pool).await?;
        let views = self.extract_views(pool).await?;

        Ok(DatabaseMetadata {
            connection_id: connection_id.to_string(),
            tables,
            views,
            extracted_at: Utc::now(),
        })
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

impl MySqlService {
    /// 提取表信息
    async fn extract_tables(&self, pool: &Pool) -> Result<Vec<TableInfo>, AppError> {
        let query = r#"
            SELECT
                TABLE_SCHEMA as table_schema,
                TABLE_NAME as table_name,
                TABLE_COMMENT as table_comment
            FROM information_schema.TABLES
            WHERE TABLE_TYPE = 'BASE TABLE'
                AND TABLE_SCHEMA NOT IN ('information_schema', 'performance_schema', 'mysql', 'sys')
            ORDER BY TABLE_SCHEMA, TABLE_NAME
        "#;

        let mut conn = pool
            .get_conn()
            .await
            .map_err(|e| AppError::MetadataExtraction(format!("获取连接失败: {}", e)))?;

        let rows: Vec<MySqlRow> = conn
            .query(query)
            .await
            .map_err(|e| AppError::MetadataExtraction(format!("提取表失败: {}", e)))?;

        let mut tables = Vec::new();
        for row in rows {
            let schema: String = row.get("table_schema").unwrap_or_default();
            let name: String = row.get("table_name").unwrap_or_default();
            let comment: Option<String> = row.get("table_comment");

            tables.push(TableInfo {
                schema: Some(schema),
                name,
                comment,
                columns: vec![], // 稍后填充
            });
        }

        Ok(tables)
    }

    /// 提取视图信息
    async fn extract_views(&self, pool: &Pool) -> Result<Vec<ViewInfo>, AppError> {
        let query = r#"
            SELECT
                TABLE_SCHEMA as table_schema,
                TABLE_NAME as table_name,
                TABLE_COMMENT as view_comment
            FROM information_schema.VIEWS
            WHERE TABLE_SCHEMA NOT IN ('information_schema', 'performance_schema', 'mysql', 'sys')
            ORDER BY TABLE_SCHEMA, TABLE_NAME
        "#;

        let mut conn = pool
            .get_conn()
            .await
            .map_err(|e| AppError::MetadataExtraction(format!("获取连接失败: {}", e)))?;

        let rows: Vec<MySqlRow> = conn
            .query(query)
            .await
            .map_err(|e| AppError::MetadataExtraction(format!("提取视图失败: {}", e)))?;

        let mut views = Vec::new();
        for row in rows {
            let schema: String = row.get("table_schema").unwrap_or_default();
            let name: String = row.get("table_name").unwrap_or_default();
            let comment: Option<String> = row.get("view_comment");

            views.push(ViewInfo {
                schema: Some(schema),
                name,
                comment,
            });
        }

        Ok(views)
    }
}

/// MySQL SQL 方言
const MYSQL_DIALECT: SqlDialect = SqlDialect {
    name: "MySQL",
    string_quote: '\'',
    identifier_quote: '`',
    supports_limit: true,
    limit_syntax: LimitSyntax::Clause,
    parameter_syntax: ParameterSyntax::QuestionMark,
    exclude_schemas: vec!["information_schema", "performance_schema", "mysql", "sys"],
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
