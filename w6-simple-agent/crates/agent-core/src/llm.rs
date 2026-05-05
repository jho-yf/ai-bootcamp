use anyhow::{anyhow, Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::message::{FunctionCall, Message, ToolCall};

// ---------------------------------------------------------------------------
// Request / response types
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: &'a [Message],
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<&'a [Value]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_choice: Option<&'static str>,
}

#[derive(Debug, Deserialize)]
pub struct ChatResponse {
    pub choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
pub struct Choice {
    pub message: AssistantMessage,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AssistantMessage {
    pub content: Option<String>,
    pub tool_calls: Option<Vec<RawToolCall>>,
}

#[derive(Debug, Deserialize)]
pub struct RawToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub call_type: String,
    pub function: RawFunctionCall,
}

#[derive(Debug, Deserialize)]
pub struct RawFunctionCall {
    pub name: String,
    pub arguments: String,
}

// ---------------------------------------------------------------------------
// LLM client
// ---------------------------------------------------------------------------

/// Thin wrapper around an OpenAI-compatible `/v1/chat/completions` endpoint.
pub struct LlmClient {
    client: Client,
    api_base: String,
    api_key: String,
}

impl LlmClient {
    pub fn new(api_base: impl Into<String>, api_key: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            api_base: api_base.into(),
            api_key: api_key.into(),
        }
    }

    /// Send a chat completion request and return the parsed response.
    pub async fn chat(
        &self,
        model: &str,
        messages: &[Message],
        tools: Option<&[Value]>,
    ) -> Result<LlmResponse> {
        let url = format!("{}/v1/chat/completions", self.api_base.trim_end_matches('/'));

        let tool_choice = tools.filter(|t| !t.is_empty()).map(|_| "auto");

        let body = ChatRequest {
            model,
            messages,
            tools: tools.filter(|t| !t.is_empty()),
            tool_choice,
        };

        let resp = self
            .client
            .post(&url)
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await
            .context("HTTP request to LLM failed")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(anyhow!("LLM API error {}: {}", status, text));
        }

        let raw: ChatResponse = resp.json().await.context("Failed to parse LLM response")?;

        let choice = raw.choices.into_iter().next().ok_or_else(|| anyhow!("No choices in LLM response"))?;

        let finish_reason = choice.finish_reason.unwrap_or_default();

        let tool_calls: Option<Vec<ToolCall>> = choice.message.tool_calls.map(|calls| {
            calls
                .into_iter()
                .map(|c| ToolCall {
                    id: c.id,
                    call_type: c.call_type,
                    function: FunctionCall {
                        name: c.function.name,
                        arguments: c.function.arguments,
                    },
                })
                .collect()
        });

        Ok(LlmResponse {
            content: choice.message.content,
            tool_calls,
            finish_reason,
        })
    }
}

/// Parsed result from a single LLM turn.
#[derive(Debug)]
pub struct LlmResponse {
    /// Text content from the assistant (may be `None` when tool calls are present).
    pub content: Option<String>,
    /// Tool calls requested by the assistant, if any.
    pub tool_calls: Option<Vec<ToolCall>>,
    /// `"stop"` means the assistant is done; `"tool_calls"` means it wants to
    /// invoke tools.
    pub finish_reason: String,
}
