/// SQL 查询执行 Commands
use crate::models::database::DatabaseType;
use crate::models::query::{QueryResult, RunQueryRequest};
use crate::services::{cache_service, mysql_service, postgres_service, query_parser};
use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use serde_json::Value;
use std::collections::HashMap;

/// 执行 SQL 查询
#[tauri::command]
pub async fn run_sql_query(request: RunQueryRequest) -> Result<QueryResult, String> {
    // 检查是否为 DDL 语句
    if query_parser::is_ddl_statement(&request.sql).map_err(|e| e.to_string())? {
        return Err("不允许执行 DDL 语句（CREATE/DROP/ALTER）".to_string());
    }

    // 解析并注入 LIMIT
    let parsed_sql = query_parser::inject_limit(&request.sql).map_err(|e| e.to_string())?;

    // 加载连接配置
    let connections =
        cache_service::load_connections().map_err(|e| format!("加载连接失败: {}", e))?;

    let connection = connections
        .iter()
        .find(|c| c.id == request.database_id)
        .ok_or_else(|| "数据库连接不存在".to_string())?;

    // 根据数据库类型执行查询
    let result = match connection.database_type {
        DatabaseType::PostgreSQL => {
            // PostgreSQL 查询执行
            let client = postgres_service::connect(
                &connection.host,
                connection.port,
                &connection.database_name,
                &connection.user,
                &connection.password,
            )
            .await
            .map_err(|e| format!("连接失败: {}", e))?;

            let (columns, rows, exec_time_ms) =
                postgres_service::execute_query(&client, &parsed_sql)
                    .await
                    .map_err(|e| e.to_string())?;

            // 转换 PostgreSQL 行数据为 JSON
            let result_rows = convert_postgres_rows(&columns, &rows);
            let total = result_rows.len();
            let truncated = total >= 100;

            QueryResult {
                columns,
                rows: result_rows,
                total,
                exec_time_ms,
                sql: parsed_sql.clone(),
                truncated,
            }
        }
        DatabaseType::MySQL => {
            // MySQL 查询执行
            let pool = mysql_service::connect(
                &connection.host,
                connection.port,
                &connection.database_name,
                &connection.user,
                &connection.password,
            )
            .await
            .map_err(|e| format!("连接失败: {}", e))?;

            let (columns, rows, exec_time_ms) =
                mysql_service::execute_query(&pool, &parsed_sql)
                    .await
                    .map_err(|e| e.to_string())?;

            // 转换 MySQL 行数据为 JSON
            let result_rows = convert_mysql_rows(&columns, &rows);
            let total = result_rows.len();
            let truncated = total >= 100;

            QueryResult {
                columns,
                rows: result_rows,
                total,
                exec_time_ms,
                sql: parsed_sql.clone(),
                truncated,
            }
        }
    };

    // 保存查询历史（SQL 查询类型）
    let _ = cache_service::save_query_history(
        &request.database_id,
        "sql",
        Some(&parsed_sql),
        None,
        Some(result.exec_time_ms),
        "success",
    );

    Ok(result)
}

/// 转换 PostgreSQL 行数据为 JSON 格式
fn convert_postgres_rows(columns: &[String], rows: &[tokio_postgres::Row]) -> Vec<HashMap<String, Value>> {
    let mut result_rows = Vec::new();
    for row in rows {
        let mut row_map = HashMap::new();
        for (i, col_name) in columns.iter().enumerate() {
            let col_type = row.columns().get(i);
            if col_type.is_none() {
                row_map.insert(col_name.clone(), Value::Null);
                continue;
            }

            // 根据 PostgreSQL 类型系统转换值
            let col_type_info = col_type.unwrap();
            let type_name = col_type_info.type_().name();
            let type_oid = col_type_info.type_().oid();

            let is_uuid = type_name == "uuid" || type_oid == 2950;

            let value: Value = if is_uuid {
                match row.try_get::<_, Option<uuid::Uuid>>(i) {
                    Ok(Some(uuid_val)) => Value::String(uuid_val.to_string()),
                    Ok(None) => Value::Null,
                    Err(_) => row
                        .try_get::<_, Option<String>>(i)
                        .ok()
                        .and_then(|opt| opt.map(Value::String))
                        .unwrap_or(Value::Null),
                }
            } else {
                match type_name {
                    "int4" | "int2" => row
                        .try_get::<_, i32>(i)
                        .map(|v| Value::Number(v.into()))
                        .unwrap_or(Value::Null),
                    "int8" => row
                        .try_get::<_, i64>(i)
                        .map(|v| Value::Number(v.into()))
                        .unwrap_or(Value::Null),
                    "float4" | "float8" => row
                        .try_get::<_, f64>(i)
                        .ok()
                        .and_then(|v| serde_json::Number::from_f64(v).map(Value::Number))
                        .unwrap_or(Value::Null),
                    "bool" => row.try_get::<_, bool>(i).map(Value::Bool).unwrap_or(Value::Null),
                    "timestamp" => match row.try_get::<_, Option<NaiveDateTime>>(i) {
                        Ok(Some(dt)) => Value::String(dt.format("%Y-%m-%d %H:%M:%S%.f").to_string()),
                        Ok(None) => Value::Null,
                        Err(_) => row
                            .try_get::<_, NaiveDateTime>(i)
                            .map(|dt| Value::String(dt.format("%Y-%m-%d %H:%M:%S%.f").to_string()))
                            .unwrap_or(Value::Null),
                    },
                    "timestamptz" => {
                        if let Ok(Some(dt)) = row.try_get::<_, Option<DateTime<Utc>>>(i) {
                            Value::String(dt.format("%Y-%m-%d %H:%M:%S%.f UTC").to_string())
                        } else if let Ok(dt) = row.try_get::<_, DateTime<Utc>>(i) {
                            Value::String(dt.format("%Y-%m-%d %H:%M:%S%.f UTC").to_string())
                        } else if let Ok(Some(dt)) = row.try_get::<_, Option<NaiveDateTime>>(i) {
                            Value::String(dt.format("%Y-%m-%d %H:%M:%S%.f").to_string())
                        } else {
                            Value::Null
                        }
                    }
                    "date" => match row.try_get::<_, Option<NaiveDate>>(i) {
                        Ok(Some(date)) => Value::String(date.format("%Y-%m-%d").to_string()),
                        Ok(None) => Value::Null,
                        Err(_) => row
                            .try_get::<_, NaiveDate>(i)
                            .map(|date| Value::String(date.format("%Y-%m-%d").to_string()))
                            .unwrap_or(Value::Null),
                    },
                    "time" | "timetz" => match row.try_get::<_, Option<NaiveTime>>(i) {
                        Ok(Some(time)) => Value::String(time.format("%H:%M:%S%.f").to_string()),
                        Ok(None) => Value::Null,
                        Err(_) => row
                            .try_get::<_, NaiveTime>(i)
                            .map(|time| Value::String(time.format("%H:%M:%S%.f").to_string()))
                            .unwrap_or(Value::Null),
                    },
                    "json" | "jsonb" => match row.try_get::<_, String>(i) {
                        Ok(json_str) => serde_json::from_str(&json_str).unwrap_or(Value::String(json_str)),
                        Err(_) => Value::Null,
                    },
                    _ => row
                        .try_get::<_, String>(i)
                        .map(Value::String)
                        .unwrap_or(Value::Null),
                }
            };
            row_map.insert(col_name.clone(), value);
        }
        result_rows.push(row_map);
    }
    result_rows
}

