// PostgreSQL 服务实现
//
// 实现 DatabaseService trait for PostgreSQL

use super::r#trait::*;
use crate::models::metadata::{DatabaseMetadata};
use crate::utils::error::AppError;
use async_trait::async_trait;
use tokio_postgres::{NoTls, Row as PgRow};
use serde_json::Value;
use std::collections::HashMap;

/// PostgreSQL 服务实现
pub struct PostgresService;

impl PostgresService {
    pub fn new() -> Self {
        Self
    }

    /// 转换 PostgreSQL 行为通用 DbRow
    fn convert_pg_row_to_db_row(pg_row: &PgRow) -> DbRow {
        let columns: Vec<String> = pg_row
            .columns()
            .iter()
            .map(|col| col.name().to_string())
            .collect();

        let values = columns
            .iter()
            .enumerate()
            .map(|(i, _)| {
                let col_type = pg_row.columns().get(i);

                if col_type.is_none() {
                    return Value::Null;
                }

                let col_type_info = col_type.unwrap();
                let type_name = col_type_info.type_().name();
                let type_oid = col_type_info.type_().oid();

                // UUID 检测
                let is_uuid = type_name == "uuid" || type_oid == 2950;

                if is_uuid {
                    match pg_row.try_get::<_, Option<String>>(i) {
                        Ok(Some(uuid_val)) => Value::String(uuid_val),
                        Ok(None) => Value::Null,
                        Err(_) => Value::Null,
                    }
                } else {
                    match type_name {
                        "int4" | "int2" => pg_row
                            .try_get::<_, i32>(i)
                            .map(|v| Value::Number(v.into()))
                            .unwrap_or(Value::Null),

                        "int8" => pg_row
                            .try_get::<_, i64>(i)
                            .map(|v| Value::Number(v.into()))
                            .unwrap_or(Value::Null),

                        "float4" | "float8" => pg_row
                            .try_get::<_, f64>(i)
                            .ok()
                            .and_then(|v| serde_json::Number::from_f64(v).map(Value::Number))
                            .unwrap_or(Value::Null),

                        "bool" => pg_row
                            .try_get::<_, bool>(i)
                            .map(Value::Bool)
                            .unwrap_or(Value::Null),

                        "timestamp" | "timestamptz" | "date" | "json" | "jsonb" | _ => {
                            pg_row
                                .try_get::<_, String>(i)
                                .map(Value::String)
                                .unwrap_or(Value::Null)
                        }
                    }
                }
            })
            .collect();

        DbRow { columns, values }
    }
}

