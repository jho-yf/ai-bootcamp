use pg_mcp::config::{AppConfig, DatabaseConfig, LlmConfig, ServerConfig};
use pg_mcp::executor::QueryExecutor;
use pg_mcp::metadata::MetadataCache;
use pg_mcp::validator::SqlValidator;
use sqlx::postgres::PgPoolOptions;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

const DATABASE_URL: &str = "postgresql://pgmcp:pgmcp_test@localhost:15432/pgmcp_test";

async fn create_pool() -> sqlx::postgres::PgPool {
    PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(5))
        .connect(DATABASE_URL)
        .await
        .expect("Failed to connect to test database. Is podman PostgreSQL running?")
}

fn create_test_config() -> AppConfig {
    AppConfig {
        database: DatabaseConfig {
            url: DATABASE_URL.to_string(),
            schema: "public".to_string(),
            allowed_tables: vec![],
            excluded_tables: vec![],
        },
        llm: LlmConfig {
            api_url: "https://api.openai.com/v1".to_string(),
            api_key: "test-key".to_string(),
            model: "gpt-4o".to_string(),
            temperature: 0.0,
            max_tokens: 1000,
        },
        server: ServerConfig {
            max_rows: 1000,
            query_timeout_secs: 30,
            prompt_budget: 8000,
            debug: true,
            metadata_refresh_secs: 0,
        },
    }
}

// ============================================================================
// 4.2 Metadata Loading Integration Tests
// ============================================================================

#[tokio::test]
async fn test_metadata_loads_all_tables() {
    let pool = create_pool().await;
    let cache = MetadataCache::new(pool.clone(), "public".to_string(), HashSet::new());

    cache.load().await.expect("Metadata load failed");

    let table_count = cache.table_count().await;
    assert!(table_count >= 3, "Expected at least 3 tables, got {}", table_count);
}

#[tokio::test]
async fn test_metadata_loads_primary_keys() {
    let pool = create_pool().await;
    let cache = MetadataCache::new(pool.clone(), "public".to_string(), HashSet::new());

    cache.load().await.expect("Metadata load failed");

    // Verify we can get context for users table with PK info
    let context = cache.get_relevant_context("users", 10000).await;
    assert!(
        context.contains("Primary keys: id"),
        "Expected PK info in context, got: {}",
        context
    );
}

#[tokio::test]
async fn test_metadata_loads_indexes() {
    let pool = create_pool().await;
    let cache = MetadataCache::new(pool.clone(), "public".to_string(), HashSet::new());

    cache.load().await.expect("Metadata load failed");

    // The idx_orders_user_id index should appear in orders table context
    let context = cache.get_relevant_context("orders", 10000).await;
    assert!(
        context.contains("idx_orders_user_id"),
        "Expected index info in orders context, got: {}",
        context
    );
}

#[tokio::test]
async fn test_metadata_loads_views() {
    let pool = create_pool().await;
    let cache = MetadataCache::new(pool.clone(), "public".to_string(), HashSet::new());

    cache.load().await.expect("Metadata load failed");

    let view_count = cache.view_count().await;
    assert!(view_count >= 1, "Expected at least 1 view, got {}", view_count);
}

#[tokio::test]
async fn test_metadata_context_relevant_table_match() {
    let pool = create_pool().await;
    let cache = MetadataCache::new(pool.clone(), "public".to_string(), HashSet::new());

    cache.load().await.expect("Metadata load failed");

    let context = cache.get_relevant_context("users", 5000).await;
    assert!(
        context.contains("Table: users"),
        "Expected users table in context"
    );
    // Without "orders" keyword, orders table should score lower but may still appear
}

#[tokio::test]
async fn test_metadata_context_excluded_tables_filtered() {
    let pool = create_pool().await;
    let excluded: HashSet<String> = vec!["users".to_string()].into_iter().collect();
    let cache = MetadataCache::new(pool.clone(), "public".to_string(), excluded);

    cache.load().await.expect("Metadata load failed");

    let context = cache.get_relevant_context("all data", 10000).await;
    assert!(
        !context.contains("Table: users"),
        "Excluded table 'users' should not appear in context"
    );
    assert!(
        context.contains("Table: orders"),
        "Non-excluded table 'orders' should appear"
    );
}

