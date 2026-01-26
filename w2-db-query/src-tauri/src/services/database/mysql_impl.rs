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

        // 提取表
        let mut tables = extract_tables(pool).await?;

        // 提取列信息
        let columns = extract_columns(pool, &tables).await?;

        // 提取主键
        let primary_keys_map = extract_primary_keys(pool, &tables).await?;

        // 提取外键
        let foreign_keys_map = extract_foreign_keys(pool, &tables).await?;

        // 组装表信息
        let mut table_column_map: std::collections::HashMap<String, Vec<crate::models::metadata::ColumnInfo>> =
            std::collections::HashMap::new();
        let mut current_columns: Vec<crate::models::metadata::ColumnInfo> = Vec::new();
        let mut current_table_key = String::new();

        for (i, col) in columns.into_iter().enumerate() {
            let table_idx = i / 100;
            if table_idx < tables.len() {
                let table = &tables[table_idx];
                let key = format!("{}.{}", table.schema, table.name);

                if key != current_table_key {
                    if !current_columns.is_empty() {
                        table_column_map.insert(current_table_key.clone(), current_columns.clone());
                    }
                    current_table_key = key;
                    current_columns = Vec::new();
                }
                current_columns.push(col);
            }
        }
        if !current_columns.is_empty() {
            table_column_map.insert(current_table_key, current_columns);
        }

        for table in &mut tables {
            let table_key = format!("{}.{}", table.schema, table.name);

            if let Some(cols) = table_column_map.get(&table_key) {
                table.columns = cols.clone();
            }

            if let Some(pk) = primary_keys_map.iter().find(|(k, _)| **k == table_key) {
                table.primary_keys = pk.1.clone();
                for col in &mut table.columns {
                    col.is_primary_key = table.primary_keys.contains(&col.name);
                }
            }

            if let Some(fk) = foreign_keys_map.iter().find(|(k, _)| **k == table_key) {
                table.foreign_keys = fk.1.clone();
            }
        }

        // 提取视图
        let views = extract_views(pool).await?;

        Ok(DatabaseMetadata {
            connection_id: connection_id.to_string(),
            tables,
            views,
            extracted_at: chrono::Utc::now(),
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

// 辅助函数：提取所有表
async fn extract_tables(pool: &mysql_async::Pool) -> Result<Vec<crate::models::metadata::TableInfo>, AppError> {
    let mut conn = pool
        .get_conn()
        .await
        .map_err(|e| AppError::QueryExecution(format!("获取连接失败: {}", e)))?;

    let rows: Vec<MySqlRow> = conn
        .query(
            "SELECT table_schema, table_name, table_type
             FROM information_schema.tables
             WHERE table_schema NOT IN ('information_schema', 'performance_schema', 'mysql', 'sys')
             AND table_type IN ('BASE TABLE', 'VIEW')
             ORDER BY table_schema, table_name",
        )
        .await
        .map_err(|e| AppError::QueryExecution(format!("查询表失败: {}", e)))?;

    let mut tables = Vec::new();
    for mut row in rows {
        let schema: String = row.take("table_schema").unwrap();
        let name: String = row.take("table_name").unwrap();
        let table_type: String = row.take("table_type").unwrap();

        tables.push(crate::models::metadata::TableInfo {
            schema,
            name,
            table_type,
            columns: vec![],
            primary_keys: vec![],
            foreign_keys: vec![],
        });
    }

    Ok(tables)
}

// 辅助函数：提取表的列信息
async fn extract_columns(
    pool: &mysql_async::Pool,
    tables: &[crate::models::metadata::TableInfo],
) -> Result<Vec<crate::models::metadata::ColumnInfo>, AppError> {
    let mut conn = pool
        .get_conn()
        .await
        .map_err(|e| AppError::QueryExecution(format!("获取连接失败: {}", e)))?;

    let mut all_columns = Vec::new();

    for table in tables {
        let sql = format!(
            "SELECT column_name, data_type, is_nullable, column_default, ordinal_position
             FROM information_schema.columns
             WHERE table_schema = '{}'
             AND table_name = '{}'
             ORDER BY ordinal_position",
            table.schema.replace("'", "''"),
            table.name.replace("'", "''")
        );

        let rows: Vec<MySqlRow> = conn
            .query(&sql)
            .await
            .map_err(|e| AppError::QueryExecution(format!("查询列失败: {}", e)))?;

        for mut row in rows {
            all_columns.push(crate::models::metadata::ColumnInfo {
                name: row.take("column_name").unwrap(),
                data_type: row.take("data_type").unwrap(),
                nullable: matches!(row.take::<String, _>("is_nullable").unwrap().as_str(), "YES"),
                default_value: row.take::<Option<String>, _>("column_default").unwrap(),
                is_primary_key: false,
                ordinal_position: row.take::<i32, _>("ordinal_position").unwrap(),
            });
        }
    }

    Ok(all_columns)
}

// 辅助函数：提取主键信息
async fn extract_primary_keys(
    pool: &mysql_async::Pool,
    tables: &[crate::models::metadata::TableInfo],
) -> Result<Vec<(String, Vec<String>)>, AppError> {
    let mut conn = pool
        .get_conn()
        .await
        .map_err(|e| AppError::QueryExecution(format!("获取连接失败: {}", e)))?;

    let mut result = Vec::new();

    for table in tables {
        let sql = format!(
            "SELECT kcu.column_name
             FROM information_schema.table_constraints tc
             JOIN information_schema.key_column_usage kcu
               ON tc.constraint_name = kcu.constraint_name
               AND tc.table_schema = kcu.table_schema
             WHERE tc.constraint_type = 'PRIMARY KEY'
               AND tc.table_schema = '{}'
               AND tc.table_name = '{}'
             ORDER BY kcu.ordinal_position",
            table.schema.replace("'", "''"),
            table.name.replace("'", "''")
        );

        let rows: Vec<MySqlRow> = conn
            .query(&sql)
            .await
            .map_err(|e| AppError::QueryExecution(format!("查询主键失败: {}", e)))?;

        let primary_keys: Vec<String> = rows
            .into_iter()
            .map(|mut row| row.take::<String, _>("column_name").unwrap())
            .collect();

        result.push((format!("{}.{}", table.schema, table.name), primary_keys));
    }

    Ok(result)
}

// 辅助函数：提取外键信息
async fn extract_foreign_keys(
    pool: &mysql_async::Pool,
    tables: &[crate::models::metadata::TableInfo],
) -> Result<Vec<(String, Vec<crate::models::metadata::ForeignKeyInfo>)>, AppError> {
    let mut conn = pool
        .get_conn()
        .await
        .map_err(|e| AppError::QueryExecution(format!("获取连接失败: {}", e)))?;

    let mut result = Vec::new();

    for table in tables {
        let sql = format!(
            "SELECT
                    kcu.constraint_name,
                    kcu.column_name,
                    kcu.referenced_table_name AS referenced_table,
                    kcu.referenced_column_name AS referenced_column
                 FROM information_schema.key_column_usage kcu
                 JOIN information_schema.referential_constraints rc
                   ON kcu.constraint_name = rc.constraint_name
                   AND kcu.table_schema = rc.constraint_schema
                 WHERE kcu.table_schema = '{}'
                   AND kcu.table_name = '{}'
                   AND kcu.referenced_table_name IS NOT NULL
                 ORDER BY kcu.constraint_name, kcu.ordinal_position",
            table.schema.replace("'", "''"),
            table.name.replace("'", "''")
        );

        let rows: Vec<MySqlRow> = conn
            .query(&sql)
            .await
            .map_err(|e| AppError::QueryExecution(format!("查询外键失败: {}", e)))?;

        let foreign_keys: Vec<crate::models::metadata::ForeignKeyInfo> = rows
            .into_iter()
            .map(|mut row| crate::models::metadata::ForeignKeyInfo {
                constraint_name: row.take("constraint_name").unwrap(),
                column_name: row.take("column_name").unwrap(),
                referenced_table: row.take("referenced_table").unwrap(),
                referenced_column: row.take("referenced_column").unwrap(),
            })
            .collect();

        result.push((format!("{}.{}", table.schema, table.name), foreign_keys));
    }

    Ok(result)
}

// 辅助函数：提取视图信息
async fn extract_views(pool: &mysql_async::Pool) -> Result<Vec<crate::models::metadata::ViewInfo>, AppError> {
    let mut conn = pool
        .get_conn()
        .await
        .map_err(|e| AppError::QueryExecution(format!("获取连接失败: {}", e)))?;

    let rows: Vec<MySqlRow> = conn
        .query(
            "SELECT table_schema, table_name, view_definition
             FROM information_schema.views
             WHERE table_schema NOT IN ('information_schema', 'performance_schema', 'mysql', 'sys')
             ORDER BY table_schema, table_name",
        )
        .await
        .map_err(|e| AppError::QueryExecution(format!("查询视图失败: {}", e)))?;

    let mut views = Vec::new();
    for mut row in rows {
        views.push(crate::models::metadata::ViewInfo {
            schema: row.take("table_schema").unwrap(),
            name: row.take("table_name").unwrap(),
            columns: vec![],
            definition: row.take::<Option<String>, _>("view_definition").unwrap(),
        });
    }

    Ok(views)
}

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
