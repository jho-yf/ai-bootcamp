use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;
use tracing::{debug, info};

#[derive(Debug, Clone)]
pub struct LlmClient {
    http: Client,
    api_url: String,
    api_key: String,
    model: String,
    temperature: f32,
    max_tokens: u32,
}

#[derive(Debug, Error)]
pub enum LlmError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("API error: {0}")]
    ApiError(String),

    #[error("Failed to extract SQL from response")]
    ExtractionError,

    #[error("Empty response from LLM")]
    EmptyResponse,
}

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
    max_tokens: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatMessageResponse,
}

#[derive(Debug, Deserialize)]
struct ChatMessageResponse {
    content: String,
}

impl LlmClient {
    pub fn new(config: &crate::config::LlmConfig) -> Self {
        let http = Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            http,
            api_url: config.api_url.clone(),
            api_key: config.api_key.clone(),
            model: config.model.clone(),
            temperature: config.temperature,
            max_tokens: config.max_tokens,
        }
    }

    pub async fn generate_sql(
        &self,
        question: &str,
        db_context: &str,
        last_error: Option<&str>,
    ) -> anyhow::Result<String> {
        info!(question = %question, "Generating SQL via LLM");

        let system_prompt = self.build_system_prompt(db_context);
        let user_prompt = self.build_user_prompt(question, last_error);

        debug!(system_prompt = %system_prompt, "System prompt");
        debug!(user_prompt = %user_prompt, "User prompt");

        let request = ChatRequest {
            model: self.model.clone(),
            messages: vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: system_prompt,
                },
                ChatMessage {
                    role: "user".to_string(),
                    content: user_prompt,
                },
            ],
            temperature: self.temperature,
            max_tokens: self.max_tokens,
        };

        let url = format!(
            "{}/chat/completions",
            self.api_url.trim_end_matches('/')
        );

        debug!(url = %url, "Sending request to LLM API");

        let response = self
            .http
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(LlmError::ApiError(format!("Status {}: {}", status, body)).into());
        }

        let chat_response: ChatResponse = response.json().await?;

        let content = chat_response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .ok_or_else(|| LlmError::EmptyResponse)?;

        debug!(response = %content, "LLM response");

        let sql = self.extract_sql(&content)?;

        info!(sql = %sql, "Extracted SQL");

        Ok(sql)
    }

    fn build_system_prompt(&self, db_context: &str) -> String {
        format!(
            r#"You are a SQL expert. Your job is to convert natural language questions into PostgreSQL SELECT queries.

Database context:
{}

Rules:
1. Generate ONLY a single SELECT statement. No INSERT, UPDATE, DELETE, CREATE, DROP, or other modifications.
2. Return ONLY the SQL query, no explanations or markdown formatting.
3. If you cannot answer the question with the available tables, return "ERROR: Cannot answer with available data".
4. For date/time questions, use PostgreSQL's date functions (NOW(), CURRENT_DATE, etc.).
5. Use proper table and column names from the schema above.
6. Do not use FOR UPDATE, FOR SHARE, or any locking clauses.
7. Always include appropriate WHERE clauses to filter results efficiently.

Important: Return ONLY the SQL, nothing else."#,
            db_context
        )
    }

    fn build_user_prompt(&self, question: &str, last_error: Option<&str>) -> String {
        if let Some(error) = last_error {
            format!(
                "The previous SQL query failed with this error:\n{}\n\nOriginal question: {}\n\nPlease fix the SQL and try again.",
                error, question
            )
        } else {
            question.to_string()
        }
    }

    fn extract_sql(&self, content: &str) -> Result<String, LlmError> {
        let trimmed = content.trim();

        // Check if it starts with ERROR:
        if trimmed.starts_with("ERROR:") {
            return Err(LlmError::ApiError(trimmed.to_string()));
        }

        // Try to extract from ```sql ... ``` block
        if let Some(sql) = self.extract_code_block(trimmed, "sql") {
            return Ok(sql);
        }

        // Try to extract from ``` ... ``` block (no language specified)
        if let Some(sql) = self.extract_code_block(trimmed, "") {
            return Ok(sql);
        }

        // Otherwise, treat the entire content as SQL
        let sql = trimmed.trim().to_string();

        if sql.is_empty() {
            return Err(LlmError::ExtractionError);
        }

        Ok(sql)
    }

    fn extract_code_block(&self, content: &str, lang: &str) -> Option<String> {
        let opening = if lang.is_empty() {
            "```"
        } else {
            &format!("```{}", lang)
        };

        let start = content.find(opening)?;
        let rest = &content[start + opening.len()..];

        let end = rest.find("```")?;
        let sql = rest[..end].trim();

        if sql.is_empty() {
            None
        } else {
            Some(sql.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::LlmConfig;

    fn create_test_client() -> LlmClient {
        LlmClient::new(&LlmConfig {
            api_url: "https://api.openai.com/v1".to_string(),
            api_key: "test-key".to_string(),
            model: "gpt-4o".to_string(),
            temperature: 0.0,
            max_tokens: 1000,
        })
    }

    #[test]
    fn test_extract_sql_from_sql_code_block() {
        let client = create_test_client();
        let content = r#"```sql
SELECT * FROM users
```"#;

        let sql = client.extract_sql(content).unwrap();
        assert_eq!(sql, "SELECT * FROM users");
    }

    #[test]
    fn test_extract_sql_from_plain_code_block() {
        let client = create_test_client();
        let content = r#"```
SELECT * FROM users
```"#;

        let sql = client.extract_sql(content).unwrap();
        assert_eq!(sql, "SELECT * FROM users");
    }

    #[test]
    fn test_extract_sql_bare() {
        let client = create_test_client();
        let content = "SELECT * FROM users";

        let sql = client.extract_sql(content).unwrap();
        assert_eq!(sql, "SELECT * FROM users");
    }

    #[test]
    fn test_extract_sql_with_whitespace() {
        let client = create_test_client();
        let content = r#"```sql
  SELECT * FROM users
```"#;

        let sql = client.extract_sql(content).unwrap();
        assert_eq!(sql, "SELECT * FROM users");
    }

    #[test]
    fn test_extract_sql_error_response() {
        let client = create_test_client();
        let content = "ERROR: Cannot answer with available data";

        let result = client.extract_sql(content);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_sql_empty() {
        let client = create_test_client();
        let content = "";

        let result = client.extract_sql(content);
        assert!(result.is_err());
    }

    #[test]
    fn test_build_system_prompt() {
        let client = create_test_client();
        let db_context = "Table: users\n| id | integer |";

        let prompt = client.build_system_prompt(db_context);

        assert!(prompt.contains("Table: users"));
        assert!(prompt.contains("ONLY a single SELECT statement"));
        assert!(prompt.contains("Return ONLY the SQL query"));
    }

    #[test]
    fn test_build_user_prompt_without_error() {
        let client = create_test_client();
        let question = "Show all users";

        let prompt = client.build_user_prompt(question, None);

        assert_eq!(prompt, "Show all users");
    }

    #[test]
    fn test_build_user_prompt_with_error() {
        let client = create_test_client();
        let question = "Show all users";
        let error = "Table 'users' not found";

        let prompt = client.build_user_prompt(question, Some(error));

        assert!(prompt.contains("previous SQL query failed"));
        assert!(prompt.contains(error));
        assert!(prompt.contains(question));
    }

    #[test]
    fn test_extract_sql_with_extra_text() {
        let client = create_test_client();
        let content = r#"Here's the SQL:

```sql
SELECT * FROM users WHERE id = 1
```

This query will return a single user."#;

        let sql = client.extract_sql(content).unwrap();
        assert_eq!(sql, "SELECT * FROM users WHERE id = 1");
    }
}
