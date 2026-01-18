/// AI 服务：集成 OpenAI API，实现自然语言到 SQL 的转换
use crate::models::metadata::DatabaseMetadata;
use crate::utils::config::{get_openai_api_base, get_openai_api_key, get_openai_model};
use crate::utils::error::AppError;
use async_openai::{
    config::OpenAIConfig,
    types::chat::{
        ChatCompletionRequestMessage, ChatCompletionRequestSystemMessage,
        ChatCompletionRequestUserMessage, CreateChatCompletionRequestArgs,
    },
    Client,
};

/// 从自然语言生成 SQL 查询
///
/// # 参数
/// * `prompt` - 用户的自然语言查询描述
/// * `metadata` - 数据库元数据（表、视图、列信息）
///
/// # 返回
/// 生成的 SQL 查询语句
pub async fn generate_sql_from_natural_language(
    prompt: &str,
    metadata: &DatabaseMetadata,
) -> Result<String, AppError> {
    // 获取 OpenAI API Key、Base URL 和模型名称
    let api_key = get_openai_api_key()?;
    let api_base = get_openai_api_base();
    let model = get_openai_model();

    // 创建 OpenAI 客户端（使用配置对象设置 API key 和 endpoint）
    let config = OpenAIConfig::new()
        .with_api_key(api_key)
        .with_api_base(api_base);
    let client = Client::with_config(config);

    // 构建 Prompt
    let system_prompt = build_system_prompt();
    let user_prompt = build_user_prompt(prompt, metadata);

    // 构建消息列表
    let messages = vec![
        ChatCompletionRequestMessage::System(ChatCompletionRequestSystemMessage {
            content: system_prompt.into(),
            ..Default::default()
        }),
        ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage {
            content: user_prompt.into(),
            ..Default::default()
        }),
    ];

    // 调用 OpenAI API
    let request = CreateChatCompletionRequestArgs::default()
        .model(&model) // 使用配置的模型名称
        .messages(messages)
        .temperature(0.0) // 使用确定性输出
        .max_completion_tokens(500u32) // SQL 查询通常不需要太多 tokens
        .build()
        .map_err(|e| AppError::AIService(format!("构建请求失败: {}", e)))?;

    let response = client
        .chat()
        .create(request)
        .await
        .map_err(|e| AppError::AIService(format!("调用 OpenAI API 失败: {}", e)))?;

    // 提取生成的 SQL
    let sql = response
        .choices
        .first()
        .and_then(|choice| choice.message.content.as_ref())
        .map(|s| s.as_str())
        .unwrap_or("");

    if sql.is_empty() {
        return Err(AppError::AIService("OpenAI 响应中没有生成 SQL".to_string()));
    }

    // 清理 SQL（移除可能的 markdown 代码块标记）
    let sql = clean_sql_output(sql);

    Ok(sql)
}

/// 构建系统 Prompt（定义 AI 的角色和行为）
fn build_system_prompt() -> String {
    r#"你是一个专业的 PostgreSQL 数据库查询生成助手。你的任务是将用户的自然语言描述转换为有效的 PostgreSQL SQL 查询。

规则：
1. 只生成 SELECT 查询，不要生成 DDL 语句（CREATE/DROP/ALTER）
2. 如果查询没有 LIMIT 子句，自动添加 LIMIT 100（限制返回行数）
3. 生成的 SQL 必须符合 PostgreSQL 语法规范
4. 只返回 SQL 查询语句，不要返回其他解释文字
5. 使用提供的数据库表结构信息来构建准确的查询
6. 注意列名和表名的大小写敏感性
7. 对于中文查询，正确理解用户意图并生成对应的 SQL

输出格式：
只输出 SQL 查询语句，不要使用 markdown 代码块，不要添加任何注释或解释。"#.to_string()
}

/// 构建用户 Prompt（包含数据库结构和用户查询）
fn build_user_prompt(prompt: &str, metadata: &DatabaseMetadata) -> String {
    // 格式化元数据为文本描述
    let schema_description = format_metadata(metadata);

    format!(
        r#"数据库结构信息：

{}

用户查询需求：
{}

请根据上述数据库结构，将用户的自然语言查询转换为 PostgreSQL SQL 查询语句。只返回 SQL 查询，不要添加任何其他内容。"#,
        schema_description, prompt
    )
}

