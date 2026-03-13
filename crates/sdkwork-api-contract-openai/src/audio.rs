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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTranslationRequest {
    pub model: String,
    pub file_id: String,
}

impl CreateTranslationRequest {
    pub fn new(model: impl Into<String>, file_id: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            file_id: file_id.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct TranslationObject {
    pub text: String,
}

impl TranslationObject {
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSpeechRequest {
    pub model: String,
    pub voice: String,
    pub input: String,
}

impl CreateSpeechRequest {
    pub fn new(
        model: impl Into<String>,
        voice: impl Into<String>,
        input: impl Into<String>,
    ) -> Self {
        Self {
            model: model.into(),
            voice: voice.into(),
            input: input.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SpeechResponse {
    pub format: String,
    pub audio_base64: String,
}

impl SpeechResponse {
    pub fn new(format: impl Into<String>, audio_base64: impl Into<String>) -> Self {
        Self {
            format: format.into(),
            audio_base64: audio_base64.into(),
        }
    }
}