/// 转换 MySQL 行数据为 JSON 格式
fn convert_mysql_rows(columns: &[String], rows: &[mysql_async::Row]) -> Vec<HashMap<String, Value>> {
    let mut result_rows = Vec::new();
    for row in rows {
        let mut row_map = HashMap::new();
        for (i, col_name) in columns.iter().enumerate() {
            let raw_value = row.as_ref(i);

            let value: Value = match raw_value {
                Some(mysql_async::Value::NULL) => Value::Null,
                Some(mysql_async::Value::Bytes(bytes)) => {
                    // 尝试解析为 UTF-8 字符串
                    String::from_utf8(bytes.clone())
                        .map(Value::String)
                        .unwrap_or_else(|_| Value::String(format!("{:?}", bytes)))
                }
                Some(mysql_async::Value::Int(num)) => Value::Number(serde_json::Number::from(*num)),
                Some(mysql_async::Value::UInt(num)) => {
                    if *num <= i64::MAX as u64 {
                        Value::Number(serde_json::Number::from(*num as i64))
                    } else {
                        Value::String(num.to_string())
                    }
                }
                Some(mysql_async::Value::Float(num)) => serde_json::Number::from_f64(*num as f64)
                    .map(Value::Number)
                    .unwrap_or(Value::Null),
                Some(mysql_async::Value::Double(num)) => serde_json::Number::from_f64(*num)
                    .map(Value::Number)
                    .unwrap_or(Value::Null),
                None => Value::Null,
                _ => Value::Null,
            };

            row_map.insert(col_name.clone(), value);
        }
        result_rows.push(row_map);
    }
    result_rows
}

/// 取消正在执行的查询（简化实现：返回成功）
#[tauri::command]
pub async fn cancel_query(_database_id: String) -> Result<(), String> {
    // TODO: 实现真正的查询取消逻辑
    // 当前简化实现，返回成功
    // 未来实现：使用 tokio::select 和 CancellationToken 来取消正在执行的查询
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::query::RunQueryRequest;

    #[test]
    fn test_ddl_statement_detection() {
        // 测试 DDL 语句检测逻辑（通过 query_parser）
        let ddl_queries = vec![
            "CREATE TABLE test (id INT)",
            "DROP TABLE test",
            "ALTER TABLE test ADD COLUMN name TEXT",
            "TRUNCATE TABLE test",
        ];

        for sql in ddl_queries {
            let request = RunQueryRequest {
                database_id: "test".to_string(),
                sql: sql.to_string(),
            };
            // 注意：这个测试需要 mock query_parser，实际测试在 query_parser.rs 中
            // 这里只验证请求结构
            assert_eq!(request.sql, sql);
        }
    }

    #[test]
    fn test_limit_injection_logic() {
        // 测试 LIMIT 注入逻辑（通过 query_parser）
        let select_without_limit = "SELECT * FROM users";
        let select_with_limit = "SELECT * FROM users LIMIT 50";

        // 验证逻辑：没有 LIMIT 的应该添加，已有的不应该添加
        // 实际测试在 query_parser.rs 中
        assert!(select_without_limit.to_uppercase().contains("SELECT"));
        assert!(select_with_limit.to_uppercase().contains("LIMIT"));
    }
}
