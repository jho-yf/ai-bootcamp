//! # agent-core
//!
//! A minimal, composable Rust SDK for building multi-turn LLM agents with
//! tool use.
//!
//! ## Quick start
//!
//! ```no_run
//! use agent_core::{Agent, AgentConfig, Tool};
//! use async_trait::async_trait;
//! use anyhow::Result;
//! use serde_json::Value;
//!
//! struct EchoTool;
//!
//! #[async_trait]
//! impl Tool for EchoTool {
//!     fn name(&self) -> &str { "echo" }
//!     fn description(&self) -> &str { "Echoes the input back" }
//!     fn parameters(&self) -> Value {
//!         serde_json::json!({
//!             "type": "object",
//!             "properties": {
//!                 "text": { "type": "string", "description": "Text to echo" }
//!             },
//!             "required": ["text"]
//!         })
//!     }
//!     async fn execute(&self, args: Value) -> Result<String> {
//!         Ok(args["text"].as_str().unwrap_or("").to_string())
//!     }
//! }
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let mut agent = Agent::new(AgentConfig {
//!         model: "gpt-4o".to_string(),
//!         api_key: std::env::var("OPENAI_API_KEY")?,
//!         ..Default::default()
//!     });
//!     agent.add_tool(EchoTool);
//!     let reply = agent.run("Echo 'hello world' for me").await?;
//!     println!("{}", reply);
//!     Ok(())
//! }
//! ```

pub mod agent;
pub mod llm;
pub mod mcp;
pub mod message;
pub mod tool;

pub use agent::{Agent, AgentConfig};
pub use mcp::McpClient;
pub use message::{FunctionCall, Message, ToolCall};
pub use tool::Tool;
