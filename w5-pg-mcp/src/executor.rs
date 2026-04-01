use serde::Serialize;
use serde_json::{json, Value};
use sqlx::postgres::PgPool;
use sqlx::{Column, Row, TypeInfo as _};
use std::time::Instant;
use thiserror::Error;
use tracing::{debug, info};

#[derive(Debug, Clone, Serialize)]
pub struct QueryResult {
    pub sql: String,
    pub columns: Vec<String>,
    pub rows: Vec<Value>,
    pub row_count: usize,
    pub truncated: bool,
    pub execution_time_ms: u64,
    pub error: Option<String>,
}

#[derive(Debug, Error)]
pub enum ExecutorError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Query timeout exceeded {0} seconds")]
    Timeout(u64),

    #[error("Failed to serialize row: {0}")]
    SerializationError(String),
}

#[derive(Debug)]
pub struct QueryExecutor {
    pool: PgPool,
    max_rows: u32,
    query_timeout_secs: u64,
    debug: bool,
}

impl QueryExecutor {
    pub fn new(pool: PgPool, max_rows: u32, query_timeout_secs: u64, debug: bool) -> Self {
        Self {
            pool,
            max_rows,
            query_timeout_secs,
            debug,
        }
    }

    pub async fn execute(&self, sql: &str) -> Result<QueryResult, ExecutorError> {
        let start = Instant::now();
        info!(sql = %sql, "Executing query");

        // Strip trailing semicolons to avoid syntax errors when wrapping in subquery
        let trimmed = sql.trim().trim_end_matches(';');

        // Check if SQL already has LIMIT clause
        let final_sql = if self.has_limit_clause(trimmed) {
            trimmed.to_string()
        } else {
            format!("SELECT * FROM ({}) AS _subq LIMIT {}", trimmed, self.max_rows)
        };

        debug!(final_sql = %final_sql, "Final SQL with LIMIT protection");

        let mut tx = self.pool.begin().await?;

        // Set transaction to read-only
        sqlx::query("SET TRANSACTION READ ONLY")
            .execute(&mut *tx)
            .await?;

        // Set statement timeout
        let timeout_ms = self.query_timeout_secs * 1000;
        sqlx::query(&format!("SET LOCAL statement_timeout = {}", timeout_ms))
            .execute(&mut *tx)
            .await?;

        // Execute query
        let result = sqlx::query(&final_sql).fetch_all(&mut *tx).await;

        // Commit transaction (even for read-only)
        tx.commit().await?;

        let execution_time_ms = start.elapsed().as_millis() as u64;

        match result {
            Ok(rows) => {
                let columns = if let Some(first_row) = rows.first() {
                    first_row
                        .columns()
                        .iter()
                        .map(|c| c.name().to_string())
                        .collect()
                } else {
                    vec![]
                };

                let serialized_rows: Vec<Value> = rows
                    .iter()
                    .map(|row| self.serialize_row(row, &columns))
                    .collect::<Result<Vec<_>, _>>()?;

                let row_count = serialized_rows.len();
                let truncated = row_count >= self.max_rows as usize;

                info!(
                    row_count = row_count,
                    execution_time_ms = execution_time_ms,
                    truncated = truncated,
                    "Query completed"
                );

                Ok(QueryResult {
                    sql: final_sql,
                    columns,
                    rows: serialized_rows,
                    row_count,
                    truncated,
                    execution_time_ms,
                    error: None,
                })
            }
            Err(e) => {
                let error_msg = if self.debug {
                    e.to_string()
                } else {
                    "Database query error (enable debug mode for details)".to_string()
                };

                info!(
                    error = %error_msg,
                    execution_time_ms = execution_time_ms,
                    "Query failed"
                );

                Ok(QueryResult {
                    sql: final_sql,
                    columns: vec![],
                    rows: vec![],
                    row_count: 0,
                    truncated: false,
                    execution_time_ms,
                    error: Some(error_msg),
                })
            }
        }
    }

    fn has_limit_clause(&self, sql: &str) -> bool {
        use sqlparser::dialect::PostgreSqlDialect;
        use sqlparser::parser::Parser;

        let dialect = PostgreSqlDialect {};

        if let Ok(statements) = Parser::parse_sql(&dialect, sql) {
            if let Some(sqlparser::ast::Statement::Query(query)) = statements.first() {
                // Check if the outermost query has a LIMIT clause
                return query.limit_clause.is_some();
            }
        }

        false
    }

    fn serialize_row(&self, row: &sqlx::postgres::PgRow, columns: &[String]) -> Result<Value, ExecutorError> {
        let mut map = serde_json::Map::new();

        for (i, column_name) in columns.iter().enumerate() {
            let value = self.get_column_value(row, i)?;
            map.insert(column_name.clone(), value);
        }

        Ok(Value::Object(map))
    }