#[async_trait]
impl DatabaseService for PostgresService {
    fn service_name(&self) -> &'static str {
        "PostgreSQL"
    }

    async fn connect(
        &self,
        host: &str,
        port: u16,
        database_name: &str,
        user: &str,
        password: &str,
    ) -> Result<DbConnection, AppError> {
        let connection_string = format!(
            "host={} port={} dbname={} user={} password={}",
            host, port, database_name, user, password
        );

        let (client, connection) = tokio_postgres::connect(&connection_string, NoTls)
            .await
            .map_err(|e| AppError::DatabaseConnection(format!("连接失败: {}", e)))?;

        // 启动连接处理器
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("PostgreSQL 连接错误: {}", e);
            }
        });

        Ok(DbConnection::PostgreSQL(client))
    }

    async fn test_connection(
        &self,
        host: &str,
        port: u16,
        database_name: &str,
        user: &str,
        password: &str,
    ) -> Result<bool, AppError> {
        let client = match self.connect(host, port, database_name, user, password).await? {
            DbConnection::PostgreSQL(c) => c,
            _ => return Err(AppError::DatabaseConnection("Invalid connection type".to_string())),
        };

        client
            .query_one("SELECT 1", &[])
            .await
            .map_err(|e| AppError::DatabaseConnection(format!("测试查询失败: {}", e)))?;

        Ok(true)
    }

    async fn execute_query(
        &self,
        connection: &DbConnection,
        sql: &str,
        _params: &[QueryParam],
    ) -> Result<QueryExecutionResult, AppError> {
        let client = match connection {
            DbConnection::PostgreSQL(c) => c,
            _ => return Err(AppError::DatabaseConnection("Invalid connection type".to_string())),
        };

        let start_time = std::time::Instant::now();

        let rows = client
            .query(sql, &[])
            .await
            .map_err(|e| AppError::QueryExecution(format!("查询执行失败: {}", e)))?;

        let exec_time_ms = start_time.elapsed().as_millis() as u64;

        let db_rows = rows.iter().map(Self::convert_pg_row_to_db_row).collect();

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
        let client = match connection {
            DbConnection::PostgreSQL(c) => c,
            _ => return Err(AppError::DatabaseConnection("Invalid connection type".to_string())),
        };

        // 提取表
        let mut tables = extract_tables(client).await?;

        // 提取主键
        let primary_keys_map = extract_primary_keys(client, &tables).await?;

        // 提取外键
        let foreign_keys_map = extract_foreign_keys(client, &tables).await?;

        // 组装表信息
        let mut table_column_map: std::collections::HashMap<String, Vec<crate::models::metadata::ColumnInfo>> =
            std::collections::HashMap::new();

        // 为每个表查询其列
        for table in &tables {
            let table_key = format!("{}.{}", table.schema, table.name);
            let table_columns = client
                .query(
                    "SELECT column_name, data_type, is_nullable, column_default, ordinal_position
                     FROM information_schema.columns
                     WHERE table_schema = $1 AND table_name = $2
                     ORDER BY ordinal_position",
                    &[&table.schema, &table.name],
                )
                .await
                .map_err(|e| AppError::MetadataExtraction(format!("查询表列失败: {}", e)))?;

            let cols: Vec<crate::models::metadata::ColumnInfo> = table_columns
                .into_iter()
                .map(|row| crate::models::metadata::ColumnInfo {
                    name: row.get("column_name"),
                    data_type: row.get("data_type"),
                    nullable: matches!(row.get::<_, String>("is_nullable").as_str(), "YES"),
                    default_value: row.get::<_, Option<String>>("column_default"),
                    is_primary_key: false,
                    ordinal_position: row.get::<_, i32>("ordinal_position"),
                })
                .collect();

            table_column_map.insert(table_key, cols);
        }

        for table in &mut tables {
            let table_key = format!("{}.{}", table.schema, table.name);

            // 填充列
            if let Some(cols) = table_column_map.get(&table_key) {
                table.columns = cols.clone();
            }

            // 填充主键
            if let Some(pk) = primary_keys_map.iter().find(|(k, _)| **k == table_key) {
                table.primary_keys = pk.1.clone();
                for col in &mut table.columns {
                    col.is_primary_key = table.primary_keys.contains(&col.name);
                }
            }

            // 填充外键
            if let Some(fk) = foreign_keys_map.iter().find(|(k, _)| **k == table_key) {
                table.foreign_keys = fk.1.clone();
            }
        }

        // 提取视图
        let views = extract_views(client).await?;

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
        &POSTGRES_DIALECT
    }
}

/// PostgreSQL SQL 方言
static POSTGRES_DIALECT: SqlDialect = SqlDialect {
    name: "PostgreSQL",
    string_quote: '\'',
    identifier_quote: '"',
    supports_limit: true,
    limit_syntax: LimitSyntax::Clause,
    parameter_syntax: ParameterSyntax::DollarNumeric,
    exclude_schemas: &["information_schema", "pg_catalog", "pg_toast"],
};