#[tokio::test]
async fn test_metadata_context_budget_truncation() {
    let pool = create_pool().await;
    let cache = MetadataCache::new(pool.clone(), "public".to_string(), HashSet::new());

    cache.load().await.expect("Metadata load failed");

    // Use a very small budget
    let context = cache.get_relevant_context("all data", 50).await;
    assert!(
        context.len() < 500,
        "Context should be truncated with small budget, got {} chars",
        context.len()
    );
}

// ============================================================================
// 4.3 End-to-End Query Integration Tests
// ============================================================================

#[tokio::test]
async fn test_executor_simple_select() {
    let pool = create_pool().await;
    let executor = QueryExecutor::new(pool.clone(), 1000, 30, true);

    let result = executor
        .execute("SELECT * FROM users")
        .await
        .expect("Query execution failed");

    assert!(result.error.is_none(), "Unexpected error: {:?}", result.error);
    assert!(result.row_count >= 3, "Expected at least 3 users, got {}", result.row_count);
    assert!(!result.truncated, "Should not be truncated");
}

#[tokio::test]
async fn test_executor_join_query() {
    let pool = create_pool().await;
    let executor = QueryExecutor::new(pool.clone(), 1000, 30, true);

    let result = executor
        .execute("SELECT u.name, o.total FROM users u JOIN orders o ON u.id = o.user_id")
        .await
        .expect("Query execution failed");

    assert!(result.error.is_none(), "Unexpected error: {:?}", result.error);
    assert!(result.row_count >= 3, "Expected at least 3 order rows");
}

#[tokio::test]
async fn test_executor_aggregation_query() {
    let pool = create_pool().await;
    let executor = QueryExecutor::new(pool.clone(), 1000, 30, true);

    let result = executor
        .execute("SELECT COUNT(*) as cnt FROM users")
        .await
        .expect("Query execution failed");

    assert!(result.error.is_none(), "Unexpected error: {:?}", result.error);
    assert_eq!(result.row_count, 1);
    // Verify the count value is a number (int8)
    let count_val = &result.rows[0]["cnt"];
    assert!(count_val.is_number(), "Count should be a number, got: {}", count_val);
}

#[tokio::test]
async fn test_executor_column_names() {
    let pool = create_pool().await;
    let executor = QueryExecutor::new(pool.clone(), 1000, 30, true);

    let result = executor
        .execute("SELECT name, email FROM users WHERE id = 1")
        .await
        .expect("Query execution failed");

    assert!(result.columns.contains(&"name".to_string()));
    assert!(result.columns.contains(&"email".to_string()));
}

// ============================================================================
// 4.4 READ ONLY Protection Tests
// ============================================================================

#[tokio::test]
async fn test_read_only_blocks_insert() {
    let pool = create_pool().await;
    let executor = QueryExecutor::new(pool.clone(), 1000, 30, true);

    // The validator would catch this first, but we also verify the executor's
    // READ ONLY transaction blocks it
    let result = executor
        .execute("INSERT INTO users (name, email) VALUES ('hack', 'hack@test.com')")
        .await;

    // Should fail because READ ONLY transaction
    match result {
        Ok(r) => {
            assert!(r.error.is_some(), "INSERT should fail in READ ONLY transaction");
        }
        Err(e) => {
            assert!(
                e.to_string().contains("read-only") || e.to_string().contains("cannot"),
                "Expected read-only error, got: {}",
                e
            );
        }
    }
}

#[tokio::test]
async fn test_read_only_blocks_update() {
    let pool = create_pool().await;
    let executor = QueryExecutor::new(pool.clone(), 1000, 30, true);

    let result = executor
        .execute("UPDATE users SET name = 'hack' WHERE id = 1")
        .await;

    match result {
        Ok(r) => {
            assert!(r.error.is_some(), "UPDATE should fail in READ ONLY transaction");
        }
        Err(e) => {
            assert!(
                e.to_string().contains("read-only") || e.to_string().contains("cannot"),
                "Expected read-only error, got: {}",
                e
            );
        }
    }
}

#[tokio::test]
async fn test_read_only_blocks_delete() {
    let pool = create_pool().await;
    let executor = QueryExecutor::new(pool.clone(), 1000, 30, true);

    let result = executor
        .execute("DELETE FROM users WHERE id = 1")
        .await;

    match result {
        Ok(r) => {
            assert!(r.error.is_some(), "DELETE should fail in READ ONLY transaction");
        }
        Err(e) => {
            assert!(
                e.to_string().contains("read-only") || e.to_string().contains("cannot"),
                "Expected read-only error, got: {}",
                e
            );
        }
    }
}

