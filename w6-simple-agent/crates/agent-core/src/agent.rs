use anyhow::{anyhow, Result};
use tracing::{debug, info, warn};

use crate::llm::LlmClient;
use crate::message::Message;
use crate::tool::{Tool, ToolRegistry};

/// Configuration for an [`Agent`].
pub struct AgentConfig {
    /// Model identifier, e.g. `"gpt-4o"` or `"claude-3-5-sonnet-20241022"`.
    pub model: String,
    /// System prompt injected as the first message in every request.
    pub system_prompt: String,
    /// Maximum number of LLM turns before the loop is forcibly terminated.
    /// Prevents runaway tool loops. Defaults to 50.
    pub max_steps: usize,
    /// Base URL of the OpenAI-compatible API, e.g. `"https://api.openai.com"`.
    pub api_base: String,
    /// API key sent as a Bearer token.
    pub api_key: String,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            model: "gpt-4o".to_string(),
            system_prompt: "You are a helpful assistant.".to_string(),
            max_steps: 50,
            api_base: "https://api.openai.com".to_string(),
            api_key: String::new(),
        }
    }
}

/// Multi-turn agent that maintains conversation history across `run()` calls.
///
/// # Example
/// ```no_run
/// # use agent_core::{Agent, AgentConfig};
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// let mut agent = Agent::new(AgentConfig {
///     model: "gpt-4o".to_string(),
///     api_key: std::env::var("OPENAI_API_KEY").unwrap(),
///     ..Default::default()
/// });
/// let reply = agent.run("Hello!").await?;
/// println!("{}", reply);
/// # Ok(())
/// # }
/// ```
pub struct Agent {
    config: AgentConfig,
    registry: ToolRegistry,
    /// Accumulated conversation history (excludes the system prompt, which is
    /// prepended on every request).
    messages: Vec<Message>,
    llm: LlmClient,
}

impl Agent {
    /// Create a new agent with the given configuration and no tools.
    pub fn new(config: AgentConfig) -> Self {
        let llm = LlmClient::new(&config.api_base, &config.api_key);
        Self {
            config,
            registry: ToolRegistry::new(),
            messages: Vec::new(),
            llm,
        }
    }

    /// Register a tool. Returns `&mut Self` for chaining.
    pub fn add_tool(&mut self, tool: impl Tool + 'static) -> &mut Self {
        self.registry.register(Box::new(tool));
        self
    }

    /// Run the agent with a new user message.
    ///
    /// Appends the user message to the conversation history, then enters the
    /// tool-call loop until the LLM returns `finish_reason == "stop"` or
    /// `max_steps` is reached.
    ///
    /// Returns the final assistant text response.
    pub async fn run(&mut self, user_message: &str) -> Result<String> {
        self.messages.push(Message::user(user_message));

        let tool_schemas = if self.registry.is_empty() {
            vec![]
        } else {
            self.registry.schemas()
        };

        let mut steps = 0;

        loop {
            if steps >= self.config.max_steps {
                warn!("Agent reached max_steps ({}), terminating loop", self.config.max_steps);
                return Err(anyhow!(
                    "Agent exceeded maximum steps ({})",
                    self.config.max_steps
                ));
            }
            steps += 1;

            // Build the full message list: system prompt + history.
            let mut request_messages: Vec<Message> =
                Vec::with_capacity(self.messages.len() + 1);
            request_messages.push(Message::system(&self.config.system_prompt));
            request_messages.extend(self.messages.iter().cloned());

            let tools_slice = if tool_schemas.is_empty() {
                None
            } else {
                Some(tool_schemas.as_slice())
            };

            debug!(step = steps, "Calling LLM");
            let response = self
                .llm
                .chat(&self.config.model, &request_messages, tools_slice)
                .await?;

            debug!(finish_reason = %response.finish_reason, "LLM responded");

            match response.finish_reason.as_str() {
                "stop" | "" => {
                    // The assistant is done — record the message and return.
                    let text = response.content.unwrap_or_default();
                    self.messages.push(Message::assistant_text(&text));
                    info!("Agent finished after {} step(s)", steps);
                    return Ok(text);
                }
                "tool_calls" | "function_call" => {
                    let tool_calls = response.tool_calls.ok_or_else(|| {
                        anyhow!("finish_reason is tool_calls but no tool_calls in response")
                    })?;

                    // Record the assistant message with tool calls.
                    self.messages
                        .push(Message::assistant_tool_calls(tool_calls.clone()));

                    // Execute each tool call and append results.
                    for call in &tool_calls {
                        let args: serde_json::Value =
                            serde_json::from_str(&call.function.arguments).unwrap_or_else(|_| {
                                serde_json::json!({})
                            });

                        info!(tool = %call.function.name, "Executing tool");
                        let result = self.registry.call(&call.function.name, args).await;
                        debug!(tool = %call.function.name, result = %result, "Tool result");

                        self.messages
                            .push(Message::tool_result(&call.id, result));
                    }
                    // Continue the loop — send results back to the LLM.
                }
                other => {
                    // Unknown finish reason — treat as done if there's content,
                    // otherwise error.
                    warn!(finish_reason = %other, "Unexpected finish_reason");
                    if let Some(text) = response.content {
                        self.messages.push(Message::assistant_text(&text));
                        return Ok(text);
                    }
                    return Err(anyhow!("Unexpected finish_reason: {}", other));
                }
            }
        }
    }

    /// Return a reference to the accumulated message history.
    pub fn messages(&self) -> &[Message] {
        &self.messages
    }

    /// Clear the conversation history (but keep tools and config).
    pub fn reset(&mut self) {
        self.messages.clear();
    }
}
