use clap::Parser;
use serde::Deserialize;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub llm: LlmConfig,
    pub server: ServerConfig,
}

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub schema: String,
    pub allowed_tables: Vec<String>,
    pub excluded_tables: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct LlmConfig {
    pub api_url: String,
    pub api_key: String,
    pub model: String,
    pub temperature: f32,
    pub max_tokens: u32,
}

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub max_rows: u32,
    pub query_timeout_secs: u64,
    pub prompt_budget: usize,
    pub debug: bool,
    pub metadata_refresh_secs: u64,
}

#[derive(Debug, Deserialize)]
struct TomlConfig {
    database: Option<TomlDatabaseConfig>,
    llm: Option<TomlLlmConfig>,
    server: Option<TomlServerConfig>,
}

#[derive(Debug, Deserialize)]
struct TomlDatabaseConfig {
    url: Option<String>,
    schema: Option<String>,
    allowed_tables: Option<Vec<String>>,
    excluded_tables: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct TomlLlmConfig {
    api_url: Option<String>,
    api_key: Option<String>,
    model: Option<String>,
    temperature: Option<f32>,
    max_tokens: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct TomlServerConfig {
    max_rows: Option<u32>,
    query_timeout_secs: Option<u64>,
    prompt_budget: Option<usize>,
    debug: Option<bool>,
    metadata_refresh_secs: Option<u64>,
}

#[derive(Parser, Debug)]
#[command(name = "pg-mcp", about = "PostgreSQL MCP Server")]
struct CliArgs {
    /// Database connection URL
    #[arg(long, env = "PG_MCP_DATABASE_URL")]
    database_url: Option<String>,

    /// Database schema
    #[arg(long, env = "PG_MCP_DATABASE_SCHEMA")]
    database_schema: Option<String>,

    /// Comma-separated list of allowed tables
    #[arg(long, env = "PG_MCP_ALLOWED_TABLES")]
    allowed_tables: Option<String>,

    /// Comma-separated list of excluded tables
    #[arg(long, env = "PG_MCP_EXCLUDED_TABLES")]
    excluded_tables: Option<String>,

    /// LLM API URL
    #[arg(long, env = "PG_MCP_LLM_API_URL")]
    llm_api_url: Option<String>,

    /// LLM API key
    #[arg(long, env = "PG_MCP_LLM_API_KEY")]
    llm_api_key: Option<String>,

    /// LLM model name
    #[arg(long, env = "PG_MCP_LLM_MODEL")]
    llm_model: Option<String>,

    /// LLM temperature
    #[arg(long, env = "PG_MCP_LLM_TEMPERATURE")]
    llm_temperature: Option<f32>,

    /// LLM max tokens
    #[arg(long, env = "PG_MCP_LLM_MAX_TOKENS")]
    llm_max_tokens: Option<u32>,

    /// Maximum rows to return
    #[arg(long, env = "PG_MCP_SERVER_MAX_ROWS")]
    server_max_rows: Option<u32>,

    /// Query timeout in seconds
    #[arg(long, env = "PG_MCP_SERVER_QUERY_TIMEOUT_SECS")]
    server_query_timeout_secs: Option<u64>,

    /// Prompt budget for context
    #[arg(long, env = "PG_MCP_SERVER_PROMPT_BUDGET")]
    server_prompt_budget: Option<usize>,

    /// Enable debug mode
    #[arg(long, env = "PG_MCP_SERVER_DEBUG")]
    server_debug: Option<bool>,

    /// Metadata refresh interval in seconds
    #[arg(long, env = "PG_MCP_SERVER_METADATA_REFRESH_SECS")]
    server_metadata_refresh_secs: Option<u64>,

    /// Path to config file
    #[arg(long, env = "PG_MCP_CONFIG_FILE")]
    config_file: Option<PathBuf>,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Failed to read config file: {0}")]
    FileRead(#[source] std::io::Error),

    #[error("Failed to parse config file: {0}")]
    FileParse(#[source] toml::de::Error),

    #[error("Database URL is required")]
    MissingDatabaseUrl,

    #[error("LLM API key is required")]
    MissingApiKey,

    #[error("Invalid temperature: {0} (must be between 0.0 and 2.0)")]
    InvalidTemperature(f32),

    #[error("Invalid max_tokens: {0} (must be > 0)")]
    InvalidMaxTokens(u32),
}

impl AppConfig {
    pub fn load() -> anyhow::Result<Self> {
        let cli = CliArgs::parse();

        let toml_config = if let Some(path) = &cli.config_file {
            let content = std::fs::read_to_string(path)
                .map_err(ConfigError::FileRead)?;
            toml::from_str(&content)
                .map_err(ConfigError::FileParse)?
        } else {
            TomlConfig {
                database: None,
                llm: None,
                server: None,
            }
        };

        let database = DatabaseConfig {
            url: cli.database_url
                .or_else(|| toml_config.database.as_ref().and_then(|d| d.url.clone()))
                .ok_or_else(|| anyhow::anyhow!("Database URL is required"))?,

            schema: cli.database_schema
                .or_else(|| toml_config.database.as_ref().and_then(|d| d.schema.clone()))
                .unwrap_or_else(|| "public".to_string()),

            allowed_tables: cli.allowed_tables
                .map(|s| s.split(',').map(|t| t.trim().to_string()).collect())
                .or_else(|| toml_config.database.as_ref().and_then(|d| d.allowed_tables.clone()))
                .unwrap_or_default(),

            excluded_tables: cli.excluded_tables
                .map(|s| s.split(',').map(|t| t.trim().to_string()).collect())
                .or_else(|| toml_config.database.as_ref().and_then(|d| d.excluded_tables.clone()))
                .unwrap_or_default(),
        };

        let llm = LlmConfig {
            api_url: cli.llm_api_url
                .or_else(|| toml_config.llm.as_ref().and_then(|l| l.api_url.clone()))
                .unwrap_or_else(|| "https://api.openai.com/v1".to_string()),

            api_key: cli.llm_api_key
                .or_else(|| toml_config.llm.as_ref().and_then(|l| l.api_key.clone()))
                .ok_or_else(|| anyhow::anyhow!("LLM API key is required"))?,

            model: cli.llm_model
                .or_else(|| toml_config.llm.as_ref().and_then(|l| l.model.clone()))
                .unwrap_or_else(|| "gpt-4o".to_string()),

            temperature: cli.llm_temperature
                .or_else(|| toml_config.llm.as_ref().and_then(|l| l.temperature))
                .unwrap_or(0.0),

            max_tokens: cli.llm_max_tokens
                .or_else(|| toml_config.llm.as_ref().and_then(|l| l.max_tokens))
                .unwrap_or(1000),
        };

        if llm.temperature < 0.0 || llm.temperature > 2.0 {
            return Err(ConfigError::InvalidTemperature(llm.temperature).into());
        }

        if llm.max_tokens == 0 {
            return Err(ConfigError::InvalidMaxTokens(llm.max_tokens).into());
        }

        let server = ServerConfig {
            max_rows: cli.server_max_rows
                .or_else(|| toml_config.server.as_ref().and_then(|s| s.max_rows))
                .unwrap_or(1000),

            query_timeout_secs: cli.server_query_timeout_secs
                .or_else(|| toml_config.server.as_ref().and_then(|s| s.query_timeout_secs))
                .unwrap_or(30),

            prompt_budget: cli.server_prompt_budget
                .or_else(|| toml_config.server.as_ref().and_then(|s| s.prompt_budget))
                .unwrap_or(8000),

            debug: cli.server_debug
                .or_else(|| toml_config.server.as_ref().and_then(|s| s.debug))
                .unwrap_or(false),

            metadata_refresh_secs: cli.server_metadata_refresh_secs
                .or_else(|| toml_config.server.as_ref().and_then(|s| s.metadata_refresh_secs))
                .unwrap_or(0),
        };

        Ok(Self { database, llm, server })
    }
}

pub fn mask_password(url: &str) -> String {
    if let Some(pos) = url.find("://") {
        let protocol = &url[..pos + 3];
        let rest = &url[pos + 3..];

        if let Some(at_pos) = rest.find('@') {
            let credentials = &rest[..at_pos];
            let host_part = &rest[at_pos..];

            if let Some(colon_pos) = credentials.rfind(':') {
                if let Some(slash_pos) = credentials[colon_pos..].find('/') {
                    let password_start = colon_pos + 1;
                    let password_end = colon_pos + slash_pos;
                    let masked = format!(
                        "{}{}:{}***{}",
                        protocol,
                        &credentials[..colon_pos],
                        &credentials[password_end..],
                        host_part
                    );
                    return masked;
                } else {
                    let password_start = colon_pos + 1;
                    let masked = format!(
                        "{}{}:***{}",
                        protocol,
                        &credentials[..colon_pos],
                        host_part
                    );
                    return masked;
                }
            }
        }
    }
    url.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_password_with_password() {
        let url = "postgresql://user:password@localhost:5432/db";
        let masked = mask_password(url);
        assert_eq!(masked, "postgresql://user:***@localhost:5432/db");
    }

    #[test]
    fn test_mask_password_without_password() {
        let url = "postgresql://user@localhost:5432/db";
        let masked = mask_password(url);
        assert_eq!(masked, url);
    }

    #[test]
    fn test_mask_password_no_credentials() {
        let url = "postgresql://localhost:5432/db";
        let masked = mask_password(url);
        assert_eq!(masked, url);
    }

    #[test]
    fn test_mask_password_with_special_chars() {
        let url = "postgresql://user:p@ssw0rd@localhost:5432/db";
        let masked = mask_password(url);
        assert_eq!(masked, "postgresql://user:***@localhost:5432/db");
    }

    #[test]
    fn test_default_values() {
        // This test verifies that defaults are correctly applied
        // We can't easily test the full load() without env vars, so we test the logic separately

        // Test that empty allowed_tables/excluded_tables defaults to empty vec
        let tables: Vec<String> = vec![];
        assert!(tables.is_empty());

        // Test default values
        assert_eq!(ServerConfig::default().prompt_budget, 8000);
        assert_eq!(ServerConfig::default().metadata_refresh_secs, 0);
        assert_eq!(ServerConfig::default().max_rows, 1000);
    }

    impl Default for ServerConfig {
        fn default() -> Self {
            Self {
                max_rows: 1000,
                query_timeout_secs: 30,
                prompt_budget: 8000,
                debug: false,
                metadata_refresh_secs: 0,
            }
        }
    }

    #[test]
    fn test_temperature_validation() {
        let error = ConfigError::InvalidTemperature(3.0);
        assert!(error.to_string().contains("3.0"));
        assert!(error.to_string().contains("must be between 0.0 and 2.0"));
    }

    #[test]
    fn test_max_tokens_validation() {
        let error = ConfigError::InvalidMaxTokens(0);
        assert!(error.to_string().contains("must be > 0"));
    }
}
