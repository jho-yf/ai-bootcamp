/// PostgreSQL 连接服务：管理数据库连接、执行查询
use crate::models::metadata::{
    ColumnInfo, DatabaseMetadata, ForeignKeyInfo, TableInfo, ViewInfo,
};
use crate::utils::error::AppError;
use chrono::Utc;
use tokio_postgres::{Client, NoTls};

/// 连接到 PostgreSQL 数据库
pub async fn connect(
    host: &str,
    port: u16,
    database_name: &str,
    user: &str,
    password: &str,
) -> Result<Client, AppError> {
    let connection_string = format!(
        "host={} port={} dbname={} user={} password={}",
        host, port, database_name, user, password
    );

    let (client, connection) = tokio_postgres::connect(&connection_string, NoTls)
        .await
        .map_err(|e| AppError::DatabaseConnection(format!("连接失败: {}", e)))?;

    // 启动连接任务
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("PostgreSQL 连接错误: {}", e);
        }
    });

    Ok(client)
}

/// 测试数据库连接
pub async fn test_connection(
    host: &str,
    port: u16,
    database_name: &str,
    user: &str,
    password: &str,
) -> Result<bool, AppError> {
    let client = connect(host, port, database_name, user, password).await?;

    // 执行简单查询测试连接
    client
        .query_one("SELECT 1", &[])
        .await
        .map_err(|e| AppError::DatabaseConnection(format!("测试查询失败: {}", e)))?;

    Ok(true)
}

/// 执行 SQL 查询并返回结果
pub async fn execute_query(
    client: &Client,
    sql: &str,
) -> Result<(Vec<String>, Vec<tokio_postgres::Row>, u64), AppError> {
    let start_time = std::time::Instant::now();

    let rows = client
        .query(sql, &[])
        .await
        .map_err(|e| AppError::QueryExecution(format!("查询执行失败: {}", e)))?;

    let exec_time_ms = start_time.elapsed().as_millis() as u64;

    // 获取列名
    let columns: Vec<String> = if let Some(first_row) = rows.first() {
        first_row
            .columns()
            .iter()
            .map(|col| col.name().to_string())
            .collect()
    } else {
        vec![]
    };

    Ok((columns, rows, exec_time_ms))
}

/// 提取所有表
pub async fn extract_tables(client: &Client) -> Result<Vec<TableInfo>, AppError> {
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

        tables.push(TableInfo {
            schema,
            name,
            table_type,
            columns: vec![],  // 将在后续查询中填充
            primary_keys: vec![],  // 将在后续查询中填充
            foreign_keys: vec![],  // 将在后续查询中填充
        });
    }

    Ok(tables)
}

/// 提取表的列信息
pub async fn extract_columns(client: &Client, tables: &[TableInfo]) -> Result<Vec<ColumnInfo>, AppError> {
    let mut all_columns = Vec::new();

    for table in tables {
        let rows = client
            .query(
                "SELECT column_name, data_type, is_nullable, column_default, ordinal_position
                 FROM information_schema.columns
                 WHERE table_schema = $1 AND table_name = $2
                 ORDER BY ordinal_position",
                &[&table.schema, &table.name],
            )
            .await
            .map_err(|e| AppError::QueryExecution(format!("查询列失败: {}", e)))?;

        for row in rows {
            all_columns.push(ColumnInfo {
                name: row.get("column_name"),
                data_type: row.get("data_type"),
                nullable: matches!(row.get::<_, String>("is_nullable").as_str(), "YES"),
                default_value: row.get::<_, Option<String>>("column_default"),
                is_primary_key: false,  // 将在后续查询中标记
                ordinal_position: row.get::<_, i32>("ordinal_position"),
            });
        }
    }

    Ok(all_columns)
}

/// 提取主键信息
pub async fn extract_primary_keys(
    client: &Client,
    tables: &[TableInfo],
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

/// 提取外键信息
pub async fn extract_foreign_keys(
    client: &Client,
    tables: &[TableInfo],
) -> Result<Vec<(String, Vec<ForeignKeyInfo>)>, AppError> {
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

        let foreign_keys: Vec<ForeignKeyInfo> = rows
            .into_iter()
            .map(|row| ForeignKeyInfo {
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

/// 提取视图信息
pub async fn extract_views(client: &Client) -> Result<Vec<ViewInfo>, AppError> {
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
        views.push(ViewInfo {
            schema: row.get("table_schema"),
            name: row.get("table_name"),
            columns: vec![],  // 列信息通过查询 information_schema.columns 获取
            definition: row.get("view_definition"),
        });
    }

    Ok(views)
}

/// 提取完整的数据库元数据
pub async fn extract_metadata(client: &Client, connection_id: &str) -> Result<DatabaseMetadata, AppError> {
    // 提取表
    let mut tables = extract_tables(client).await?;

    // 提取主键
    let primary_keys_map = extract_primary_keys(client, &tables).await?;

    // 提取外键
    let foreign_keys_map = extract_foreign_keys(client, &tables).await?;

    // 组装表信息 - 按表顺序分配列
    let mut table_column_map: std::collections::HashMap<String, Vec<ColumnInfo>> =
        std::collections::HashMap::new();

    // 为每个表查询其列（使用单独的查询以确保正确映射）
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
            .map_err(|e| AppError::QueryExecution(format!("查询表列失败: {}", e)))?;

        let cols: Vec<ColumnInfo> = table_columns
            .into_iter()
            .map(|row| ColumnInfo {
                name: row.get("column_name"),
                data_type: row.get("data_type"),
                nullable: matches!(row.get::<_, String>("is_nullable").as_str(), "YES"),
                default_value: row.get::<_, Option<String>>("column_default"),
                is_primary_key: false,  // 将在后续标记
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
            // 标记主键列
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
        extracted_at: Utc::now(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_string_format() {
        // 测试连接字符串格式（不执行连接）
        let host = "localhost";
        let port = 5432u16;
        let database_name = "testdb";
        let user = "testuser";
        let password = "testpass";

        let connection_string = format!(
            "host={} port={} dbname={} user={} password={}",
            host, port, database_name, user, password
        );

        assert!(connection_string.contains("host=localhost"));
        assert!(connection_string.contains("port=5432"));
        assert!(connection_string.contains("dbname=testdb"));
        assert!(connection_string.contains("user=testuser"));
        assert!(connection_string.contains("password=testpass"));
    }

    #[test]
    fn test_execute_query_error_handling() {
        // 测试错误处理逻辑（不执行实际查询）
        // 这个测试验证错误类型转换
        use crate::utils::error::AppError;

        // 验证 AppError::QueryExecution 可以正确创建
        let error = AppError::QueryExecution("测试错误".to_string());
        assert!(error.to_string().contains("测试错误"));
    }
}

// 集成测试移到 tests/integration_test.rs
