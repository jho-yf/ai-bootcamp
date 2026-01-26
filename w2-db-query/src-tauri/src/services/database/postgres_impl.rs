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

        // 复用现有的元数据提取逻辑
        let metadata = crate::services::postgres_service::extract_metadata(client, connection_id).await?;
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
