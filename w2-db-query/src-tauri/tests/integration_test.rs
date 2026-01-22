use std::env;
/// 集成测试：需要 PostgreSQL 数据库
///
/// 运行这些测试有两种方式：
///
/// 方式 1：使用 Docker（推荐，但需要 testcontainers API 兼容）
/// ```bash
/// cargo test --test integration_test --features testcontainers
/// ```
///
/// 方式 2：使用外部 PostgreSQL（通过环境变量配置）
/// ```bash
/// export TEST_DB_HOST=localhost
/// export TEST_DB_PORT=5432
/// export TEST_DB_NAME=testdb
/// export TEST_DB_USER=testuser
/// export TEST_DB_PASSWORD=testpass
/// cargo test --test integration_test
/// ```
///
/// 注意：这些测试会执行真实的数据库操作
use w2_db_query_lib::services::postgres_service;

/// 从环境变量获取测试数据库配置
fn get_test_db_config() -> Option<(String, u16, String, String, String)> {
    let host = env::var("TEST_DB_HOST").ok()?;
    let port = env::var("TEST_DB_PORT").ok()?.parse::<u16>().ok()?;
    let database_name = env::var("TEST_DB_NAME").ok()?;
    let user = env::var("TEST_DB_USER").ok()?;
    let password = env::var("TEST_DB_PASSWORD").ok()?;

    Some((host, port, database_name, user, password))
}

#[tokio::test]
async fn test_connect() {
    let Some((host, port, database_name, user, password)) = get_test_db_config() else {
        println!("跳过测试：未设置 TEST_DB_* 环境变量");
        println!("设置环境变量或使用 --features testcontainers 运行测试");
        return;
    };

    let result = postgres_service::connect(&host, port, &database_name, &user, &password).await;
    assert!(result.is_ok(), "应该能够连接到 PostgreSQL 数据库");
}

#[tokio::test]
async fn test_test_connection() {
    let Some((host, port, database_name, user, password)) = get_test_db_config() else {
        println!("跳过测试：未设置 TEST_DB_* 环境变量");
        return;
    };

    let result =
        postgres_service::test_connection(&host, port, &database_name, &user, &password).await;
    assert!(result.is_ok(), "连接测试应该成功");
    assert_eq!(result.unwrap(), true);
}

#[tokio::test]
async fn test_execute_query() {
    let Some((host, port, database_name, user, password)) = get_test_db_config() else {
        println!("跳过测试：未设置 TEST_DB_* 环境变量");
        return;
    };

    let client = postgres_service::connect(&host, port, &database_name, &user, &password)
        .await
        .expect("应该能够连接到数据库");

    // 创建测试表
    client
        .execute(
            "CREATE TABLE IF NOT EXISTS test_table (id SERIAL PRIMARY KEY, name TEXT)",
            &[],
        )
        .await
        .expect("应该能够创建表");

    // 清理旧数据
    client.execute("DELETE FROM test_table", &[]).await.ok();

    // 插入测试数据
    client
        .execute(
            "INSERT INTO test_table (name) VALUES ($1), ($2)",
            &[&"Alice", &"Bob"],
        )
        .await
        .expect("应该能够插入数据");

    // 执行查询
    let (columns, rows, exec_time_ms) =
        postgres_service::execute_query(&client, "SELECT id, name FROM test_table ORDER BY id")
            .await
            .expect("应该能够执行查询");

    // 验证结果
    assert_eq!(columns, vec!["id", "name"]);
    assert_eq!(rows.len(), 2);
    assert!(exec_time_ms > 0);

    // 验证数据
    let id1: i32 = rows[0].get(0);
    let name1: String = rows[0].get(1);
    assert_eq!(id1, 1);
    assert_eq!(name1, "Alice");

    let id2: i32 = rows[1].get(0);
    let name2: String = rows[1].get(1);
    assert_eq!(id2, 2);
    assert_eq!(name2, "Bob");
}

#[tokio::test]
async fn test_execute_query_empty_result() {
    let Some((host, port, database_name, user, password)) = get_test_db_config() else {
        println!("跳过测试：未设置 TEST_DB_* 环境变量");
        return;
    };

    let client = postgres_service::connect(&host, port, &database_name, &user, &password)
        .await
        .expect("应该能够连接到数据库");

    let (_columns, rows, _exec_time_ms) = postgres_service::execute_query(
        &client,
        "SELECT * FROM information_schema.tables WHERE table_name = 'non_existent_table'",
    )
    .await
    .expect("应该能够执行查询");

    // 空结果也应该正常返回
    assert_eq!(rows.len(), 0);
}

#[tokio::test]
async fn test_connection_failure() {
    // 测试连接失败的情况（使用不存在的端口）
    let result = postgres_service::connect("localhost", 9999, "nonexistent", "user", "pass").await;
    assert!(result.is_err(), "连接不存在的数据库应该失败");
}

#[cfg(feature = "testcontainers")]
mod testcontainers_tests {
    use super::*;

    // TODO: 当 testcontainers 0.15 API 兼容性问题解决后，在这里实现自动容器启动
    // 当前由于 API 不兼容，使用环境变量方式更可靠

    #[tokio::test]
    #[ignore]
    async fn test_with_testcontainers() {
        // 占位符：未来可以在这里实现使用 testcontainers 自动启动容器的测试
        println!("testcontainers 集成测试待实现");
    }
}
