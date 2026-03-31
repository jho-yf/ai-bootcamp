mod config;
mod executor;
mod llm;
mod metadata;
mod server;
mod validator;

use config::AppConfig;
use executor::QueryExecutor;
use llm::LlmClient;
use metadata::MetadataCache;
use rmcp::ServiceExt;
use server::PgMcpServer;
use sqlx::postgres::PgPoolOptions;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info};
use validator::SqlValidator;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing subscriber
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    // Load configuration
    let config = AppConfig::load()?;

    // Log configuration with masked password
    let masked_url = config::mask_password(&config.database.url);
    info!(
        database_url = %masked_url,
        database_schema = %config.database.schema,
        llm_model = %config.llm.model,
        max_rows = config.server.max_rows,
        query_timeout_secs = config.server.query_timeout_secs,
        "Configuration loaded"
    );

    // Create PostgreSQL connection pool
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(5))
        .connect(&config.database.url)
        .await?;

    info!("Connected to PostgreSQL");

    // Create and load metadata cache
    let excluded_tables: HashSet<String> = config
        .database
        .excluded_tables
        .iter()
        .map(|t| t.to_lowercase())
        .collect();

    let metadata = Arc::new(MetadataCache::new(
        pool.clone(),
        config.database.schema.clone(),
        excluded_tables,
    ));

    metadata.load().await?;

    let table_count = metadata.table_count().await;
    let view_count = metadata.view_count().await;
    info!(table_count = table_count, view_count = view_count, "Metadata loaded");

    // Start metadata refresh loop if configured
    let refresh_handle = if config.server.metadata_refresh_secs > 0 {
        let handle = metadata.start_refresh_loop(config.server.metadata_refresh_secs);
        info!(
            refresh_interval_secs = config.server.metadata_refresh_secs,
            "Metadata refresh loop started"
        );
        Some(handle)
    } else {
        None
    };

    // Create components
    let llm = Arc::new(LlmClient::new(&config.llm));
    let executor = Arc::new(QueryExecutor::new(
        pool.clone(),
        config.server.max_rows,
        config.server.query_timeout_secs,
        config.server.debug,
    ));

    let allowed_tables: HashSet<String> = config
        .database
        .allowed_tables
        .iter()
        .map(|t| t.to_lowercase())
        .collect();

    let excluded_tables: HashSet<String> = config
        .database
        .excluded_tables
        .iter()
        .map(|t| t.to_lowercase())
        .collect();

    let validator = SqlValidator::new(allowed_tables, excluded_tables);

    // Create MCP server
    let server = PgMcpServer::new(
        Arc::new(config.clone()),
        metadata,
        llm,
        executor,
        validator,
    );

    info!("Starting MCP server on stdio transport");

    // Create stdio transport
    let transport = (tokio::io::stdin(), tokio::io::stdout());

    // Start the server
    let service = server.serve(transport).await?;

    info!("MCP server started");

    // Wait for shutdown
    service.waiting().await?;

    info!("MCP server shutting down");

    // Abort refresh loop if running
    if let Some(handle) = refresh_handle {
        handle.abort();
        info!("Metadata refresh loop stopped");
    }

    // Close database pool
    pool.close().await;

    info!("PostgreSQL connection pool closed");

    Ok(())
}
