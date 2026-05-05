use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;

/// Implement this trait to expose a callable tool to the agent.
///
/// Each tool must be `Send + Sync` so it can be held behind a shared
/// reference across async task boundaries.
#[async_trait]
pub trait Tool: Send + Sync {
    /// Unique name used to identify the tool in LLM requests.
    fn name(&self) -> &str;

    /// Human-readable description sent to the LLM so it knows when to call
    /// this tool.
    fn description(&self) -> &str;

    /// JSON Schema object describing the tool's parameters.
    ///
    /// Must be a JSON object with `"type": "object"` and a `"properties"` key.
    fn parameters(&self) -> Value;

    /// Execute the tool with the given arguments (parsed from the LLM's JSON).
    ///
    /// Return `Ok(String)` with the result text, or `Err` if the tool fails.
    /// The agent loop converts errors to tool result messages rather than
    /// aborting, so implementations should return descriptive error messages.
    async fn execute(&self, args: Value) -> Result<String>;
}

/// Serialises a `Tool` into the OpenAI function-calling schema format.
pub fn tool_to_openai_schema(tool: &dyn Tool) -> Value {
    serde_json::json!({
        "type": "function",
        "function": {
            "name": tool.name(),
            "description": tool.description(),
            "parameters": tool.parameters(),
        }
    })
}

/// Holds all registered tools and dispatches calls by name.
pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self { tools: HashMap::new() }
    }

    /// Register a tool. Replaces any existing tool with the same name.
    pub fn register(&mut self, tool: Box<dyn Tool>) {
        self.tools.insert(tool.name().to_string(), tool);
    }

    /// Return the OpenAI schema array for all registered tools.
    pub fn schemas(&self) -> Vec<Value> {
        self.tools.values().map(|t| tool_to_openai_schema(t.as_ref())).collect()
    }

    /// Execute a tool by name. Returns an error string (not `Err`) when the
    /// tool is not found or its execution fails, so the agent loop can forward
    /// the message back to the LLM without aborting.
    pub async fn call(&self, name: &str, args: Value) -> String {
        match self.tools.get(name) {
            None => format!("Error: tool '{}' not found", name),
            Some(tool) => match tool.execute(args).await {
                Ok(result) => result,
                Err(e) => format!("Error: {}", e),
            },
        }
    }

    pub fn is_empty(&self) -> bool {
        self.tools.is_empty()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// Allow `Box<dyn Tool>` to be passed wherever `impl Tool` is expected.
#[async_trait::async_trait]
impl Tool for Box<dyn Tool> {
    fn name(&self) -> &str {
        (**self).name()
    }

    fn description(&self) -> &str {
        (**self).description()
    }

    fn parameters(&self) -> Value {
        (**self).parameters()
    }

    async fn execute(&self, args: Value) -> anyhow::Result<String> {
        (**self).execute(args).await
    }
}
