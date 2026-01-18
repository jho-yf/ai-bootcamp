/// 查询结果模型
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryResult {
    /// 列名列表
    pub columns: Vec<String>,

    /// 行数据（每行是一个 HashMap，key 是列名，value 是列值）
    pub rows: Vec<HashMap<String, Value>>,

    /// 总行数
    pub total: usize,

    /// 执行时间（毫秒）
    pub exec_time_ms: u64,

    /// 执行的 SQL（包含自动添加的 LIMIT）
    pub sql: String,

    /// 是否被截断（超过 LIMIT）
    pub truncated: bool,
}

/// 执行查询请求
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunQueryRequest {
    pub database_id: String,
    pub sql: String,
}
