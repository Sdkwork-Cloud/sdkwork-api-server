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
