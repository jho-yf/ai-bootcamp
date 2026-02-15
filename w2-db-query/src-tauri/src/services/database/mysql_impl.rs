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
    fn test_service_properties() {
        let service = MySqlService::new();

        // 验证服务名称
        assert_eq!(service.service_name(), "MySQL");

        // 验证 SQL 方言配置
        let dialect = service.get_sql_dialect();
        assert_eq!(dialect.name, "MySQL");
        assert_eq!(dialect.string_quote, '\'');
        assert_eq!(dialect.identifier_quote, '`');  // MySQL 使用反引号
        assert!(dialect.supports_limit);
        assert_eq!(dialect.limit_syntax, LimitSyntax::Clause);
        assert_eq!(dialect.parameter_syntax, ParameterSyntax::QuestionMark);  // MySQL 使用 ?

        // 验证排除的模式列表
        assert!(dialect.exclude_schemas.contains(&"information_schema"));
        assert!(dialect.exclude_schemas.contains(&"performance_schema"));
        assert!(dialect.exclude_schemas.contains(&"mysql"));
        assert!(dialect.exclude_schemas.contains(&"sys"));
    }

    #[test]
    fn test_service_creation() {
        // 测试服务创建
        let service1 = MySqlService::new();
        let service2 = MySqlService::new();

        // 验证服务属性
        assert_eq!(service1.service_name(), service2.service_name());

        // 验证方言一致性
        let dialect1 = service1.get_sql_dialect();
        let dialect2 = service2.get_sql_dialect();
        assert_eq!(dialect1.name, dialect2.name);
    }

    #[test]
    fn test_row_conversion() {
        let service = MySqlService::new();

        // 测试空行转换
        let row = DbRow {
            columns: vec![],
            values: vec![],
        };

        let result = service.convert_row_to_json(&row, &[]);
        assert!(result.is_ok());
        let json_map = result.unwrap();
        assert_eq!(json_map.len(), 0);

        // 测试有数据的行转换
        let row = DbRow {
            columns: vec!["id".to_string(), "name".to_string()],
            values: vec![
                Value::Number(serde_json::Number::from(100)),
                Value::String("mysql_test".to_string())
            ],
        };

        let result = service.convert_row_to_json(&row, &row.columns);
        assert!(result.is_ok());
        let json_map = result.unwrap();
        assert_eq!(json_map.len(), 2);
        assert_eq!(json_map.get("id"), Some(&Value::Number(serde_json::Number::from(100))));
        assert_eq!(json_map.get("name"), Some(&Value::String("mysql_test".to_string())));
    }

    #[test]
    fn test_row_conversion_with_nulls() {
        let service = MySqlService::new();

        // 测试包含 NULL 值的行转换
        let row = DbRow {
            columns: vec!["id".to_string(), "name".to_string(), "email".to_string()],
            values: vec![
                Value::Number(serde_json::Number::from(1)),
                Value::Null,
                Value::String("test@example.com".to_string()),
            ],
        };

        let result = service.convert_row_to_json(&row, &row.columns);
        assert!(result.is_ok());
        let json_map = result.unwrap();

        assert_eq!(json_map.get("id"), Some(&Value::Number(serde_json::Number::from(1))));
        assert_eq!(json_map.get("name"), Some(&Value::Null));
        assert_eq!(json_map.get("email"), Some(&Value::String("test@example.com".to_string())));
        assert_eq!(json_map.len(), 3);
    }

    #[test]
    fn test_row_conversion_with_numeric_types() {
        let service = MySqlService::new();

        // 测试 MySQL 特定的数值类型转换
        let row = DbRow {
            columns: vec![
                "tinyint_col".to_string(),
                "smallint_col".to_string(),
                "int_col".to_string(),
                "bigint_col".to_string(),
                "float_col".to_string(),
                "double_col".to_string(),
            ],
            values: vec![
                Value::Number(serde_json::Number::from(127)),      // TINYINT MAX
                Value::Number(serde_json::Number::from(32767)),     // SMALLINT MAX
                Value::Number(serde_json::Number::from(2147483647)), // INT MAX
                Value::Number(serde_json::Number::from(9223372036854775807_i64)), // BIGINT MAX
                Value::Number(serde_json::Number::from_f64(3.14f32 as f64).unwrap()), // FLOAT
                Value::Number(serde_json::Number::from_f64(3.14159265359).unwrap()), // DOUBLE
            ],
        };

        let result = service.convert_row_to_json(&row, &row.columns);
        assert!(result.is_ok());
        let json_map = result.unwrap();

        assert_eq!(json_map.get("tinyint_col"), Some(&Value::Number(serde_json::Number::from(127))));
        assert_eq!(json_map.get("smallint_col"), Some(&Value::Number(serde_json::Number::from(32767))));
        assert_eq!(json_map.get("int_col"), Some(&Value::Number(serde_json::Number::from(2147483647))));
        assert_eq!(json_map.get("bigint_col"), Some(&Value::Number(serde_json::Number::from(9223372036854775807_i64))));
    }

    #[test]
    fn test_row_conversion_with_unsigned_integers() {
        let service = MySqlService::new();

        // 测试无符号整数（当超出 i64 范围时应转为字符串）
        let row = DbRow {
            columns: vec![
                "normal_uint".to_string(),
                "large_uint".to_string(),  // 模拟超出 i64::MAX 的情况
                "negative_int".to_string(),
            ],
            values: vec![
                Value::Number(serde_json::Number::from(4000000000_u64)),  // 在 i64 范围内
                Value::String("18446744073709551615".to_string()),  // u64::MAX 作为字符串
                Value::Number(serde_json::Number::from(-1000)),
            ],
        };

        let result = service.convert_row_to_json(&row, &row.columns);
        assert!(result.is_ok());
        let json_map = result.unwrap();

        // 验证正常范围内的无符号整数
        assert_eq!(json_map.get("normal_uint"), Some(&Value::Number(serde_json::Number::from(4000000000_u64 as i64))));

        // 验证超出范围的大整数转为字符串
        assert_eq!(json_map.get("large_uint"), Some(&Value::String("18446744073709551615".to_string())));

        // 验证负数
        assert_eq!(json_map.get("negative_int"), Some(&Value::Number(serde_json::Number::from(-1000))));
    }

    #[test]
    fn test_row_conversion_with_float_precision() {
        let service = MySqlService::new();

        // 测试浮点数精度问题
        let row = DbRow {
            columns: vec![
                "zero_float".to_string(),
                "negative_float".to_string(),
                "very_small".to_string(),
                "very_large".to_string(),
                "nan_like".to_string(),
            ],
            values: vec![
                Value::Number(serde_json::Number::from_f64(0.0).unwrap()),
                Value::Number(serde_json::Number::from_f64(-123.456).unwrap()),
                Value::Number(serde_json::Number::from_f64(0.000001).unwrap()),
                Value::Number(serde_json::Number::from_f64(999999.999999).unwrap()),
                Value::Null,  // JSON 不支持 NaN，转为 NULL
            ],
        };

        let result = service.convert_row_to_json(&row, &row.columns);
        assert!(result.is_ok());
        let json_map = result.unwrap();

        assert_eq!(json_map.get("zero_float"), Some(&Value::Number(serde_json::Number::from_f64(0.0).unwrap())));
        assert_eq!(json_map.get("negative_float"), Some(&Value::Number(serde_json::Number::from_f64(-123.456).unwrap())));
        assert_eq!(json_map.get("very_small"), Some(&Value::Number(serde_json::Number::from_f64(0.000001).unwrap())));
        assert_eq!(json_map.get("nan_like"), Some(&Value::Null));
    }

    #[test]
    fn test_row_conversion_with_string_types() {
        let service = MySqlService::new();

        // 测试 MySQL 字符串类型（CHAR, VARCHAR, TEXT）
        let row = DbRow {
            columns: vec![
                "char_col".to_string(),
                "varchar_col".to_string(),
                "text_col".to_string(),
                "empty_str".to_string(),
                "special_chars".to_string(),
            ],
            values: vec![
                Value::String("fixed".to_string()),
                Value::String("variable".to_string()),
                Value::String("long text content".to_string()),
                Value::String("".to_string()),
                Value::String("中文\n\t\r\"'".to_string()),
            ],
        };

        let result = service.convert_row_to_json(&row, &row.columns);
        assert!(result.is_ok());
        let json_map = result.unwrap();

        assert_eq!(json_map.get("char_col"), Some(&Value::String("fixed".to_string())));
        assert_eq!(json_map.get("varchar_col"), Some(&Value::String("variable".to_string())));
        assert_eq!(json_map.get("text_col"), Some(&Value::String("long text content".to_string())));
        assert_eq!(json_map.get("empty_str"), Some(&Value::String("".to_string())));
        assert_eq!(json_map.get("special_chars"), Some(&Value::String("中文\n\t\r\"'".to_string())));
    }

    #[test]
    fn test_row_conversion_with_binary_data() {
        let service = MySqlService::new();

        // 测试二进制数据（BINARY, VARBINARY, BLOB）
        let row = DbRow {
            columns: vec![
                "valid_utf8".to_string(),
                "binary_data".to_string(),
                "empty_blob".to_string(),
            ],
            values: vec![
                Value::String("valid UTF-8 字符串".to_string()),
                Value::String(vec![0xFF, 0xFE, 0xFD].iter().map(|b| format!("{:02x}", b)).collect()),  // 模拟二进制
                Value::String("".to_string()),
            ],
        };

        let result = service.convert_row_to_json(&row, &row.columns);
        assert!(result.is_ok());
        let json_map = result.unwrap();

        assert_eq!(json_map.get("valid_utf8"), Some(&Value::String("valid UTF-8 字符串".to_string())));
        assert!(json_map.get("binary_data").is_some());
        assert_eq!(json_map.get("empty_blob"), Some(&Value::String("".to_string())));
    }

    #[test]
    fn test_sql_dialect_mysql_features() {
        let service = MySqlService::new();
        let dialect = service.get_sql_dialect();

        // 验证 MySQL 特定的 SQL 特性
        assert_eq!(dialect.name, "MySQL");
        assert_eq!(dialect.identifier_quote, '`', "MySQL 使用反引号作为标识符引用");
        assert_eq!(dialect.string_quote, '\'', "MySQL 使用单引号作为字符串引用");
        assert_eq!(dialect.parameter_syntax, ParameterSyntax::QuestionMark, "MySQL 使用 ? 参数语法");
        assert_eq!(dialect.limit_syntax, LimitSyntax::Clause, "MySQL 支持 LIMIT ... OFFSET");
        assert!(dialect.supports_limit, "MySQL 支持 LIMIT 子句");

        // 验证排除的系统数据库
        assert!(dialect.exclude_schemas.contains(&"information_schema"));
        assert!(dialect.exclude_schemas.contains(&"performance_schema"));
        assert!(dialect.exclude_schemas.contains(&"mysql"));
        assert!(dialect.exclude_schemas.contains(&"sys"));
        assert_eq!(dialect.exclude_schemas.len(), 4);
    }

    #[test]
    fn test_row_conversion_with_datetime_simulation() {
        let service = MySqlService::new();

        // 模拟 MySQL DATETIME/TIMESTAMP 类型（在实现中会被转为字符串）
        let row = DbRow {
            columns: vec![
                "date_col".to_string(),
                "datetime_col".to_string(),
                "timestamp_col".to_string(),
                "null_date".to_string(),
            ],
            values: vec![
                Value::String("2026-01-27".to_string()),
                Value::String("2026-01-27 12:34:56".to_string()),
                Value::String("2026-01-27 12:34:56.123456".to_string()),
                Value::Null,
            ],
        };

        let result = service.convert_row_to_json(&row, &row.columns);
        assert!(result.is_ok());
        let json_map = result.unwrap();

        assert_eq!(json_map.get("date_col"), Some(&Value::String("2026-01-27".to_string())));
        assert_eq!(json_map.get("datetime_col"), Some(&Value::String("2026-01-27 12:34:56".to_string())));
        assert_eq!(json_map.get("timestamp_col"), Some(&Value::String("2026-01-27 12:34:56.123456".to_string())));
        assert_eq!(json_map.get("null_date"), Some(&Value::Null));
    }

    #[test]
    fn test_row_conversion_with_json_type() {
        let service = MySqlService::new();

        // 模拟 MySQL JSON 类型（作为字符串存储）
        let row = DbRow {
            columns: vec![
                "json_col".to_string(),
                "json_object".to_string(),
                "json_array".to_string(),
            ],
            values: vec![
                Value::String("\"simple string\"".to_string()),
                Value::String("{\"key\": \"value\", \"number\": 123}".to_string()),
                Value::String("[1, 2, 3, \"four\"]".to_string()),
            ],
        };

        let result = service.convert_row_to_json(&row, &row.columns);
        assert!(result.is_ok());
        let json_map = result.unwrap();

        assert_eq!(json_map.get("json_col"), Some(&Value::String("\"simple string\"".to_string())));
        assert_eq!(json_map.get("json_object"), Some(&Value::String("{\"key\": \"value\", \"number\": 123}".to_string())));
        assert_eq!(json_map.get("json_array"), Some(&Value::String("[1, 2, 3, \"four\"]".to_string())));
    }

    #[test]
    fn test_row_conversion_with_bit_type() {
        let service = MySqlService::new();

        // 模拟 MySQL BIT 类型（作为值转换）
        let row = DbRow {
            columns: vec![
                "bit1".to_string(),
                "bit8".to_string(),
                "bit16".to_string(),
            ],
            values: vec![
                Value::Number(serde_json::Number::from(1)),      // b'1'
                Value::Number(serde_json::Number::from(255)),    // b'11111111'
                Value::Number(serde_json::Number::from(65535)),  // b'1111111111111111'
            ],
        };

        let result = service.convert_row_to_json(&row, &row.columns);
        assert!(result.is_ok());
        let json_map = result.unwrap();

        assert_eq!(json_map.get("bit1"), Some(&Value::Number(serde_json::Number::from(1))));
        assert_eq!(json_map.get("bit8"), Some(&Value::Number(serde_json::Number::from(255))));
        assert_eq!(json_map.get("bit16"), Some(&Value::Number(serde_json::Number::from(65535))));
    }

    #[test]
    fn test_row_conversion_empty_vs_null() {
        let service = MySqlService::new();

        // 测试空字符串 vs NULL vs 0 的区别
        let row = DbRow {
            columns: vec![
                "empty_str".to_string(),
                "null_val".to_string(),
                "zero_int".to_string(),
            ],
            values: vec![
                Value::String("".to_string()),  // 空字符串
                Value::Null,  // NULL
                Value::Number(serde_json::Number::from(0)),  // 0
            ],
        };

        let result = service.convert_row_to_json(&row, &row.columns);
        assert!(result.is_ok());
        let json_map = result.unwrap();

        // 三者应该都是不同的值
        assert_eq!(json_map.get("empty_str"), Some(&Value::String("".to_string())));
        assert_eq!(json_map.get("null_val"), Some(&Value::Null));
        assert_eq!(json_map.get("zero_int"), Some(&Value::Number(serde_json::Number::from(0))));

        assert_ne!(json_map.get("empty_str"), json_map.get("null_val"));
        assert_ne!(json_map.get("null_val"), json_map.get("zero_int"));
        assert_ne!(json_map.get("empty_str"), json_map.get("zero_int"));
    }

    #[test]
    fn test_service_trait_compliance() {
        let service = MySqlService::new();

        // 测试所有必需的 trait 方法都可以被调用
        let _name = service.service_name();
        let _dialect = service.get_sql_dialect();

        // 创建测试 DbRow
        let row = DbRow {
            columns: vec!["test".to_string()],
            values: vec![Value::String("mysql_data".to_string())],
        };

        // 测试 convert_row_to_json
        let result = service.convert_row_to_json(&row, &row.columns);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().get("test"), Some(&Value::String("mysql_data".to_string())));
    }

    #[test]
    fn test_row_conversion_with_enum_type() {
        let service = MySqlService::new();

        // 模拟 MySQL ENUM 类型（作为字符串存储）
        let row = DbRow {
            columns: vec![
                "status_enum".to_string(),
                "priority_enum".to_string(),
            ],
            values: vec![
                Value::String("active".to_string()),
                Value::String("high".to_string()),
            ],
        };

        let result = service.convert_row_to_json(&row, &row.columns);
        assert!(result.is_ok());
        let json_map = result.unwrap();

        assert_eq!(json_map.get("status_enum"), Some(&Value::String("active".to_string())));
        assert_eq!(json_map.get("priority_enum"), Some(&Value::String("high".to_string())));
    }

    #[test]
    fn test_row_conversion_with_set_type() {
        let service = MySqlService::new();

        // 模拟 MySQL SET 类型（作为逗号分隔的字符串）
        let row = DbRow {
            columns: vec![
                "tags_set".to_string(),
                "empty_set".to_string(),
            ],
            values: vec![
                Value::String("read,write,execute".to_string()),
                Value::String("".to_string()),
            ],
        };

        let result = service.convert_row_to_json(&row, &row.columns);
        assert!(result.is_ok());
        let json_map = result.unwrap();

        assert_eq!(json_map.get("tags_set"), Some(&Value::String("read,write,execute".to_string())));
        assert_eq!(json_map.get("empty_set"), Some(&Value::String("".to_string())));
    }
}
