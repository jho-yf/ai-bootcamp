//! Minimal MCP (Model Context Protocol) client over stdio JSON-RPC 2.0.
//!
//! Spawns a child process (the MCP server), communicates via stdin/stdout
//! using newline-delimited JSON-RPC 2.0, and wraps each discovered tool as
//! a [`crate::tool::Tool`] implementation.

use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::sync::Mutex;

use crate::tool::Tool;

// ---------------------------------------------------------------------------
// JSON-RPC 2.0 wire types
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
struct JsonRpcRequest {
    jsonrpc: &'static str,
    id: u64,
    method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<Value>,
}

#[derive(Debug, Deserialize)]
struct JsonRpcResponse {
    #[allow(dead_code)]
    id: Option<Value>,
    result: Option<Value>,
    error: Option<Value>,
}

// ---------------------------------------------------------------------------
// MCP tool descriptor (from tools/list)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, Clone)]
pub struct McpToolInfo {
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "inputSchema")]
    pub input_schema: Option<Value>,
}

// ---------------------------------------------------------------------------
// Shared transport state
// ---------------------------------------------------------------------------

struct Transport {
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    next_id: u64,
}

impl Transport {
    async fn send_request(&mut self, method: &str, params: Option<Value>) -> Result<Value> {
        let id = self.next_id;
        self.next_id += 1;

        let req = JsonRpcRequest {
            jsonrpc: "2.0",
            id,
            method: method.to_string(),
            params,
        };

        let mut line = serde_json::to_string(&req).context("Serialize JSON-RPC request")?;
        line.push('\n');

        self.stdin
            .write_all(line.as_bytes())
            .await
            .context("Write to MCP stdin")?;
        self.stdin.flush().await.context("Flush MCP stdin")?;

        // Read lines until we get a response matching our id (skip notifications).
        loop {
            let mut buf = String::new();
            let n = self
                .stdout
                .read_line(&mut buf)
                .await
                .context("Read from MCP stdout")?;
            if n == 0 {
                return Err(anyhow!("MCP server closed stdout unexpectedly"));
            }
            let trimmed = buf.trim();
            if trimmed.is_empty() {
                continue;
            }

            let resp: JsonRpcResponse =
                serde_json::from_str(trimmed).context("Parse JSON-RPC response")?;

            if let Some(err) = resp.error {
                return Err(anyhow!("MCP error: {}", err));
            }

            return resp.result.ok_or_else(|| anyhow!("JSON-RPC response has no result"));
        }
    }
}

// ---------------------------------------------------------------------------
// McpClient
// ---------------------------------------------------------------------------

/// Manages a connection to a single MCP server process.
pub struct McpClient {
    transport: Arc<Mutex<Transport>>,
    #[allow(dead_code)]
    child: Child,
}

impl McpClient {
    /// Spawn the MCP server and perform the `initialize` handshake.
    pub async fn connect(program: &str, args: &[&str]) -> Result<Self> {
        let mut child = Command::new(program)
            .args(args)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::inherit())
            .spawn()
            .with_context(|| format!("Failed to spawn MCP server: {} {:?}", program, args))?;

        let stdin = child.stdin.take().ok_or_else(|| anyhow!("No stdin on MCP child"))?;
        let stdout = child.stdout.take().ok_or_else(|| anyhow!("No stdout on MCP child"))?;

        let mut transport = Transport {
            stdin,
            stdout: BufReader::new(stdout),
            next_id: 1,
        };

        // MCP initialize handshake
        let init_params = serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "agent-core",
                "version": "0.1.0"
            }
        });
        transport
            .send_request("initialize", Some(init_params))
            .await
            .context("MCP initialize failed")?;

        // Send initialized notification (no response expected)
        let notif = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized",
            "params": {}
        });
        let mut line = serde_json::to_string(&notif)?;
        line.push('\n');
        transport.stdin.write_all(line.as_bytes()).await?;
        transport.stdin.flush().await?;

        Ok(Self {
            transport: Arc::new(Mutex::new(transport)),
            child,
        })
    }

    /// List all tools exposed by the MCP server.
    pub async fn list_tools(&self) -> Result<Vec<McpToolInfo>> {
        let mut t = self.transport.lock().await;
        let result = t.send_request("tools/list", None).await?;
        let tools: Vec<McpToolInfo> = serde_json::from_value(
            result
                .get("tools")
                .cloned()
                .unwrap_or(Value::Array(vec![])),
        )
        .context("Parse tools/list response")?;
        Ok(tools)
    }

    /// Call a tool on the MCP server and return its text result.
    pub async fn call_tool(&self, name: &str, arguments: Value) -> Result<String> {
        let params = serde_json::json!({
            "name": name,
            "arguments": arguments,
        });
        let mut t = self.transport.lock().await;
        let result = t.send_request("tools/call", Some(params)).await?;

        // MCP tools/call returns { content: [ { type: "text", text: "..." } ] }
        let content = result.get("content").cloned().unwrap_or(Value::Array(vec![]));
        let mut parts: Vec<String> = Vec::new();
        if let Some(arr) = content.as_array() {
            for item in arr {
                if let Some(text) = item.get("text").and_then(|v| v.as_str()) {
                    parts.push(text.to_string());
                }
            }
        }
        if parts.is_empty() {
            Ok(serde_json::to_string(&result)?)
        } else {
            Ok(parts.join("\n"))
        }
    }

    /// Build `Tool` wrappers for every tool the MCP server exposes.
    pub async fn into_tools(self) -> Result<Vec<Box<dyn Tool>>> {
        let tools_info = self.list_tools().await?;
        let transport = Arc::clone(&self.transport);

        let tools: Vec<Box<dyn Tool>> = tools_info
            .into_iter()
            .map(|info| {
                let t: Box<dyn Tool> = Box::new(McpTool {
                    info,
                    transport: Arc::clone(&transport),
                });
                t
            })
            .collect();

        // Intentionally leak the child — it will be cleaned up when the
        // process exits. For production use you'd want to track it.
        std::mem::forget(self.child);

        Ok(tools)
    }
}

// ---------------------------------------------------------------------------
// McpTool — wraps a single MCP tool as a Tool impl
// ---------------------------------------------------------------------------

struct McpTool {
    info: McpToolInfo,
    transport: Arc<Mutex<Transport>>,
}

#[async_trait]
impl Tool for McpTool {
    fn name(&self) -> &str {
        &self.info.name
    }

    fn description(&self) -> &str {
        self.info.description.as_deref().unwrap_or("")
    }

    fn parameters(&self) -> Value {
        self.info.input_schema.clone().unwrap_or_else(|| {
            serde_json::json!({ "type": "object", "properties": {} })
        })
    }

    async fn execute(&self, args: Value) -> Result<String> {
        let params = serde_json::json!({
            "name": self.info.name,
            "arguments": args,
        });
        let mut t = self.transport.lock().await;
        let result = t.send_request("tools/call", Some(params)).await?;

        let content = result.get("content").cloned().unwrap_or(Value::Array(vec![]));
        let mut parts: Vec<String> = Vec::new();
        if let Some(arr) = content.as_array() {
            for item in arr {
                if let Some(text) = item.get("text").and_then(|v| v.as_str()) {
                    parts.push(text.to_string());
                }
            }
        }
        if parts.is_empty() {
            Ok(serde_json::to_string(&result)?)
        } else {
            Ok(parts.join("\n"))
        }
    }
}