// ============================================================================
// 4.5 Error Handling Integration Tests
// ============================================================================

#[tokio::test]
async fn test_nonexistent_table_error() {
    let pool = create_pool().await;
    let executor = QueryExecutor::new(pool.clone(), 1000, 30, true);

    let result = executor
        .execute("SELECT * FROM nonexistent_table")
        .await
        .expect("Should not panic");

    assert!(result.error.is_some(), "Expected error for nonexistent table");
}

#[tokio::test]
async fn test_invalid_column_error() {
    let pool = create_pool().await;
    let executor = QueryExecutor::new(pool.clone(), 1000, 30, true);

    let result = executor
        .execute("SELECT nonexistent_column FROM users")
        .await
        .expect("Should not panic");

    assert!(result.error.is_some(), "Expected error for nonexistent column");
}

#[tokio::test]
async fn test_debug_mode_shows_error_details() {
    let pool = create_pool().await;
    let executor_debug = QueryExecutor::new(pool.clone(), 1000, 30, true);
    let executor_prod = QueryExecutor::new(pool.clone(), 1000, 30, false);

    let result_debug = executor_debug
        .execute("SELECT * FROM nonexistent_table_xyz")
        .await
        .expect("Should not panic");

    let result_prod = executor_prod
        .execute("SELECT * FROM nonexistent_table_xyz")
        .await
        .expect("Should not panic");

    assert!(result_debug.error.as_ref().unwrap().contains("nonexistent_table_xyz"));
    assert!(!result_prod.error.as_ref().unwrap().contains("nonexistent_table_xyz"));
}

// ============================================================================
// Type Serialization Tests
// ============================================================================

#[tokio::test]
async fn test_integer_serialization() {
    let pool = create_pool().await;
    let executor = QueryExecutor::new(pool.clone(), 1000, 30, true);

    let result = executor
        .execute("SELECT id FROM users WHERE id = 1 LIMIT 1")
        .await
        .expect("Query failed");

    assert!(!result.rows.is_empty(), "Should have at least 1 row");
    let id_val = &result.rows[0]["id"];
    assert!(id_val.is_number(), "id should serialize as number, got: {}", id_val);
    assert_eq!(id_val.as_i64(), Some(1));
}

#[tokio::test]
async fn test_boolean_serialization() {
    let pool = create_pool().await;
    let executor = QueryExecutor::new(pool.clone(), 1000, 30, true);

    let result = executor
        .execute("SELECT active FROM users WHERE id = 1")
        .await
        .expect("Query failed");

    let active_val = &result.rows[0]["active"];
    assert!(active_val.is_boolean(), "active should serialize as boolean, got: {}", active_val);
}

#[tokio::test]
async fn test_null_serialization() {
    let pool = create_pool().await;
    let executor = QueryExecutor::new(pool.clone(), 1000, 30, true);

    // email can be null
    let result = executor
        .execute("SELECT NULL as val")
        .await
        .expect("Query failed");

    let null_val = &result.rows[0]["val"];
    assert!(null_val.is_null(), "NULL should serialize as JSON null, got: {}", null_val);
}

#[tokio::test]
async fn test_numeric_serialization() {
    let pool = create_pool().await;
    let executor = QueryExecutor::new(pool.clone(), 1000, 30, true);

    let result = executor
        .execute("SELECT total FROM orders WHERE id = 1 LIMIT 1")
        .await
        .expect("Query failed");

    assert!(!result.rows.is_empty(), "Should have at least 1 row");
    let total_val = &result.rows[0]["total"];
    // NUMERIC falls back to string serialization since sqlx doesn't have native f64 for NUMERIC
    assert!(
        total_val.is_string() || total_val.is_number(),
        "NUMERIC should serialize as string or number, got: {}",
        total_val
    );
}

// ============================================================================
// LIMIT Protection Tests
// ============================================================================

#[tokio::test]
async fn test_limit_protection_wraps_query() {
    let pool = create_pool().await;
    let executor = QueryExecutor::new(pool.clone(), 2, 30, true);

    let result = executor
        .execute("SELECT * FROM users")
        .await
        .expect("Query failed");

    assert!(result.truncated, "Should be truncated with max_rows=2 and 3 users");
    assert_eq!(result.row_count, 2);
}

