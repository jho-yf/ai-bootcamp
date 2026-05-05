use agent_core::{Agent, AgentConfig, McpClient};
use anyhow::Result;

/// This example connects to the `@modelcontextprotocol/server-filesystem` MCP
/// server via stdio, discovers its tools, and uses them through the agent.
///
/// Prerequisites:
///   npm install -g @modelcontextprotocol/server-filesystem
///
/// Run:
///   OPENAI_API_KEY=sk-... cargo run -p mcp_example
#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let api_key = std::env::var("OPENAI_API_KEY")
        .expect("OPENAI_API_KEY environment variable must be set");

    let api_base = std::env::var("OPENAI_API_BASE")
        .unwrap_or_else(|_| "https://api.openai.com".to_string());

    let model = std::env::var("OPENAI_MODEL")
        .unwrap_or_else(|_| "gpt-4o".to_string());

    // The MCP server command — serves the /tmp directory.
    let mcp_program = "npx";
    let mcp_args = ["-y", "@modelcontextprotocol/server-filesystem", "/tmp"];

    println!("Connecting to MCP filesystem server (serving /tmp)...");
    let mcp_client = McpClient::connect(mcp_program, &mcp_args).await?;

    let tools = mcp_client.into_tools().await?;
    println!("Discovered {} MCP tool(s)", tools.len());
    for t in &tools {
        println!("  - {}: {}", t.name(), t.description());
    }
    println!();

    let mut agent = Agent::new(AgentConfig {
        model,
        system_prompt: "You are a helpful assistant with access to filesystem tools \
                        provided by an MCP server. Use them to explore and read files."
            .to_string(),
        max_steps: 20,
        api_base,
        api_key,
    });

    for tool in tools {
        agent.add_tool(tool);
    }

    let question = "List the files in /tmp and read the content of any text files you find there.";
    println!("User: {}", question);
    println!();

    let reply = agent.run(question).await?;
    println!("Assistant: {}", reply);

    Ok(())
}
