use crate::config::AppConfig;
use crate::executor::QueryExecutor;
use crate::llm::LlmClient;
use crate::metadata::MetadataCache;
use crate::validator::SqlValidator;
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::tool::Parameters;
use rmcp::model::{ServerInfo, ServerCapabilities, Implementation};
use rmcp::{ServerHandler, ServiceExt};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info};

const MAX_RETRIES: usize = 1;

#[derive(Debug, Clone)]
pub struct PgMcpServer {
    tool_router: ToolRouter<Self>,
    config: Arc<AppConfig>,
    metadata: Arc<MetadataCache>,
    llm: Arc<LlmClient>,
    executor: Arc<QueryExecutor>,
    validator: SqlValidator,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct QueryParams {
    #[schemars(description = "Natural language question about the database")]
    question: String,
}

#[tool_router]
impl PgMcpServer {
    #[tool_router]
    pub fn new(
        config: Arc<AppConfig>,
        metadata: Arc<MetadataCache>,
        llm: Arc<LlmClient>,
        executor: Arc<QueryExecutor>,
        validator: SqlValidator,
    ) -> Self {
        Self {
            tool_router: ToolRouter::new(),
            config,
            metadata,
            llm,
            executor,
            validator,
        }
    }

    #[tool(
        name = "query",
        description = "Execute a natural language query against the PostgreSQL database and return results"
    )]
    async fn query(&self, Parameters(params): Parameters<QueryParams>) -> Result<String, rmcp::ErrorData> {
        info!(question = %params.question, "Received query request");

        // Get database context for the LLM
        let db_context = self
            .metadata
            .get_relevant_context(&params.question, self.config.server.prompt_budget)
            .await;

        let mut last_error: Option<String> = None;

        for attempt in 0..=MAX_RETRIES {
            info!(attempt = attempt, "Attempting to generate and execute SQL");

            // Generate SQL via LLM
            let sql = match self
                .llm
                .generate_sql(&params.question, &db_context, last_error.as_deref())
                .await
            {
                Ok(sql) => sql,
                Err(e) => {
                    error!(error = %e, "Failed to generate SQL");
                    return Err(rmcp::ErrorData::new(
                        rmcp::model::ErrorCode::INTERNAL_ERROR,
                        format!("Failed to generate SQL: {}", e),
                        None,
                    ));
                }
            };

            info!(sql = %sql, "Generated SQL");

            // Validate SQL
            if let Err(e) = self.validator.validate(&sql) {
                error!(error = %e, "SQL validation failed");
                // Don't retry on validation errors - the LLM generated invalid SQL
                return Err(rmcp::ErrorData::new(
                    rmcp::model::ErrorCode::INVALID_PARAMS,
                    format!("SQL validation failed: {}", e),
                    None,
                ));
            }

            // Execute SQL
            match self.executor.execute(&sql).await {
                Ok(result) => {
                    if let Some(error) = &result.error {
                        // Execution failed, may retry
                        if attempt < MAX_RETRIES {
                            info!(error = %error, "Query execution failed, will retry");
                            last_error = Some(error.clone());
                            continue;
                        } else {
                            error!(error = %error, "Query execution failed after retries");
                            return Err(rmcp::ErrorData::new(
                                rmcp::model::ErrorCode::INTERNAL_ERROR,
                                format!("Query execution failed: {}", error),
                                None,
                            ));
                        }
                    }

                    // Success
                    info!(
                        sql = %result.sql,
                        row_count = result.row_count,
                        execution_time_ms = result.execution_time_ms,
                        "Query executed successfully"
                    );

                    let json_result = serde_json::to_string(&result)
                        .map_err(|e| {
                            rmcp::ErrorData::new(
                                rmcp::model::ErrorCode::INTERNAL_ERROR,
                                format!("Failed to serialize result: {}", e),
                                None,
                            )
                        })?;

                    return Ok(json_result);
                }
                Err(e) => {
                    if attempt < MAX_RETRIES {
                        info!(error = %e, "Executor error, will retry");
                        last_error = Some(e.to_string());
                        continue;
                    } else {
                        error!(error = %e, "Executor error after retries");
                        return Err(rmcp::ErrorData::new(
                            rmcp::model::ErrorCode::INTERNAL_ERROR,
                            format!("Query execution error: {}", e),
                            None,
                        ));
                    }
                }
            }
        }

        // Should not reach here
        Err(rmcp::ErrorData::new(
            rmcp::model::ErrorCode::INTERNAL_ERROR,
            "Unexpected error: exhausted retries",
            None,
        ))
    }
}

#[tool_handler]
impl ServerHandler for PgMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            name: "pg-mcp".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            capabilities: ServerCapabilities::default(),
            instructions: Some("PostgreSQL MCP Server - Execute natural language queries against PostgreSQL databases. \
                Use the 'query' tool to ask questions about your data.".to_string()),
            implementation: Some(Implementation {
                name: "pg-mcp".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_params_schema() {
        let params = QueryParams {
            question: "Show all users".to_string(),
        };

        assert_eq!(params.question, "Show all users");
    }

    #[test]
    fn test_server_info() {
        let config = Arc::new(AppConfig {
            database: crate::config::DatabaseConfig {
                url: "postgresql://localhost/test".to_string(),
                schema: "public".to_string(),
                allowed_tables: vec![],
                excluded_tables: vec![],
            },
            llm: crate::config::LlmConfig {
                api_url: "https://api.openai.com/v1".to_string(),
                api_key: "test".to_string(),
                model: "gpt-4o".to_string(),
                temperature: 0.0,
                max_tokens: 1000,
            },
            server: crate::config::ServerConfig {
                max_rows: 1000,
                query_timeout_secs: 30,
                prompt_budget: 8000,
                debug: false,
                metadata_refresh_secs: 0,
            },
        });

        // We can't fully test server creation without a pool, but we can verify the types
        assert_eq!(config.server.max_rows, 1000);
    }
}