#[tokio::test]
async fn test_existing_limit_preserved() {
    let pool = create_pool().await;
    let executor = QueryExecutor::new(pool.clone(), 1000, 30, true);

    let result = executor
        .execute("SELECT * FROM users LIMIT 1")
        .await
        .expect("Query failed");

    assert_eq!(result.row_count, 1);
    assert!(!result.truncated);
}

// ============================================================================
// MCP Protocol Tests
// ============================================================================

#[tokio::test]
async fn test_mcp_server_info() {
    let config = Arc::new(create_test_config());
    let pool = create_pool().await;

    let metadata = Arc::new(MetadataCache::new(
        pool.clone(),
        "public".to_string(),
        HashSet::new(),
    ));
    metadata.load().await.expect("Metadata load failed");

    let llm = Arc::new(pg_mcp::llm::LlmClient::new(&config.llm));
    let executor = Arc::new(QueryExecutor::new(
        pool.clone(),
        config.server.max_rows,
        config.server.query_timeout_secs,
        config.server.debug,
    ));
    let validator = SqlValidator::new(HashSet::new(), HashSet::new());

    let server = pg_mcp::server::PgMcpServer::new(config, metadata, llm, executor, validator);

    let info = rmcp::handler::server::ServerHandler::get_info(&server);

    // Verify server info contains expected capabilities
    assert_eq!(info.server_info.name, "pg-mcp");
}

#[tokio::test]
async fn test_mcp_server_creation() {
    let config = Arc::new(create_test_config());
    let pool = create_pool().await;

    let metadata = Arc::new(MetadataCache::new(
        pool.clone(),
        "public".to_string(),
        HashSet::new(),
    ));
    metadata.load().await.expect("Metadata load failed");

    let llm = Arc::new(pg_mcp::llm::LlmClient::new(&config.llm));
    let executor = Arc::new(QueryExecutor::new(
        pool.clone(),
        config.server.max_rows,
        config.server.query_timeout_secs,
        config.server.debug,
    ));
    let validator = SqlValidator::new(HashSet::new(), HashSet::new());

    // Verify server can be created without panic
    let _server = pg_mcp::server::PgMcpServer::new(config, metadata, llm, executor, validator);
}

#[tokio::test]
async fn test_mcp_query_tool_validator_error() {
    // This test verifies that validation errors return INVALID_PARAMS
    // We'll test this via the validator directly since it's called before LLM
    let validator = SqlValidator::new(HashSet::new(), HashSet::new());

    let result = validator.validate("INSERT INTO users VALUES (1, 'test')");
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("Only SELECT"),
        "Expected validation error, got: {}",
        err
    );
}

// ============================================================================
// Validator Integration Tests (no DB needed)
// ============================================================================

#[tokio::test]
async fn test_validator_sql_injection_multi_statement() {
    let validator = SqlValidator::new(HashSet::new(), HashSet::new());

    let result = validator.validate("SELECT * FROM users; DROP TABLE users");
    assert!(result.is_err());
}

#[tokio::test]
async fn test_validator_cte_data_modification() {
    let validator = SqlValidator::new(HashSet::new(), HashSet::new());

    let result = validator.validate(
        "WITH d AS (DELETE FROM users RETURNING *) SELECT * FROM d"
    );
    assert!(result.is_err());
}

#[tokio::test]
async fn test_validator_table_access_in_subquery() {
    let allowed: HashSet<String> = vec!["users".to_string()].into_iter().collect();
    let validator = SqlValidator::new(allowed, HashSet::new());

    let result = validator.validate(
        "SELECT * FROM users WHERE id IN (SELECT user_id FROM orders)"
    );
    assert!(result.is_err(), "Should reject subquery referencing unauthorized table");
}

#[tokio::test]
async fn test_validator_table_access_in_join() {
    let allowed: HashSet<String> = vec!["users".to_string()].into_iter().collect();
    let validator = SqlValidator::new(allowed, HashSet::new());

    let result = validator.validate(
        "SELECT * FROM users JOIN orders ON users.id = orders.user_id"
    );
    assert!(result.is_err(), "Should reject JOIN with unauthorized table");
}

// ============================================================================
// Config Integration Tests
// ============================================================================

#[tokio::test]
async fn test_mask_password_integration() {
    let url = "postgresql://user:p@ss@localhost:5432/db";
    let masked = pg_mcp::config::mask_password(url);
    assert!(!masked.contains("p@ss"));
    assert!(masked.contains("***"));
}