// 辅助函数：提取所有表
async fn extract_tables(client: &tokio_postgres::Client) -> Result<Vec<crate::models::metadata::TableInfo>, AppError> {
    let rows = client
        .query(
            "SELECT table_schema, table_name, table_type
             FROM information_schema.tables
             WHERE table_schema NOT IN ('information_schema', 'pg_catalog', 'pg_toast')
             AND table_type IN ('BASE TABLE', 'VIEW')
             ORDER BY table_schema, table_name",
            &[],
        )
        .await
        .map_err(|e| AppError::QueryExecution(format!("查询表失败: {}", e)))?;

    let mut tables = Vec::new();
    for row in rows {
        let schema: String = row.get("table_schema");
        let name: String = row.get("table_name");
        let table_type: String = row.get("table_type");

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

// 辅助函数：提取主键信息
async fn extract_primary_keys(
    client: &tokio_postgres::Client,
    tables: &[crate::models::metadata::TableInfo],
) -> Result<Vec<(String, Vec<String>)>, AppError> {
    let mut result = Vec::new();

    for table in tables {
        let rows = client
            .query(
                "SELECT kcu.column_name
                 FROM information_schema.table_constraints tc
                 JOIN information_schema.key_column_usage kcu
                   ON tc.constraint_name = kcu.constraint_name
                   AND tc.table_schema = kcu.table_schema
                 WHERE tc.constraint_type = 'PRIMARY KEY'
                   AND tc.table_schema = $1
                   AND tc.table_name = $2
                 ORDER BY kcu.ordinal_position",
                &[&table.schema, &table.name],
            )
            .await
            .map_err(|e| AppError::QueryExecution(format!("查询主键失败: {}", e)))?;

        let primary_keys: Vec<String> = rows
            .into_iter()
            .map(|row| row.get("column_name"))
            .collect();

        result.push((format!("{}.{}", table.schema, table.name), primary_keys));
    }

    Ok(result)
}

// 辅助函数：提取外键信息
async fn extract_foreign_keys(
    client: &tokio_postgres::Client,
    tables: &[crate::models::metadata::TableInfo],
) -> Result<Vec<(String, Vec<crate::models::metadata::ForeignKeyInfo>)>, AppError> {
    let mut result = Vec::new();

    for table in tables {
        let rows = client
            .query(
                "SELECT
                        tc.constraint_name,
                        kcu.column_name,
                        ccu.table_name AS referenced_table,
                        ccu.column_name AS referenced_column
                     FROM information_schema.table_constraints tc
                     JOIN information_schema.key_column_usage kcu
                       ON tc.constraint_name = kcu.constraint_name
                       AND tc.table_schema = kcu.table_schema
                     JOIN information_schema.constraint_column_usage ccu
                       ON ccu.constraint_name = tc.constraint_name
                       AND ccu.table_schema = tc.table_schema
                     WHERE tc.constraint_type = 'FOREIGN KEY'
                       AND tc.table_schema = $1
                       AND tc.table_name = $2
                     ORDER BY tc.constraint_name, kcu.ordinal_position",
                &[&table.schema, &table.name],
            )
            .await
            .map_err(|e| AppError::QueryExecution(format!("查询外键失败: {}", e)))?;

        let foreign_keys: Vec<crate::models::metadata::ForeignKeyInfo> = rows
            .into_iter()
            .map(|row| crate::models::metadata::ForeignKeyInfo {
                constraint_name: row.get("constraint_name"),
                column_name: row.get("column_name"),
                referenced_table: row.get("referenced_table"),
                referenced_column: row.get("referenced_column"),
            })
            .collect();

        result.push((format!("{}.{}", table.schema, table.name), foreign_keys));
    }

    Ok(result)
}

// 辅助函数：提取视图信息
async fn extract_views(client: &tokio_postgres::Client) -> Result<Vec<crate::models::metadata::ViewInfo>, AppError> {
    let rows = client
        .query(
            "SELECT table_schema, table_name, view_definition
             FROM information_schema.views
             WHERE table_schema NOT IN ('information_schema', 'pg_catalog', 'pg_toast')
             ORDER BY table_schema, table_name",
            &[],
        )
        .await
        .map_err(|e| AppError::QueryExecution(format!("查询视图失败: {}", e)))?;

    let mut views = Vec::new();
    for row in rows {
        views.push(crate::models::metadata::ViewInfo {
            schema: row.get("table_schema"),
            name: row.get("table_name"),
            columns: vec![],
            definition: row.get("view_definition"),
        });
    }

    Ok(views)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_properties() {
        let service = PostgresService::new();

        // 验证服务名称
        assert_eq!(service.service_name(), "PostgreSQL");

        // 验证 SQL 方言配置
        let dialect = service.get_sql_dialect();
        assert_eq!(dialect.name, "PostgreSQL");
        assert_eq!(dialect.string_quote, '\'');
        assert_eq!(dialect.identifier_quote, '"');
        assert!(dialect.supports_limit);
        assert_eq!(dialect.limit_syntax, LimitSyntax::Clause);
        assert_eq!(dialect.parameter_syntax, ParameterSyntax::DollarNumeric);

        // 验证排除的模式列表
        assert!(dialect.exclude_schemas.contains(&"information_schema"));
        assert!(dialect.exclude_schemas.contains(&"pg_catalog"));
        assert!(dialect.exclude_schemas.contains(&"pg_toast"));
    }

    #[test]
    fn test_service_creation() {
        // 测试服务创建
        let service1 = PostgresService::new();
        let service2 = PostgresService::new();

        // 验证服务属性
        assert_eq!(service1.service_name(), service2.service_name());

        // 验证方言一致性
        let dialect1 = service1.get_sql_dialect();
        let dialect2 = service2.get_sql_dialect();
        assert_eq!(dialect1.name, dialect2.name);
    }

    #[test]
    fn test_row_conversion() {
        let service = PostgresService::new();

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
                Value::Number(serde_json::Number::from(42)),
                Value::String("test".to_string())
            ],
        };

        let result = service.convert_row_to_json(&row, &row.columns);
        assert!(result.is_ok());
        let json_map = result.unwrap();
        assert_eq!(json_map.len(), 2);
        assert_eq!(json_map.get("id"), Some(&Value::Number(serde_json::Number::from(42))));
        assert_eq!(json_map.get("name"), Some(&Value::String("test".to_string())));
    }

    #[test]
    fn test_row_conversion_with_nulls() {
        let service = PostgresService::new();

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
    fn test_row_conversion_with_mixed_types() {
        let service = PostgresService::new();

        // 测试混合数据类型（模拟 PostgreSQL 的类型转换）
        let row = DbRow {
            columns: vec![
                "int_col".to_string(),
                "bigint_col".to_string(),
                "text_col".to_string(),
                "bool_col".to_string(),
                "float_col".to_string(),
                "null_col".to_string(),
            ],
            values: vec![
                Value::Number(serde_json::Number::from(2147483647)),  // i32::MAX
                Value::Number(serde_json::Number::from(9223372036854775807_i64)),  // i64::MAX
                Value::String("PostgreSQL text".to_string()),
                Value::Bool(true),
                Value::Number(serde_json::Number::from_f64(3.14159265359).unwrap()),
                Value::Null,
            ],
        };

        let result = service.convert_row_to_json(&row, &row.columns);
        assert!(result.is_ok());
        let json_map = result.unwrap();

        // 验证整数
        assert_eq!(json_map.get("int_col"), Some(&Value::Number(serde_json::Number::from(2147483647))));
        assert_eq!(json_map.get("bigint_col"), Some(&Value::Number(serde_json::Number::from(9223372036854775807_i64))));

        // 验证文本
        assert_eq!(json_map.get("text_col"), Some(&Value::String("PostgreSQL text".to_string())));

        // 验证布尔值
        assert_eq!(json_map.get("bool_col"), Some(&Value::Bool(true)));

        // 验证浮点数
        assert_eq!(json_map.get("float_col"), Some(&Value::Number(serde_json::Number::from_f64(3.14159265359).unwrap())));

        // 验证 NULL
        assert_eq!(json_map.get("null_col"), Some(&Value::Null));

        assert_eq!(json_map.len(), 6);
    }

    #[test]
    fn test_row_conversion_with_boolean_values() {
        let service = PostgresService::new();

        // 测试 true 和 false
        let row_true = DbRow {
            columns: vec!["active".to_string()],
            values: vec![Value::Bool(true)],
        };

        let result_true = service.convert_row_to_json(&row_true, &row_true.columns);
        assert!(result_true.is_ok());
        assert_eq!(result_true.unwrap().get("active"), Some(&Value::Bool(true)));

        // 测试 false
        let row_false = DbRow {
            columns: vec!["active".to_string()],
            values: vec![Value::Bool(false)],
        };

        let result_false = service.convert_row_to_json(&row_false, &row_false.columns);
        assert!(result_false.is_ok());
        assert_eq!(result_false.unwrap().get("active"), Some(&Value::Bool(false)));
    }

    #[test]
    fn test_row_conversion_with_numeric_precision() {
        let service = PostgresService::new();

        // 测试各种数值精度
        let row = DbRow {
            columns: vec![
                "tiny_int".to_string(),
                "small_int".to_string(),
                "normal_int".to_string(),
                "negative_int".to_string(),
                "zero".to_string(),
            ],
            values: vec![
                Value::Number(serde_json::Number::from(1)),
                Value::Number(serde_json::Number::from(1000)),
                Value::Number(serde_json::Number::from(1000000)),
                Value::Number(serde_json::Number::from(-999999)),
                Value::Number(serde_json::Number::from(0)),
            ],
        };

        let result = service.convert_row_to_json(&row, &row.columns);
        assert!(result.is_ok());
        let json_map = result.unwrap();

        assert_eq!(json_map.get("tiny_int"), Some(&Value::Number(serde_json::Number::from(1))));
        assert_eq!(json_map.get("small_int"), Some(&Value::Number(serde_json::Number::from(1000))));
        assert_eq!(json_map.get("normal_int"), Some(&Value::Number(serde_json::Number::from(1000000))));
        assert_eq!(json_map.get("negative_int"), Some(&Value::Number(serde_json::Number::from(-999999))));
        assert_eq!(json_map.get("zero"), Some(&Value::Number(serde_json::Number::from(0))));
    }

    #[test]
    fn test_row_conversion_with_special_characters() {
        let service = PostgresService::new();

        // 测试包含特殊字符的字符串
        let row = DbRow {
            columns: vec![
                "unicode".to_string(),
                "newlines".to_string(),
                "quotes".to_string(),
                "empty".to_string(),
            ],
            values: vec![
                Value::String("中文日本語한국어".to_string()),
                Value::String("line1\nline2\rline3".to_string()),
                Value::String("contains \"double\" and 'single' quotes".to_string()),
                Value::String("".to_string()),
            ],
        };

        let result = service.convert_row_to_json(&row, &row.columns);
        assert!(result.is_ok());
        let json_map = result.unwrap();

        assert_eq!(json_map.get("unicode"), Some(&Value::String("中文日本語한국어".to_string())));
        assert_eq!(json_map.get("newlines"), Some(&Value::String("line1\nline2\rline3".to_string())));
        assert_eq!(json_map.get("quotes"), Some(&Value::String("contains \"double\" and 'single' quotes".to_string())));
        assert_eq!(json_map.get("empty"), Some(&Value::String("".to_string())));
    }

    #[test]
    fn test_sql_dialect_postgresql_features() {
        let service = PostgresService::new();
        let dialect = service.get_sql_dialect();

        // 验证 PostgreSQL 特定的 SQL 特性
        assert_eq!(dialect.name, "PostgreSQL");
        assert_eq!(dialect.identifier_quote, '"', "PostgreSQL 使用双引号作为标识符引用");
        assert_eq!(dialect.string_quote, '\'', "PostgreSQL 使用单引号作为字符串引用");
        assert_eq!(dialect.parameter_syntax, ParameterSyntax::DollarNumeric, "PostgreSQL 使用 $1, $2 参数语法");
        assert_eq!(dialect.limit_syntax, LimitSyntax::Clause, "PostgreSQL 支持 LIMIT ... OFFSET");
        assert!(dialect.supports_limit, "PostgreSQL 支持 LIMIT 子句");

        // 验证排除的系统模式
        assert!(dialect.exclude_schemas.contains(&"information_schema"));
        assert!(dialect.exclude_schemas.contains(&"pg_catalog"));
        assert!(dialect.exclude_schemas.contains(&"pg_toast"));
        assert_eq!(dialect.exclude_schemas.len(), 3);
    }

    #[test]
    fn test_row_conversion_empty_vs_null() {
        let service = PostgresService::new();

        // 测试空字符串 vs NULL
        let row = DbRow {
            columns: vec!["empty_str".to_string(), "null_val".to_string()],
            values: vec![
                Value::String("".to_string()),  // 空字符串
                Value::Null,  // NULL
            ],
        };

        let result = service.convert_row_to_json(&row, &row.columns);
        assert!(result.is_ok());
        let json_map = result.unwrap();

        // 空字符串和 NULL 应该是不同的
        assert_eq!(json_map.get("empty_str"), Some(&Value::String("".to_string())));
        assert_eq!(json_map.get("null_val"), Some(&Value::Null));
        assert_ne!(json_map.get("empty_str"), json_map.get("null_val"));
    }

    #[test]
    fn test_row_conversion_large_dataset() {
        let service = PostgresService::new();

        // 测试多列数据集
        let columns: Vec<String> = (0..10).map(|i| format!("col_{}", i)).collect();
        let values: Vec<Value> = (0..10)
            .map(|i| match i % 4 {
                0 => Value::Number(serde_json::Number::from(i)),
                1 => Value::String(format!("value_{}", i)),
                2 => Value::Bool(i % 2 == 0),
                _ => Value::Null,
            })
            .collect();

        let row = DbRow { columns: columns.clone(), values };
        let result = service.convert_row_to_json(&row, &columns);

        assert!(result.is_ok());
        let json_map = result.unwrap();
        assert_eq!(json_map.len(), 10);

        // 验证部分数据
        assert_eq!(json_map.get("col_0"), Some(&Value::Number(serde_json::Number::from(0))));
        assert_eq!(json_map.get("col_1"), Some(&Value::String("value_1".to_string())));
    }

    #[test]
    fn test_service_trait_compliance() {
        let service = PostgresService::new();

        // 测试所有必需的 trait 方法都可以被调用
        let _name = service.service_name();
        let _dialect = service.get_sql_dialect();

        // 创建测试 DbRow
        let row = DbRow {
            columns: vec!["test".to_string()],
            values: vec![Value::String("data".to_string())],
        };

        // 测试 convert_row_to_json
        let result = service.convert_row_to_json(&row, &row.columns);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().get("test"), Some(&Value::String("data".to_string())));
    }
}
