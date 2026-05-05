use serde::{Deserialize, Serialize};

/// A single tool call requested by the assistant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub call_type: String,
    pub function: FunctionCall,
}

/// The function name and JSON-encoded arguments inside a tool call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String,
}

/// A chat message in OpenAI format.
///
/// Serialises to the wire format expected by `/v1/chat/completions`.
/// The `role` field drives which other fields are present:
///
/// - `"system"` / `"user"` — only `content` is set.
/// - `"assistant"` — `content` may be `None` when `tool_calls` is present.
/// - `"tool"` — `content` holds the tool result; `tool_call_id` identifies
///   which call this is the response for.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "role", rename_all = "lowercase")]
pub enum Message {
    System {
        content: String,
    },
    User {
        content: String,
    },
    Assistant {
        #[serde(skip_serializing_if = "Option::is_none")]
        content: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        tool_calls: Option<Vec<ToolCall>>,
    },
    Tool {
        tool_call_id: String,
        content: String,
    },
}

impl Message {
    pub fn system(content: impl Into<String>) -> Self {
        Message::System { content: content.into() }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Message::User { content: content.into() }
    }

    pub fn assistant_text(content: impl Into<String>) -> Self {
        Message::Assistant {
            content: Some(content.into()),
            tool_calls: None,
        }
    }

    pub fn assistant_tool_calls(tool_calls: Vec<ToolCall>) -> Self {
        Message::Assistant {
            content: None,
            tool_calls: Some(tool_calls),
        }
    }

    pub fn tool_result(tool_call_id: impl Into<String>, content: impl Into<String>) -> Self {
        Message::Tool {
            tool_call_id: tool_call_id.into(),
            content: content.into(),
        }
    }
}
