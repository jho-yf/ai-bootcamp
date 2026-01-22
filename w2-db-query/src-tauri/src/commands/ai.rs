/// AI 自然语言查询 Commands
use crate::commands::metadata::get_database_metadata;
use crate::commands::query::run_sql_query;
use crate::models::query::{QueryResult, RunQueryRequest};
use crate::services::{ai_service, cache_service};

/// 从自然语言生成 SQL（不执行）
///
/// # 参数
/// * `database_id` - 数据库连接 ID
/// * `prompt` - 用户的自然语言查询描述
///
/// # 返回
/// 生成的 SQL 查询语句
#[tauri::command]
pub async fn generate_sql_from_nl(database_id: String, prompt: String) -> Result<String, String> {
    // 获取数据库元数据
    let metadata = get_database_metadata(database_id.clone())
        .await
        .map_err(|e| format!("获取数据库元数据失败: {}", e))?;

    // 使用 AI 服务生成 SQL
    let sql = ai_service::generate_sql_from_natural_language(&prompt, &metadata)
        .await
        .map_err(|e| format!("生成 SQL 失败: {}", e))?;

    Ok(sql)
}

/// 执行自然语言查询（生成 SQL 并执行）
///
/// # 参数
/// * `database_id` - 数据库连接 ID
/// * `prompt` - 用户的自然语言查询描述
///
/// # 返回
/// 包含生成的 SQL 和查询结果的响应
#[tauri::command]
pub async fn run_nl_query(database_id: String, prompt: String) -> Result<NLQueryResponse, String> {
    // 生成 SQL
    let generated_sql = generate_sql_from_nl(database_id.clone(), prompt.clone())
        .await
        .map_err(|e| format!("生成 SQL 失败: {}", e))?;

    // 执行查询
    let request = RunQueryRequest {
        database_id: database_id.clone(),
        sql: generated_sql.clone(),
    };

    let result = run_sql_query(request)
        .await
        .map_err(|e| format!("执行查询失败: {}", e))?;

    let response = NLQueryResponse {
        generated_sql: generated_sql.clone(),
        result: result.clone(),
    };

    // 保存查询历史（自然语言查询类型）
    let _ = cache_service::save_query_history(
        &database_id,
        "natural_language",
        Some(&generated_sql),
        Some(&prompt),
        Some(result.exec_time_ms),
        "success",
    );

    Ok(response)
}

/// 自然语言查询响应
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NLQueryResponse {
    /// 生成的 SQL 查询
    pub generated_sql: String,
    /// 查询结果
    pub result: QueryResult,
}
