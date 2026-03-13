use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateResponseRequest {
    pub model: String,
    pub input: Value,
    pub stream: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountResponseInputTokensRequest {
    pub model: String,
    pub input: Value,
}

impl CountResponseInputTokensRequest {
    pub fn new(model: impl Into<String>, input: Value) -> Self {
        Self {
            model: model.into(),
            input,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactResponseRequest {
    pub model: String,
    pub input: Value,
}

impl CompactResponseRequest {
    pub fn new(model: impl Into<String>, input: Value) -> Self {
        Self {
            model: model.into(),
            input,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ResponseObject {
    pub id: String,
    pub object: &'static str,
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    pub output: Vec<ResponseOutputItem>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResponseOutputItem {
    pub r#type: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResponseInputItemObject {
    pub id: String,
    pub object: &'static str,
    pub r#type: &'static str,
}

impl ResponseInputItemObject {
    pub fn message(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            object: "response.input_item",
            r#type: "message",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ListResponseInputItemsResponse {
    pub object: &'static str,
    pub data: Vec<ResponseInputItemObject>,
}

impl ListResponseInputItemsResponse {
    pub fn new(data: Vec<ResponseInputItemObject>) -> Self {
        Self {
            object: "list",
            data,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DeleteResponseResponse {
    pub id: String,
    pub object: &'static str,
    pub deleted: bool,
}

impl DeleteResponseResponse {
    pub fn deleted(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            object: "response.deleted",
            deleted: true,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ResponseInputTokensObject {
    pub object: &'static str,
    pub input_tokens: u64,
}

impl ResponseInputTokensObject {
    pub fn new(input_tokens: u64) -> Self {
        Self {
            object: "response.input_tokens",
            input_tokens,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ResponseCompactionObject {
    pub id: String,
    pub object: &'static str,
    pub model: String,
}

impl ResponseCompactionObject {
    pub fn new(id: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            object: "response.compaction",
            model: model.into(),
        }
    }
}

impl ResponseObject {
    pub fn empty(id: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            object: "response",
            model: model.into(),
            status: None,
            output: Vec::new(),
        }
    }

    pub fn cancelled(id: impl Into<String>, model: impl Into<String>) -> Self {
        let mut response = Self::empty(id, model);
        response.status = Some("cancelled".to_owned());
        response
    }
}
