// 注意：Rust 的测试通常放在同一文件的 #[cfg(test)] 模块中
// 这个文件用于存放可能需要共享的测试辅助函数

#[cfg(test)]
pub mod test_helpers {
    use crate::models::query::{QueryResult, RunQueryRequest};
    use std::collections::HashMap;

    /// 创建测试用的 QueryResult
    pub fn create_test_query_result() -> QueryResult {
        QueryResult {
            columns: vec!["id".to_string(), "name".to_string()],
            rows: vec![
                {
                    let mut row = HashMap::new();
                    row.insert("id".to_string(), serde_json::json!(1));
                    row.insert("name".to_string(), serde_json::json!("Alice"));
                    row
                },
                {
                    let mut row = HashMap::new();
                    row.insert("id".to_string(), serde_json::json!(2));
                    row.insert("name".to_string(), serde_json::json!("Bob"));
                    row
                },
            ],
            total: 2,
            exec_time_ms: 45,
            sql: "SELECT * FROM users LIMIT 100".to_string(),
            truncated: false,
        }
    }

    /// 创建测试用的 RunQueryRequest
    pub fn create_test_query_request() -> RunQueryRequest {
        RunQueryRequest {
            database_id: "test-db-1".to_string(),
            sql: "SELECT * FROM users".to_string(),
        }
    }
}