/// 格式化数据库元数据为可读的文本描述
fn format_metadata(metadata: &DatabaseMetadata) -> String {
    let mut parts = Vec::new();

    // 格式化表信息
    if !metadata.tables.is_empty() {
        parts.push("表 (Tables):".to_string());
        for table in &metadata.tables {
            let schema_prefix = if table.schema != "public" {
                format!("{}.", table.schema)
            } else {
                String::new()
            };
            parts.push(format!(
                "  - {}{} ({})",
                schema_prefix, table.name, table.table_type
            ));

            // 列出所有列
            if !table.columns.is_empty() {
                let column_list: Vec<String> = table
                    .columns
                    .iter()
                    .map(|col| {
                        let mut col_desc = format!("{} ({})", col.name, col.data_type);
                        if col.is_primary_key {
                            col_desc.push_str(" [PRIMARY KEY]");
                        }
                        if !col.nullable {
                            col_desc.push_str(" [NOT NULL]");
                        }
                        col_desc
                    })
                    .collect();
                parts.push(format!("    列: {}", column_list.join(", ")));
            }

            // 列出主键
            if !table.primary_keys.is_empty() {
                parts.push(format!("    主键: {}", table.primary_keys.join(", ")));
            }

            // 列出外键
            if !table.foreign_keys.is_empty() {
                let fk_list: Vec<String> = table
                    .foreign_keys
                    .iter()
                    .map(|fk| {
                        format!(
                            "{}.{} -> {}.{}",
                            table.name, fk.column_name, fk.referenced_table, fk.referenced_column
                        )
                    })
                    .collect();
                parts.push(format!("    外键: {}", fk_list.join(", ")));
            }
        }
    }

    // 格式化视图信息
    if !metadata.views.is_empty() {
        parts.push("\n视图 (Views):".to_string());
        for view in &metadata.views {
            let schema_prefix = if view.schema != "public" {
                format!("{}.", view.schema)
            } else {
                String::new()
            };
            parts.push(format!("  - {}{}", schema_prefix, view.name));

            if !view.columns.is_empty() {
                let column_list: Vec<String> = view
                    .columns
                    .iter()
                    .map(|col| format!("{} ({})", col.name, col.data_type))
                    .collect();
                parts.push(format!("    列: {}", column_list.join(", ")));
            }
        }
    }

    if parts.is_empty() {
        "数据库中没有找到表或视图。".to_string()
    } else {
        parts.join("\n")
    }
}

/// 清理 SQL 输出（移除 markdown 代码块标记）
fn clean_sql_output(sql: &str) -> String {
    let sql = sql.trim();

    // 移除 markdown 代码块标记
    let sql = if sql.starts_with("```sql") {
        sql.strip_prefix("```sql")
            .and_then(|s| s.strip_suffix("```"))
            .unwrap_or(sql)
            .trim()
    } else if sql.starts_with("```") {
        sql.strip_prefix("```")
            .and_then(|s| s.strip_suffix("```"))
            .unwrap_or(sql)
            .trim()
    } else {
        sql
    };

    // 移除行尾分号（可选，但保持一致性）
    let sql = sql.trim_end_matches(';').trim();

    sql.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::metadata::{ColumnInfo, TableInfo};

    #[test]
    fn test_format_metadata() {
        let metadata = DatabaseMetadata {
            connection_id: "test".to_string(),
            tables: vec![TableInfo {
                schema: "public".to_string(),
                name: "users".to_string(),
                table_type: "BASE TABLE".to_string(),
                columns: vec![
                    ColumnInfo {
                        name: "id".to_string(),
                        data_type: "integer".to_string(),
                        nullable: false,
                        default_value: None,
                        is_primary_key: true,
                        ordinal_position: 1,
                    },
                    ColumnInfo {
                        name: "name".to_string(),
                        data_type: "varchar".to_string(),
                        nullable: false,
                        default_value: None,
                        is_primary_key: false,
                        ordinal_position: 2,
                    },
                ],
                primary_keys: vec!["id".to_string()],
                foreign_keys: vec![],
            }],
            views: vec![],
            extracted_at: chrono::Utc::now(),
        };

        let formatted = format_metadata(&metadata);
        assert!(formatted.contains("users"));
        assert!(formatted.contains("id (integer)"));
        assert!(formatted.contains("name (varchar)"));
    }

    #[test]
    fn test_clean_sql_output() {
        assert_eq!(
            clean_sql_output("SELECT * FROM users"),
            "SELECT * FROM users"
        );
        assert_eq!(
            clean_sql_output("```sql\nSELECT * FROM users\n```"),
            "SELECT * FROM users"
        );
        assert_eq!(
            clean_sql_output("```\nSELECT * FROM users\n```"),
            "SELECT * FROM users"
        );
        assert_eq!(
            clean_sql_output("SELECT * FROM users;"),
            "SELECT * FROM users"
        );
    }
}
