/// PostgreSQL 连接服务：管理数据库连接、执行查询
use crate::utils::error::AppError;
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

// 单元测试移到 tests/integration_test.rs
