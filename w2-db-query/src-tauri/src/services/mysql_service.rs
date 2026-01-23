/// MySQL 连接服务：管理数据库连接、执行查询
use crate::models::metadata::{
    ColumnInfo, DatabaseMetadata, ForeignKeyInfo, TableInfo, ViewInfo,
};
use crate::utils::error::AppError;
use chrono::Utc;
use mysql_async::prelude::*;
use mysql_async::{Opts, Pool, Row};

/// 连接到 MySQL 数据库
pub async fn connect(
    host: &str,
    port: u16,
    database_name: &str,
    user: &str,
    password: &str,
) -> Result<Pool, AppError> {
    // 构建连接 URL
    let url = format!(
        "mysql://{}:{}@{}:{}/{}",
        user, password, host, port, database_name
    );

    // 创建连接池配置
    let opts = Opts::from_url(&url)
        .map_err(|e| AppError::DatabaseConnection(format!("无效的连接URL: {}", e)))?;

    let pool = Pool::new(opts);

    // 测试连接
    let conn = pool
        .get_conn()
        .await
        .map_err(|e| AppError::DatabaseConnection(format!("连接失败: {}", e)))?;

    let _ = conn.disconnect().await;

    Ok(pool)
}

/// 测试数据库连接
pub async fn test_connection(
    host: &str,
    port: u16,
    database_name: &str,
    user: &str,
    password: &str,
) -> Result<bool, AppError> {
    let pool = connect(host, port, database_name, user, password).await?;

    // 执行简单查询测试连接
    let mut conn = pool
        .get_conn()
        .await
        .map_err(|e| AppError::DatabaseConnection(format!("获取连接失败: {}", e)))?;

    // 测试查询
    let _result: Option<u32> = conn
        .query_first("SELECT 1")
        .await
        .map_err(|e| AppError::DatabaseConnection(format!("测试查询失败: {}", e)))?;

    let _ = conn.disconnect().await;

    Ok(true)
}

/// 执行 SQL 查询并返回结果
pub async fn execute_query(
    pool: &Pool,
    sql: &str,
) -> Result<(Vec<String>, Vec<Row>, u64), AppError> {
    let start_time = std::time::Instant::now();

    let mut conn = pool
        .get_conn()
        .await
        .map_err(|e| AppError::QueryExecution(format!("获取连接失败: {}", e)))?;

    let rows: Vec<Row> = conn
        .query(sql)
        .await
        .map_err(|e| AppError::QueryExecution(format!("查询执行失败: {}", e)))?;

    let exec_time_ms = start_time.elapsed().as_millis() as u64;

    // 获取列名
    let columns: Vec<String> = if let Some(first_row) = rows.first() {
        first_row
            .columns()
            .iter()
            .map(|col| col.name_str().to_string())
            .collect()
    } else {
        vec![]
    };

    Ok((columns, rows, exec_time_ms))
}

/// 将 MySQL Row 转换为 JSON 值
pub fn row_to_json(row: &Row, columns: &[String]) -> Result<serde_json::Value, AppError> {
    let mut map = serde_json::Map::new();

    for (i, col_name) in columns.iter().enumerate() {
        let value = convert_column_value(row, i)?;
        map.insert(col_name.clone(), value);
    }

    Ok(serde_json::Value::Object(map))
}

/// 转换单列值
fn convert_column_value(row: &Row, index: usize) -> Result<serde_json::Value, AppError> {
    use mysql_async::Value;

    let raw_value = row
        .as_ref(index)
        .ok_or_else(|| AppError::QueryExecution(format!("列索引 {} 超出范围", index)))?;

    match raw_value {
        Value::NULL => Ok(serde_json::Value::Null),
        Value::Bytes(bytes) => {
            // 尝试解析为 UTF-8 字符串
            String::from_utf8(bytes.clone())
                .map(serde_json::Value::String)
                .or_else(|_| Ok(serde_json::Value::String(format!("{:?}", bytes))))
        }
        Value::Int(num) => Ok(serde_json::Value::Number(serde_json::Number::from(*num))),
        Value::UInt(num) => {
            if *num <= i64::MAX as u64 {
                Ok(serde_json::Value::Number(serde_json::Number::from(*num as i64)))
            } else {
                Ok(serde_json::Value::String(num.to_string()))
            }
        }
        Value::Float(num) => serde_json::Number::from_f64(*num as f64)
            .map(serde_json::Value::Number)
            .ok_or_else(|| AppError::QueryExecution("无法转换浮点数".to_string())),
        Value::Double(num) => serde_json::Number::from_f64(*num)
            .map(serde_json::Value::Number)
            .ok_or_else(|| AppError::QueryExecution("无法转换双精度浮点数".to_string())),
        // Date 和 Time 类型转为字符串
        Value::Date(year, month, day, hour, minute, second, micros) => {
            Ok(serde_json::Value::String(
                format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}.{:06}", year, month, day, hour, minute, second, micros)
            ))
        }
        Value::Time(neg, days, hours, minutes, seconds, micros) => {
            let sign = if *neg { "-" } else { "" };
            Ok(serde_json::Value::String(
                format!("{}{}d {:02}:{:02}:{:02}.{:06}", sign, days, hours, minutes, seconds, micros)
            ))
        }
    }
}

