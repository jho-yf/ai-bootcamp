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
    fn test_service_name() {
        let service = PostgresService::new();
        assert_eq!(service.service_name(), "PostgreSQL");
    }

    #[test]
    fn test_sql_dialect() {
        let service = PostgresService::new();
        let dialect = service.get_sql_dialect();

        assert_eq!(dialect.name, "PostgreSQL");
        assert_eq!(dialect.string_quote, '\'');
        assert_eq!(dialect.identifier_quote, '"');
        assert_eq!(dialect.supports_limit, true);
        assert_eq!(dialect.limit_syntax, LimitSyntax::Clause);
        assert_eq!(dialect.parameter_syntax, ParameterSyntax::DollarNumeric);
    }
}
