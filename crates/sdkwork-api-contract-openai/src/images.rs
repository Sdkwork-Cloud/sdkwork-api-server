use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateImageRequest {
    pub model: String,
    pub prompt: String,
}

impl CreateImageRequest {
    pub fn new(model: impl Into<String>, prompt: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            prompt: prompt.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ImageObject {
    pub b64_json: String,
}

impl ImageObject {
    pub fn base64(payload: impl Into<String>) -> Self {
        Self {
            b64_json: payload.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ImagesResponse {
    pub data: Vec<ImageObject>,
}

impl ImagesResponse {
    pub fn new(data: Vec<ImageObject>) -> Self {
        Self { data }
    }
}
