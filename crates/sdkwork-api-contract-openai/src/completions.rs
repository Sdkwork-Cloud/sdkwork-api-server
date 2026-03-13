use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCompletionRequest {
    pub model: String,
    pub prompt: String,
}

impl CreateCompletionRequest {
    pub fn new(model: impl Into<String>, prompt: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            prompt: prompt.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CompletionChoice {
    pub index: u32,
    pub text: String,
    pub finish_reason: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct CompletionObject {
    pub id: String,
    pub object: &'static str,
    pub choices: Vec<CompletionChoice>,
}

impl CompletionObject {
    pub fn new(id: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            object: "text_completion",
            choices: vec![CompletionChoice {
                index: 0,
                text: text.into(),
                finish_reason: "stop",
            }],
        }
    }
}
