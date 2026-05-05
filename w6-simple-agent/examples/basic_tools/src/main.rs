use agent_core::{Agent, AgentConfig, Tool};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;
use std::fs;

// ---------------------------------------------------------------------------
// read_file tool
// ---------------------------------------------------------------------------

struct ReadFileTool;

#[async_trait]
impl Tool for ReadFileTool {
    fn name(&self) -> &str {
        "read_file"
    }

    fn description(&self) -> &str {
        "Read the contents of a file at the given path"
    }

    fn parameters(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Absolute or relative path to the file"
                }
            },
            "required": ["path"]
        })
    }

    async fn execute(&self, args: Value) -> Result<String> {
        let path = args["path"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("'path' argument is required"))?;
        let contents = fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("Failed to read '{}': {}", path, e))?;
        Ok(contents)
    }
}

// ---------------------------------------------------------------------------
// list_directory tool
// ---------------------------------------------------------------------------

struct ListDirectoryTool;

#[async_trait]
impl Tool for ListDirectoryTool {
    fn name(&self) -> &str {
        "list_directory"
    }

    fn description(&self) -> &str {
        "List the files and directories inside a given directory path"
    }

    fn parameters(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the directory to list"
                }
            },
            "required": ["path"]
        })
    }

    async fn execute(&self, args: Value) -> Result<String> {
        let path = args["path"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("'path' argument is required"))?;

        let entries = fs::read_dir(path)
            .map_err(|e| anyhow::anyhow!("Failed to list '{}': {}", path, e))?;

        let mut names: Vec<String> = entries
            .filter_map(|e| e.ok())
            .map(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                if e.path().is_dir() {
                    format!("{}/", name)
                } else {
                    name
                }
            })
            .collect();

        names.sort();
        Ok(names.join("\n"))
    }
}

// ---------------------------------------------------------------------------
// main
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let api_key = std::env::var("OPENAI_API_KEY")
        .expect("OPENAI_API_KEY environment variable must be set");

    let api_base = std::env::var("OPENAI_API_BASE")
        .unwrap_or_else(|_| "https://api.openai.com".to_string());

    let model = std::env::var("OPENAI_MODEL")
        .unwrap_or_else(|_| "gpt-4o".to_string());

    let mut agent = Agent::new(AgentConfig {
        model,
        system_prompt: "You are a helpful assistant with access to filesystem tools. \
                        Use them to answer questions about files and directories."
            .to_string(),
        max_steps: 20,
        api_base,
        api_key,
    });

    agent.add_tool(ReadFileTool);
    agent.add_tool(ListDirectoryTool);

    let question = "List the files in /tmp and tell me what you see";
    println!("User: {}", question);
    println!();

    let reply = agent.run(question).await?;
    println!("Assistant: {}", reply);

    Ok(())
}