    fn get_column_value(&self, row: &sqlx::postgres::PgRow, i: usize) -> Result<Value, ExecutorError> {
        let type_name = row.columns()[i].type_info().name().to_lowercase();

        match type_name.as_str() {
            "int2" => {
                match row.try_get::<Option<i16>, _>(i) {
                    Ok(Some(v)) => Ok(json!(v as i64)),
                    Ok(None) => Ok(Value::Null),
                    Err(_) => Ok(Value::Null),
                }
            }
            "int4" => {
                match row.try_get::<Option<i32>, _>(i) {
                    Ok(Some(v)) => Ok(json!(v as i64)),
                    Ok(None) => Ok(Value::Null),
                    Err(_) => Ok(Value::Null),
                }
            }
            "int8" => {
                match row.try_get::<Option<i64>, _>(i) {
                    Ok(Some(v)) => Ok(json!(v)),
                    Ok(None) => Ok(Value::Null),
                    Err(_) => Ok(Value::Null),
                }
            }
            "float4" => {
                match row.try_get::<Option<f32>, _>(i) {
                    Ok(Some(v)) => Ok(json!(v as f64)),
                    Ok(None) => Ok(Value::Null),
                    Err(_) => Ok(Value::Null),
                }
            }
            "float8" => {
                match row.try_get::<Option<f64>, _>(i) {
                    Ok(Some(v)) => Ok(json!(v)),
                    Ok(None) => Ok(Value::Null),
                    Err(_) => Ok(Value::Null),
                }
            }
            "bool" => {
                match row.try_get::<Option<bool>, _>(i) {
                    Ok(Some(v)) => Ok(json!(v)),
                    Ok(None) => Ok(Value::Null),
                    Err(_) => Ok(Value::Null),
                }
            }
            "json" | "jsonb" => {
                match row.try_get::<Option<Value>, _>(i) {
                    Ok(Some(v)) => Ok(v),
                    Ok(None) => Ok(Value::Null),
                    Err(_) => Ok(Value::Null),
                }
            }
            "numeric" => {
                match row.try_get::<Option<rust_decimal::Decimal>, _>(i) {
                    Ok(Some(v)) => Ok(json!(v.to_string())),
                    Ok(None) => Ok(Value::Null),
                    Err(_) => {
                        // Fallback to string
                        match row.try_get::<Option<String>, _>(i) {
                            Ok(Some(v)) => Ok(json!(v)),
                            Ok(None) => Ok(Value::Null),
                            Err(_) => Ok(Value::Null),
                        }
                    }
                }
            }
            _ => {
                // Fallback to string representation
                match row.try_get::<Option<String>, _>(i) {
                    Ok(Some(v)) => Ok(json!(v)),
                    Ok(None) => Ok(Value::Null),
                    Err(_) => Ok(Value::Null),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check_has_limit(sql: &str) -> bool {
        use sqlparser::dialect::PostgreSqlDialect;
        use sqlparser::parser::Parser;

        let dialect = PostgreSqlDialect {};
        if let Ok(statements) = Parser::parse_sql(&dialect, sql) {
            if let Some(sqlparser::ast::Statement::Query(query)) = statements.first() {
                return query.limit_clause.is_some();
            }
        }
        false
    }

    #[test]
    fn test_has_limit_clause_with_limit() {
        assert!(check_has_limit("SELECT * FROM users LIMIT 10"));
        assert!(check_has_limit("SELECT * FROM users LIMIT 10 OFFSET 20"));
    }

    #[test]
    fn test_has_limit_clause_without_limit() {
        assert!(!check_has_limit("SELECT * FROM users"));
        assert!(!check_has_limit("SELECT * FROM users WHERE id = 1"));
    }

    #[test]
    fn test_has_limit_clause_with_subquery() {
        // Outer query has limit
        assert!(check_has_limit("SELECT * FROM (SELECT * FROM users) AS u LIMIT 5"));

        // No limit on outer query
        assert!(!check_has_limit("SELECT * FROM (SELECT * FROM users LIMIT 5) AS u"));
    }

    #[test]
    fn test_query_result_serialization() {
        let result = QueryResult {
            sql: "SELECT * FROM users".to_string(),
            columns: vec!["id".to_string(), "name".to_string()],
            rows: vec![json!({"id": 1, "name": "Alice"})],
            row_count: 1,
            truncated: false,
            execution_time_ms: 10,
            error: None,
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"sql\":\"SELECT * FROM users\""));
        assert!(json.contains("\"row_count\":1"));
        assert!(json.contains("\"execution_time_ms\":10"));
    }

    #[test]
    fn test_query_result_with_error() {
        let result = QueryResult {
            sql: "SELECT * FROM non_existent".to_string(),
            columns: vec![],
            rows: vec![],
            row_count: 0,
            truncated: false,
            execution_time_ms: 5,
            error: Some("Table not found".to_string()),
        };

        assert!(result.error.is_some());
        assert_eq!(result.row_count, 0);
    }
}
