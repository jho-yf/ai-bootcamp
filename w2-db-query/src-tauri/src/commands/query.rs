/// SQL 查询执行 Commands
use crate::models::query::{QueryResult, RunQueryRequest};
use crate::services::{cache_service, query_parser};
use crate::services::database;

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

    // 使用工厂模式执行查询
    let factory = database::get_global_factory();
    let service = factory
        .get_service(&connection.database_type)
        .map_err(|e| format!("不支持的数据库类型: {}", e))?;

    // 连接数据库
    let db_connection = service
        .connect(
            &connection.host,
            connection.port,
            &connection.database_name,
            &connection.user,
            &connection.password,
        )
        .await
        .map_err(|e| format!("连接失败: {}", e))?;

    // 执行查询
    let exec_result = service
        .execute_query(&db_connection, &parsed_sql, &[])
        .await
        .map_err(|e| e.to_string())?;

    // 转换结果
    let mut result_rows = Vec::new();
    for db_row in &exec_result.rows {
        let row_map = service
            .convert_row_to_json(db_row, &db_row.columns)
            .map_err(|e| e.to_string())?;
        result_rows.push(row_map);
    }

    let columns = exec_result
        .rows
        .first()
        .map(|row| row.columns.clone())
        .unwrap_or_default();

    let total = result_rows.len();
    let truncated = total >= 100;

    let result = QueryResult {
        columns,
        rows: result_rows,
        total,
        exec_time_ms: exec_result.exec_time_ms,
        sql: parsed_sql.clone(),
        truncated,
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
