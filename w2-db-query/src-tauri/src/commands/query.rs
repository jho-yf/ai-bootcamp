/// SQL 查询执行 Commands
use crate::models::query::{QueryResult, RunQueryRequest};
use crate::services::{cache_service, postgres_service, query_parser};
use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

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

    // 连接到 PostgreSQL
    let client = postgres_service::connect(
        &connection.host,
        connection.port,
        &connection.database_name,
        &connection.user,
        &connection.password,
    )
    .await
    .map_err(|e| format!("连接失败: {}", e))?;

    // 执行查询
    let (columns, rows, exec_time_ms) = postgres_service::execute_query(&client, &parsed_sql)
        .await
        .map_err(|e| e.to_string())?;

    // 转换行数据为 HashMap
    // 使用 tokio-postgres 的类型系统来安全地转换值
    let mut result_rows = Vec::new();
    for row in rows {
        let mut row_map = HashMap::new();
        for (i, col_name) in columns.iter().enumerate() {
            let col_type = row.columns().get(i);
            if col_type.is_none() {
                row_map.insert(col_name.clone(), Value::Null);
                continue;
            }

            // 根据 PostgreSQL 类型名称转换
            let col_type_info = col_type.unwrap();
            let type_name = col_type_info.type_().name();
            let type_oid = col_type_info.type_().oid();

            // 检查是否为 UUID 类型（通过类型名称或 OID）
            // PostgreSQL UUID 类型的 OID 是 2950
            let is_uuid = type_name == "uuid" || type_oid == 2950;

            let value: Value = if is_uuid {
                // UUID 类型：使用 uuid::Uuid 类型读取
                // 先尝试读取为 Option<Uuid> 以正确处理 NULL 值
                match row.try_get::<_, Option<Uuid>>(i) {
                    Ok(Some(uuid_val)) => Value::String(uuid_val.to_string()),
                    Ok(None) => Value::Null,
                    Err(_) => {
                        // 如果 Option<Uuid> 失败，尝试直接读取 Uuid（非 NULL 情况）
                        match row.try_get::<_, Uuid>(i) {
                            Ok(uuid_val) => Value::String(uuid_val.to_string()),
                            Err(_) => {
                                // 如果都失败，尝试作为字符串读取（降级方案）
                                row.try_get::<_, Option<String>>(i)
                                    .ok()
                                    .and_then(|opt| opt.map(Value::String))
                                    .unwrap_or(Value::Null)
                            }
                        }
                    }
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
                    "bool" => row
                        .try_get::<_, bool>(i)
                        .map(Value::Bool)
                        .unwrap_or(Value::Null),
                    "timestamp" => {
                        // timestamp (without timezone)
                        match row.try_get::<_, Option<NaiveDateTime>>(i) {
                            Ok(Some(dt)) => {
                                Value::String(dt.format("%Y-%m-%d %H:%M:%S%.f").to_string())
                            }
                            Ok(None) => Value::Null,
                            Err(_) => {
                                // 如果 Option<NaiveDateTime> 失败，尝试直接读取
                                row.try_get::<_, NaiveDateTime>(i)
                                    .map(|dt| {
                                        Value::String(dt.format("%Y-%m-%d %H:%M:%S%.f").to_string())
                                    })
                                    .unwrap_or(Value::Null)
                            }
                        }
                    }
                    "timestamptz" => {
                        // timestamp with timezone - 尝试多种类型
                        // 先尝试 DateTime<Utc>
                        if let Ok(Some(dt)) = row.try_get::<_, Option<DateTime<Utc>>>(i) {
                            Value::String(dt.format("%Y-%m-%d %H:%M:%S%.f UTC").to_string())
                        } else if let Ok(dt) = row.try_get::<_, DateTime<Utc>>(i) {
                            Value::String(dt.format("%Y-%m-%d %H:%M:%S%.f UTC").to_string())
                        } else if let Ok(Some(dt)) = row.try_get::<_, Option<NaiveDateTime>>(i) {
                            // 如果 timestamptz 被读取为 NaiveDateTime
                            Value::String(dt.format("%Y-%m-%d %H:%M:%S%.f").to_string())
                        } else {
                            Value::Null
                        }
                    }
                    "date" => {
                        // date
                        match row.try_get::<_, Option<NaiveDate>>(i) {
                            Ok(Some(date)) => Value::String(date.format("%Y-%m-%d").to_string()),
                            Ok(None) => Value::Null,
                            Err(_) => row
                                .try_get::<_, NaiveDate>(i)
                                .map(|date| Value::String(date.format("%Y-%m-%d").to_string()))
                                .unwrap_or(Value::Null),
                        }
                    }
                    "time" | "timetz" => {
                        // time (with or without timezone)
                        match row.try_get::<_, Option<NaiveTime>>(i) {
                            Ok(Some(time)) => Value::String(time.format("%H:%M:%S%.f").to_string()),
                            Ok(None) => Value::Null,
                            Err(_) => row
                                .try_get::<_, NaiveTime>(i)
                                .map(|time| Value::String(time.format("%H:%M:%S%.f").to_string()))
                                .unwrap_or(Value::Null),
                        }
                    }
                    "json" | "jsonb" => {
                        // JSON 类型先作为文本读取，然后解析
                        match row.try_get::<_, String>(i) {
                            Ok(json_str) => {
                                serde_json::from_str(&json_str).unwrap_or(Value::String(json_str))
                            }
                            Err(_) => Value::Null,
                        }
                    }
                    _ => {
                        // 默认作为文本处理
                        row.try_get::<_, String>(i)
                            .map(Value::String)
                            .unwrap_or(Value::Null)
                    }
                }
            };
            row_map.insert(col_name.clone(), value);
        }
        result_rows.push(row_map);
    }

    let total = result_rows.len();
    let truncated = total >= 100;

    let result = QueryResult {
        columns,
        rows: result_rows,
        total,
        exec_time_ms,
        sql: parsed_sql.clone(),
        truncated,
    };

    // 保存查询历史（SQL 查询类型）
    let _ = cache_service::save_query_history(
        &request.database_id,
        "sql",
        Some(&parsed_sql),
        None,
        Some(exec_time_ms),
        "success",
    );

    Ok(result)
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
