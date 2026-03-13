use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessageInput {
    pub role: String,
    pub content: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateChatCompletionRequest {
    pub model: String,
    pub messages: Vec<ChatMessageInput>,
    pub stream: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateChatCompletionRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
}

impl UpdateChatCompletionRequest {
    pub fn new(metadata: Value) -> Self {
        Self {
            metadata: Some(metadata),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ChunkChoice {
    pub index: u32,
    pub delta: ChunkDelta,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct ChunkDelta {
    pub content: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChatCompletionChunk {
    pub id: String,
    pub object: &'static str,
    pub model: String,
    pub choices: Vec<ChunkChoice>,
}

impl ChatCompletionChunk {
    pub fn empty(id: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            object: "chat.completion.chunk",
            model: model.into(),
            choices: vec![ChunkChoice {
                index: 0,
                delta: ChunkDelta::default(),
                finish_reason: None,
            }],
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ChatCompletionMessage {
    pub role: &'static str,
    pub content: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChatCompletionChoice {
    pub index: u32,
    pub message: ChatCompletionMessage,
    pub finish_reason: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub object: &'static str,
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
    pub choices: Vec<ChatCompletionChoice>,
}

impl ChatCompletionResponse {
    pub fn empty(id: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            object: "chat.completion",
            model: model.into(),
            metadata: None,
            choices: vec![ChatCompletionChoice {
                index: 0,
                message: ChatCompletionMessage {
                    role: "assistant",
                    content: String::new(),
                },
                finish_reason: "stop",
            }],
        }
    }

    pub fn with_metadata(id: impl Into<String>, model: impl Into<String>, metadata: Value) -> Self {
        let mut response = Self::empty(id, model);
        response.metadata = Some(metadata);
        response
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ListChatCompletionsResponse {
    pub object: &'static str,
    pub data: Vec<ChatCompletionResponse>,
}

impl ListChatCompletionsResponse {
    pub fn new(data: Vec<ChatCompletionResponse>) -> Self {
        Self {
            object: "list",
            data,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DeleteChatCompletionResponse {
    pub id: String,
    pub object: &'static str,
    pub deleted: bool,
}

impl DeleteChatCompletionResponse {
    pub fn deleted(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            object: "chat.completion.deleted",
            deleted: true,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ChatCompletionMessageObject {
    pub id: String,
    pub object: &'static str,
    pub role: &'static str,
    pub content: Value,
}

impl ChatCompletionMessageObject {
    pub fn assistant(id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            object: "chat.completion.message",
            role: "assistant",
            content: Value::String(content.into()),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ListChatCompletionMessagesResponse {
    pub object: &'static str,
    pub data: Vec<ChatCompletionMessageObject>,
}

impl ListChatCompletionMessagesResponse {
    pub fn new(data: Vec<ChatCompletionMessageObject>) -> Self {
        Self {
            object: "list",
            data,
        }
    }
}
