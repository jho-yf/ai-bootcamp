// PostgreSQL 服务实现
//
// 实现 DatabaseService trait for PostgreSQL
//
// 文件位置: src-tauri/src/services/database/postgres_impl.rs

use super::trait::*;
use crate::models::metadata::{
    DatabaseMetadata, TableInfo, ViewInfo, ColumnInfo, ForeignKeyInfo
};
use crate::utils::error::AppError;
use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use tokio_postgres::{Client, NoTls, Row as PgRow};
use std::collections::HashMap;

/// PostgreSQL 服务实现
pub struct PostgresService;

impl PostgresService {
    pub fn new() -> Self {
        Self
    }

    /// 转换 PostgreSQL 行为通用 DbRow
    ///
    /// # 类型转换规则
    /// - UUID → String (标准格式)
    /// - int4/int8 → Number
    /// - float4/float8 → Number
    /// - bool → Bool
    /// - timestamp/timestamptz → String (ISO 8601)
    /// - date → String (YYYY-MM-DD)
    /// - json/jsonb → JSON 对象
    /// - 其他 → String
    fn convert_pg_row_to_db_row(pg_row: &PgRow) -> DbRow {
        let columns = pg_row
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
                    match pg_row.try_get::<_, Option<uuid::Uuid>>(i) {
                        Ok(Some(uuid_val)) => Value::String(uuid_val.to_string()),
                        Ok(None) => Value::Null,
                        Err(_) => pg_row
                            .try_get::<_, Option<String>>(i)
                            .ok()
                            .and_then(|opt| opt.map(Value::String))
                            .unwrap_or(Value::Null),
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

                        "timestamp" => match pg_row.try_get::<_, Option<NaiveDateTime>>(i) {
                            Ok(Some(dt)) => Value::String(dt.format("%Y-%m-%d %H:%M:%S%.f").to_string()),
                            Ok(None) => Value::Null,
                            Err(_) => pg_row
                                .try_get::<_, NaiveDateTime>(i)
                                .map(|dt| Value::String(dt.format("%Y-%m-%d %H:%M:%S%.f").to_string()))
                                .unwrap_or(Value::Null),
                        },

                        "timestamptz" => {
                            if let Ok(Some(dt)) = pg_row.try_get::<_, Option<DateTime<Utc>>>(i) {
                                Value::String(dt.format("%Y-%m-%d %H:%M:%S%.f UTC").to_string())
                            } else if let Ok(dt) = pg_row.try_get::<_, DateTime<Utc>>(i) {
                                Value::String(dt.format("%Y-%m-%d %H:%M:%S%.f UTC").to_string())
                            } else {
                                Value::Null
                            }
                        }

                        "date" => match pg_row.try_get::<_, Option<NaiveDate>>(i) {
                            Ok(Some(date)) => Value::String(date.format("%Y-%m-%d").to_string()),
                            Ok(None) => Value::Null,
                            Err(_) => pg_row
                                .try_get::<_, NaiveDate>(i)
                                .map(|date| Value::String(date.format("%Y-%m-%d").to_string()))
                                .unwrap_or(Value::Null),
                        },

                        "json" | "jsonb" => match pg_row.try_get::<_, String>(i) {
                            Ok(json_str) => {
                                serde_json::from_str(&json_str).unwrap_or(Value::String(json_str))
                            }
                            Err(_) => Value::Null,
                        },

                        _ => pg_row
                            .try_get::<_, String>(i)
                            .map(Value::String)
                            .unwrap_or(Value::Null),
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
        match self.connect(host, port, database_name, user, password).await? {
            DbConnection::PostgreSQL(client) => {
                client
                    .query_one("SELECT 1", &[])
                    .await
                    .map_err(|e| AppError::DatabaseConnection(format!("测试查询失败: {}", e)))?;
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

        // 提取表信息
        let tables = self.extract_tables(client).await?;
        let views = self.extract_views(client).await?;

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
        &POSTGRES_DIALECT
    }
}

impl PostgresService {
    /// 提取表信息
    async fn extract_tables(&self, client: &Client) -> Result<Vec<TableInfo>, AppError> {
        let query = r#"
            SELECT
                t.table_schema,
                t.table_name,
                obj_description((t.table_schema||'.'||t.table_name)::regclass) as table_comment
            FROM information_schema.tables t
            WHERE t.table_type = 'BASE TABLE'
                AND t.table_schema NOT IN ('information_schema', 'pg_catalog', 'pg_toast')
            ORDER BY t.table_schema, t.table_name
        "#;

        let rows = client
            .query(query, &[])
            .await
            .map_err(|e| AppError::MetadataExtraction(format!("提取表失败: {}", e)))?;

        let mut tables = Vec::new();
        for row in rows {
            let schema: String = row.get("table_schema");
            let name: String = row.get("table_name");
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
    async fn extract_views(&self, client: &Client) -> Result<Vec<ViewInfo>, AppError> {
        let query = r#"
            SELECT
                v.table_schema,
                v.table_name,
                obj_description((v.table_schema||'.'||v.table_name)::regclass) as view_comment
            FROM information_schema.views v
            WHERE v.table_schema NOT IN ('information_schema', 'pg_catalog')
            ORDER BY v.table_schema, v.table_name
        "#;

        let rows = client
            .query(query, &[])
            .await
            .map_err(|e| AppError::MetadataExtraction(format!("提取视图失败: {}", e)))?;

        let mut views = Vec::new();
        for row in rows {
            let schema: String = row.get("table_schema");
            let name: String = row.get("table_name");
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

/// PostgreSQL SQL 方言
const POSTGRES_DIALECT: SqlDialect = SqlDialect {
    name: "PostgreSQL",
    string_quote: '\'',
    identifier_quote: '"',
    supports_limit: true,
    limit_syntax: LimitSyntax::Clause,
    parameter_syntax: ParameterSyntax::DollarNumeric,
    exclude_schemas: vec!["information_schema", "pg_catalog", "pg_toast"],
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
