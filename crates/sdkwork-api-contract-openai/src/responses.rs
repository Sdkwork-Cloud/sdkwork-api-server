use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateResponseRequest {
    pub model: String,
    pub input: Value,
    pub stream: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResponseObject {
    pub id: String,
    pub object: &'static str,
    pub model: String,
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

impl ResponseObject {
    pub fn empty(id: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            object: "response",
            model: model.into(),
            output: Vec::new(),
        }
    }
}
