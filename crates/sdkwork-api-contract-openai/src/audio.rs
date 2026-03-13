use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTranscriptionRequest {
    pub model: String,
    pub file_id: String,
}

impl CreateTranscriptionRequest {
    pub fn new(model: impl Into<String>, file_id: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            file_id: file_id.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct TranscriptionObject {
    pub text: String,
}

impl TranscriptionObject {
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }
}