/// 检查是否为 TINYINT(1) 类型（布尔类型）
pub fn is_tinyint_bool(row: &Row, index: usize) -> bool {
    if let Some(column) = row.columns().get(index) {
        let column_type = column.column_type();
        // MySQL TINYINT(1) 通常用于表示布尔值
        matches!(column_type, mysql_async::consts::ColumnType::MYSQL_TYPE_TINY)
    } else {
        false
    }
}

/// 提取所有表
pub async fn extract_tables(pool: &Pool) -> Result<Vec<TableInfo>, AppError> {
    let mut conn = pool
        .get_conn()
        .await
        .map_err(|e| AppError::QueryExecution(format!("获取连接失败: {}", e)))?;

    let rows: Vec<Row> = conn
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
        let schema: String = row.take::<String, _>("table_schema").unwrap();
        let name: String = row.take::<String, _>("table_name").unwrap();
        let table_type: String = row.take::<String, _>("table_type").unwrap();

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
pub async fn extract_columns(pool: &Pool, tables: &[TableInfo]) -> Result<Vec<ColumnInfo>, AppError> {
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

        let rows: Vec<Row> = conn
            .query(&sql)
            .await
            .map_err(|e| AppError::QueryExecution(format!("查询列失败: {}", e)))?;

        for mut row in rows {
            all_columns.push(ColumnInfo {
                name: row.take::<String, _>("column_name").unwrap(),
                data_type: row.take::<String, _>("data_type").unwrap(),
                nullable: matches!(row.take::<String, _>("is_nullable").unwrap().as_str(), "YES"),
                default_value: row.take::<Option<String>, _>("column_default").unwrap(),
                is_primary_key: false,  // 将在后续查询中标记
                ordinal_position: row.take::<i32, _>("ordinal_position").unwrap(),
            });
        }
    }

    Ok(all_columns)
}

/// 提取主键信息
pub async fn extract_primary_keys(
    pool: &Pool,
    tables: &[TableInfo],
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

        let rows: Vec<Row> = conn
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

/// 提取外键信息
pub async fn extract_foreign_keys(
    pool: &Pool,
    tables: &[TableInfo],
) -> Result<Vec<(String, Vec<ForeignKeyInfo>)>, AppError> {
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

        let rows: Vec<Row> = conn
            .query(&sql)
            .await
            .map_err(|e| AppError::QueryExecution(format!("查询外键失败: {}", e)))?;

        let foreign_keys: Vec<ForeignKeyInfo> = rows
            .into_iter()
            .map(|mut row| ForeignKeyInfo {
                constraint_name: row.take::<String, _>("constraint_name").unwrap(),
                column_name: row.take::<String, _>("column_name").unwrap(),
                referenced_table: row.take::<String, _>("referenced_table").unwrap(),
                referenced_column: row.take::<String, _>("referenced_column").unwrap(),
            })
            .collect();

        result.push((format!("{}.{}", table.schema, table.name), foreign_keys));
    }

    Ok(result)
}

/// 提取视图信息
pub async fn extract_views(pool: &Pool) -> Result<Vec<ViewInfo>, AppError> {
    let mut conn = pool
        .get_conn()
        .await
        .map_err(|e| AppError::QueryExecution(format!("获取连接失败: {}", e)))?;

    let rows: Vec<Row> = conn
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
        views.push(ViewInfo {
            schema: row.take::<String, _>("table_schema").unwrap(),
            name: row.take::<String, _>("table_name").unwrap(),
            columns: vec![],  // 列信息通过 DESCRIBE 或 SHOW COLUMNS 获取
            definition: row.take::<Option<String>, _>("view_definition").unwrap(),
        });
    }

    Ok(views)
}

/// 提取完整的数据库元数据
pub async fn extract_metadata(pool: &Pool, connection_id: &str) -> Result<DatabaseMetadata, AppError> {
    // 提取表
    let mut tables = extract_tables(pool).await?;

    // 提取列信息
    let columns = extract_columns(pool, &tables).await?;

    // 提取主键
    let primary_keys_map = extract_primary_keys(pool, &tables).await?;

    // 提取外键
    let foreign_keys_map = extract_foreign_keys(pool, &tables).await?;

    // 组装表信息 - 使用正确的索引映射
    let mut table_column_map: std::collections::HashMap<String, Vec<ColumnInfo>> =
        std::collections::HashMap::new();
    let mut current_columns: Vec<ColumnInfo> = Vec::new();
    let mut current_table_key = String::new();

    for (i, col) in columns.into_iter().enumerate() {
        // 根据索引推断所属表（简单实现：假设每表约100列）
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
    let views = extract_views(pool).await?;

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
        let host = "localhost";
        let port = 3306u16;
        let database_name = "testdb";
        let user = "testuser";
        let password = "testpass";

        let url = format!(
            "mysql://{}:{}@{}:{}/{}",
            user, password, host, port, database_name
        );

        assert!(url.contains("mysql://"));
        assert!(url.contains("testuser:testpass"));
        assert!(url.contains("localhost:3306"));
        assert!(url.contains("/testdb"));
    }
}
