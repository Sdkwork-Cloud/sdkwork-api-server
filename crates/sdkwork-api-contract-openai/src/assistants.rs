use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAssistantRequest {
    pub name: String,
    pub model: String,
}

impl CreateAssistantRequest {
    pub fn new(name: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            model: model.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAssistantRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}

impl UpdateAssistantRequest {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: Some(name.into()),
            model: None,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct AssistantObject {
    pub id: String,
    pub object: &'static str,
    pub name: String,
    pub model: String,
}

impl AssistantObject {
    pub fn new(id: impl Into<String>, name: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            object: "assistant",
            name: name.into(),
            model: model.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ListAssistantsResponse {
    pub object: &'static str,
    pub data: Vec<AssistantObject>,
    pub first_id: Option<String>,
    pub last_id: Option<String>,
    pub has_more: bool,
}

impl ListAssistantsResponse {
    pub fn new(data: Vec<AssistantObject>) -> Self {
        Self {
            object: "list",
            first_id: data.first().map(|assistant| assistant.id.clone()),
            last_id: data.last().map(|assistant| assistant.id.clone()),
            has_more: false,
            data,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DeleteAssistantResponse {
    pub id: String,
    pub object: &'static str,
    pub deleted: bool,
}

impl DeleteAssistantResponse {
    pub fn deleted(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            object: "assistant.deleted",
            deleted: true,
        }
    }
}
